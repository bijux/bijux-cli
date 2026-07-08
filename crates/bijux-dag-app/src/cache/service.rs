use crate::run_data::{env_cache_dir, load_snapshot};
use crate::{read_file, ExitCode};
use bijux_dag_artifacts::{CacheIdentity, OutputsIndex};
use bijux_dag_runtime::{
    cache_entry_has_required_proof, cache_entry_manifest_version_supported,
    cache_explainability_proof_from_meta, cache_key_explanation, cache_key_input_from_meta,
    cache_metadata_version_supported, CacheEntryManifest, CacheExplainabilityProof,
    CacheManifestOutput,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use tar::{Archive, Builder};

const MAX_CACHE_ARCHIVE_TOTAL_BYTES: u64 = 8 * 1024 * 1024;
const MAX_CACHE_ARCHIVE_ENTRIES: usize = 10_000;

fn hash_bytes(bytes: &[u8]) -> String {
    bijux_dag_artifacts::hash::sha256_hex(bytes)
}

#[derive(Debug, Clone)]
struct CacheEntryCandidate {
    key: String,
    meta: Value,
    explainability: Option<CacheExplainabilityProof>,
    valid: bool,
}

#[derive(Debug, Clone)]
struct CacheMissReason {
    code: &'static str,
    message: String,
}

fn cache_entry_exists(cache_dir: &Path, key: &str) -> bool {
    cache_dir.join(key).is_dir()
}

fn load_cache_entry_candidate(entry: &Path) -> Result<Option<CacheEntryCandidate>, ExitCode> {
    let manifest_path = entry.join("manifest.json");
    let meta_path = entry.join("meta.json");
    if !manifest_path.exists() || !meta_path.exists() {
        return Ok(None);
    }
    let manifest: CacheEntryManifest =
        serde_json::from_str(&fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let meta: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    if !cache_metadata_version_supported(&meta)
        || !cache_entry_manifest_version_supported(&manifest)
    {
        return Ok(None);
    }
    let key = meta.get("cache_key").and_then(Value::as_str).unwrap_or_default().to_string();
    let valid = if key.is_empty() { false } else { verify_cache_entry_cli(entry, &key, "", "")? };
    Ok(Some(CacheEntryCandidate {
        key,
        explainability: cache_explainability_proof_from_meta(&meta),
        meta,
        valid,
    }))
}

fn matching_factor_score(
    identity: &CacheIdentity,
    trace: &Value,
    candidate: &CacheEntryCandidate,
) -> usize {
    let adapter_outputs_schema_version =
        trace.get("adapter_outputs_schema_version").and_then(Value::as_str).unwrap_or_default();
    let mut score = 0usize;
    let checks = [
        (
            candidate.meta.get("node_definition_fingerprint").and_then(Value::as_str),
            Some(identity.node_definition_fingerprint.as_str()),
        ),
        (
            candidate.meta.get("declared_environment_fingerprint").and_then(Value::as_str),
            Some(identity.declared_environment_fingerprint.as_str()),
        ),
        (
            candidate.meta.get("input_lineage_fingerprint").and_then(Value::as_str),
            Some(identity.input_lineage_fingerprint.as_str()),
        ),
        (
            candidate.meta.get("adapter_id").and_then(Value::as_str),
            trace.get("adapter_id").and_then(Value::as_str),
        ),
        (
            candidate.meta.get("adapter_version").and_then(Value::as_str),
            trace.get("adapter_version").and_then(Value::as_str),
        ),
        (
            candidate
                .meta
                .get("produces_outputs_schema_version")
                .or_else(|| candidate.meta.get("output_schema_version"))
                .and_then(Value::as_str),
            Some(adapter_outputs_schema_version),
        ),
        (
            candidate.meta.get("policy_fingerprint").and_then(Value::as_str),
            Some(identity.policy_fingerprint.as_str()),
        ),
        (
            candidate.meta.get("execution_contract_fingerprint").and_then(Value::as_str),
            Some(identity.execution_contract_fingerprint.as_str()),
        ),
        (
            candidate.meta.get("backend_class").and_then(Value::as_str),
            Some(identity.backend_class.as_str()),
        ),
    ];
    for (stored, current) in checks {
        if stored == current {
            score += 1;
        }
    }
    score
}

fn compare_cache_candidate(
    identity: &CacheIdentity,
    trace: &Value,
    candidate: &CacheEntryCandidate,
) -> Vec<CacheMissReason> {
    let mut reasons = Vec::new();

    if candidate.explainability.as_ref().map(|proof| proof.params_fingerprint.as_str())
        != Some(identity.params_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_params",
            message: "changed params".to_string(),
        });
    }
    if candidate.meta.get("input_lineage_fingerprint").and_then(Value::as_str)
        != Some(identity.input_lineage_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_input_hashes",
            message: "changed input hashes".to_string(),
        });
    }
    if let (Some(current), Some(stored)) = (
        identity.command_fingerprint.as_deref(),
        candidate.explainability.as_ref().and_then(|proof| proof.command_fingerprint.as_deref()),
    ) {
        if stored != current {
            reasons.push(CacheMissReason {
                code: "changed_command",
                message: "changed command".to_string(),
            });
        }
    }

    let adapter_outputs_schema_version =
        trace.get("adapter_outputs_schema_version").and_then(Value::as_str).unwrap_or_default();
    let adapter_changed = candidate.meta.get("adapter_id").and_then(Value::as_str)
        != trace.get("adapter_id").and_then(Value::as_str)
        || candidate.meta.get("adapter_version").and_then(Value::as_str)
            != trace.get("adapter_version").and_then(Value::as_str)
        || candidate
            .meta
            .get("produces_outputs_schema_version")
            .or_else(|| candidate.meta.get("output_schema_version"))
            .and_then(Value::as_str)
            != Some(adapter_outputs_schema_version);
    if adapter_changed {
        reasons.push(CacheMissReason {
            code: "changed_adapter_identity",
            message: "changed adapter identity".to_string(),
        });
    }

    if candidate.meta.get("policy_fingerprint").and_then(Value::as_str)
        != Some(identity.policy_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "policy_bypass",
            message: "policy-based cache bypass".to_string(),
        });
    }

    if reasons.is_empty()
        && candidate.meta.get("node_definition_fingerprint").and_then(Value::as_str)
            != Some(identity.node_definition_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_node_definition",
            message: "node definition changed".to_string(),
        });
    }
    if candidate.meta.get("declared_environment_fingerprint").and_then(Value::as_str)
        != Some(identity.declared_environment_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_environment",
            message: "declared environment changed".to_string(),
        });
    }
    if candidate.meta.get("execution_contract_fingerprint").and_then(Value::as_str)
        != Some(identity.execution_contract_fingerprint.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_execution_contract",
            message: "execution contract changed".to_string(),
        });
    }
    if candidate.meta.get("backend_class").and_then(Value::as_str)
        != Some(identity.backend_class.as_str())
    {
        reasons.push(CacheMissReason {
            code: "changed_backend",
            message: "backend changed".to_string(),
        });
    }

    reasons
}

fn cache_candidates_for_node(
    cache_dir: &Path,
    node_id: &str,
) -> Result<Vec<CacheEntryCandidate>, ExitCode> {
    let mut candidates = Vec::new();
    if !cache_dir.exists() {
        return Ok(candidates);
    }
    for entry in fs::read_dir(cache_dir).map_err(|_| ExitCode::from(3))? {
        let entry = entry.map_err(|_| ExitCode::from(3))?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(candidate) = load_cache_entry_candidate(&path)? else {
            continue;
        };
        if candidate.meta.get("node_id").and_then(Value::as_str) == Some(node_id) {
            candidates.push(candidate);
        }
    }
    candidates.sort_by(|left, right| left.key.cmp(&right.key));
    Ok(candidates)
}

fn verify_cache_dir(dir: &Path) -> Result<Value, ExitCode> {
    let mut checked = 0u64;
    let mut corrupt = 0u64;
    let mut corrupt_keys: Vec<String> = Vec::new();
    if dir.exists() {
        for entry in fs::read_dir(dir).map_err(|_| ExitCode::from(3))? {
            let entry = entry.map_err(|_| ExitCode::from(3))?;
            let path = entry.path();
            if path.is_dir() {
                checked += 1;
                let key = entry.file_name().to_string_lossy().to_string();
                if !verify_cache_entry_cli(&path, &key, "", "")? {
                    corrupt += 1;
                    corrupt_keys.push(key);
                }
            }
        }
    }
    corrupt_keys.sort();
    corrupt_keys.dedup();
    Ok(json!({ "checked": checked, "corrupt": corrupt, "corrupt_keys": corrupt_keys }))
}

pub(crate) fn verify_cache_dirs(local: &Path, remote: Option<&Path>) -> Result<Value, ExitCode> {
    let local_report = verify_cache_dir(local)?;
    let mut checked_total = local_report["checked"].as_u64().unwrap_or(0);
    let mut corrupt_total = local_report["corrupt"].as_u64().unwrap_or(0);
    let mut out = json!({
        "local": local_report,
        "checked_total": checked_total,
        "corrupt_total": corrupt_total,
    });
    if let Some(remote_dir) = remote {
        let remote_report = verify_cache_dir(remote_dir)?;
        checked_total += remote_report["checked"].as_u64().unwrap_or(0);
        corrupt_total += remote_report["corrupt"].as_u64().unwrap_or(0);
        out["remote"] = remote_report;
        out["checked_total"] = json!(checked_total);
        out["corrupt_total"] = json!(corrupt_total);
    }
    Ok(out)
}

pub(crate) fn pack_cache_entry(entry: &Path, out: &Path) -> Result<(), ExitCode> {
    let file = fs::File::create(out).map_err(|_| ExitCode::from(3))?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut builder = Builder::new(enc);
    append_cache_entry_archive(&mut builder, entry, entry)?;
    let enc = builder.into_inner().map_err(|_| ExitCode::from(3))?;
    enc.finish().map_err(|_| ExitCode::from(3))?;
    Ok(())
}

fn append_cache_entry_archive<W: std::io::Write>(
    builder: &mut Builder<W>,
    root: &Path,
    current: &Path,
) -> Result<(), ExitCode> {
    let mut entries = fs::read_dir(current)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok())
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let path = entry.path();
        let rel = path.strip_prefix(root).map_err(|_| ExitCode::from(3))?;
        let file_type = entry.file_type().map_err(|_| ExitCode::from(3))?;
        if file_type.is_dir() {
            builder.append_dir(rel, &path).map_err(|_| ExitCode::from(3))?;
            append_cache_entry_archive(builder, root, &path)?;
        } else if file_type.is_file() {
            builder.append_path_with_name(&path, rel).map_err(|_| ExitCode::from(3))?;
        } else {
            return Err(ExitCode::from(3));
        }
    }
    Ok(())
}

pub(crate) fn unpack_cache_entry(pack: &Path, cache_dir: &Path) -> Result<(), ExitCode> {
    let file = fs::File::open(pack).map_err(|_| ExitCode::from(3))?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    let tmp = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
    unpack_cache_archive_bounded(&mut archive, tmp.path())?;
    let unpacked_root = unpacked_cache_entry_root(tmp.path())?;
    let meta_path = unpacked_root.join("meta.json");
    let mut meta: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let key = meta.get("cache_key").and_then(|v| v.as_str()).ok_or(ExitCode::from(3))?.to_string();
    let adapter_id = meta.get("adapter_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let adapter_version =
        meta.get("adapter_version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !verify_cache_entry_cli(&unpacked_root, &key, &adapter_id, &adapter_version)? {
        return Err(ExitCode::from(3));
    }
    if let Some(obj) = meta.as_object_mut() {
        obj.insert("cache_source".to_string(), Value::String("pack".to_string()));
    }
    fs::write(&meta_path, serde_json::to_vec_pretty(&meta).unwrap())
        .map_err(|_| ExitCode::from(3))?;
    let dst = cache_dir.join(&key);
    if dst.exists() {
        let _ = fs::remove_dir_all(&dst);
    }
    copy_dir_all(&unpacked_root, &dst).map_err(|_| ExitCode::from(3))?;
    Ok(())
}

fn unpacked_cache_entry_root(root: &Path) -> Result<std::path::PathBuf, ExitCode> {
    if root.join("meta.json").exists() {
        return Ok(root.to_path_buf());
    }
    let mut children = fs::read_dir(root)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    children.sort();
    if children.len() == 1 && children[0].join("meta.json").exists() {
        return Ok(children.remove(0));
    }
    Err(ExitCode::from(3))
}

pub(crate) fn unpack_cache_archive_bounded<R: std::io::Read>(
    archive: &mut Archive<R>,
    dst: &Path,
) -> Result<(), ExitCode> {
    let mut total_bytes: u64 = 0;
    let mut entry_count: usize = 0;
    let entries = archive.entries().map_err(|_| ExitCode::from(3))?;
    for entry in entries {
        let mut entry = entry.map_err(|_| ExitCode::from(3))?;
        entry_count += 1;
        if entry_count > MAX_CACHE_ARCHIVE_ENTRIES {
            return Err(ExitCode::from(3));
        }
        let header = entry.header();
        let kind = header.entry_type();
        if !(kind.is_file() || kind.is_dir()) {
            return Err(ExitCode::from(3));
        }
        total_bytes = total_bytes.saturating_add(header.size().map_err(|_| ExitCode::from(3))?);
        if total_bytes > MAX_CACHE_ARCHIVE_TOTAL_BYTES {
            return Err(ExitCode::from(3));
        }
        entry.unpack_in(dst).map_err(|_| ExitCode::from(3))?;
    }
    Ok(())
}

fn cache_manifest_output_matches(
    manifest_output: &CacheManifestOutput,
    file: &bijux_dag_artifacts::OutputFile,
) -> bool {
    manifest_output.path == file.path
        && manifest_output.name == file.name
        && manifest_output.kind == file.kind
        && manifest_output.media_type == file.media_type
}

pub(crate) fn verify_cache_entry_cli(
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
) -> Result<bool, ExitCode> {
    let index_path = entry.join("outputs").join("index.json");
    let meta_path = entry.join("meta.json");
    let manifest_path = entry.join("manifest.json");
    if !index_path.exists() || !meta_path.exists() || !manifest_path.exists() {
        return Ok(false);
    }
    let meta_raw = fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?;
    let meta: Value = serde_json::from_str(&meta_raw).map_err(|_| ExitCode::from(3))?;
    if meta.get("cache_key").and_then(|v| v.as_str()) != Some(expected_key) {
        return Ok(false);
    }
    if !cache_metadata_version_supported(&meta) || !cache_entry_has_required_proof(&meta) {
        return Ok(false);
    }
    let Some(key_input) = cache_key_input_from_meta(&meta) else {
        return Ok(false);
    };
    if cache_key_explanation(&key_input).key != expected_key {
        return Ok(false);
    }
    let manifest_raw = fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?;
    let manifest: CacheEntryManifest =
        serde_json::from_str(&manifest_raw).map_err(|_| ExitCode::from(3))?;
    if !cache_entry_manifest_version_supported(&manifest) {
        return Ok(false);
    }
    if manifest.cache_key != expected_key {
        return Ok(false);
    }
    if !adapter_id.is_empty() && meta.get("adapter_id").and_then(|v| v.as_str()) != Some(adapter_id)
    {
        return Ok(false);
    }
    if !adapter_version.is_empty()
        && meta.get("adapter_version").and_then(|v| v.as_str()) != Some(adapter_version)
    {
        return Ok(false);
    }
    let data = fs::read_to_string(&index_path).map_err(|_| ExitCode::from(3))?;
    let index: OutputsIndex = serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
    let node_fingerprint =
        meta.get("node_fingerprint").and_then(|v| v.as_str()).unwrap_or_default();
    for expected_output in &manifest.outputs {
        let indexed = index.files.iter().find(|file| file.path == expected_output.path);
        if expected_output.required && indexed.is_none() {
            return Ok(false);
        }
        let Some(file) = indexed else {
            continue;
        };
        if !cache_manifest_output_matches(expected_output, file)
            || file.node_id != manifest.node_id
            || file.node_fingerprint != node_fingerprint
        {
            return Ok(false);
        }
    }
    for file in index.files {
        if !manifest.outputs.iter().any(|output| cache_manifest_output_matches(output, &file)) {
            return Ok(false);
        }
        let fpath = entry.join("outputs").join(&file.path);
        if !fpath.exists() {
            return Ok(false);
        }
        let bytes = fs::read(&fpath).map_err(|_| ExitCode::from(3))?;
        let sha = hash_bytes(&bytes);
        if sha != file.sha256 {
            return Ok(false);
        }
    }
    Ok(true)
}

pub(crate) fn explain_cache_key(
    cache_dir: &Path,
    key: &str,
    expected_adapter_id: &str,
    expected_adapter_version: &str,
) -> Result<Value, ExitCode> {
    let entry = cache_dir.join(key);
    let mut reasons = Vec::new();
    let mut taxonomy = Vec::new();
    if !entry.exists() {
        reasons.push("missing cache entry directory".to_string());
        taxonomy.push("no_entry".to_string());
        return Ok(json!({
            "key": key,
            "eligible": false,
            "reasons": reasons,
            "taxonomy": taxonomy
        }));
    }
    let meta_path = entry.join("meta.json");
    let index_path = entry.join("outputs").join("index.json");
    let manifest_path = entry.join("manifest.json");
    if !meta_path.exists() {
        reasons.push("missing meta.json".to_string());
        taxonomy.push("artifact_missing".to_string());
    }
    if !index_path.exists() {
        reasons.push("missing outputs/index.json".to_string());
        taxonomy.push("artifact_missing".to_string());
    }
    if !manifest_path.exists() {
        reasons.push("missing manifest.json".to_string());
        taxonomy.push("artifact_missing".to_string());
    }
    let mut meta = Value::Null;
    let mut key_components = Value::Null;
    let mut manifest = Value::Null;
    if meta_path.exists() {
        meta = serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        if let Some(key_input) = cache_key_input_from_meta(&meta) {
            let explanation = cache_key_explanation(&key_input);
            key_components = json!({
                "execution_fingerprint_component": meta.get("node_fingerprint").cloned().unwrap_or(Value::Null),
                "node_definition_component": meta.get("node_definition_fingerprint").cloned().unwrap_or(Value::Null),
                "declared_environment_component": meta.get("declared_environment_fingerprint").cloned().unwrap_or(Value::Null),
                "input_lineage_component": meta.get("input_lineage_fingerprint").cloned().unwrap_or(Value::Null),
                "adapter_component": {
                    "adapter_id": meta.get("adapter_id").cloned().unwrap_or(Value::Null),
                    "adapter_version": meta.get("adapter_version").cloned().unwrap_or(Value::Null),
                    "output_schema_version": meta.get("produces_outputs_schema_version")
                        .or_else(|| meta.get("output_schema_version"))
                        .cloned()
                        .unwrap_or(Value::Null),
                },
                "policy_component": meta.get("policy_fingerprint").cloned().unwrap_or(Value::Null),
                "execution_contract_component": meta.get("execution_contract_fingerprint")
                    .cloned()
                    .unwrap_or(Value::Null),
                "backend_component": meta.get("backend_class").cloned().unwrap_or(Value::Null),
                "computed_cache_key": explanation.key,
                "stored_cache_key": meta.get("cache_key").cloned().unwrap_or(Value::Null),
                "intentional_inputs": explanation.intentional_inputs,
            });
            if meta.get("cache_key").and_then(|v| v.as_str()) != Some(explanation.key.as_str()) {
                reasons.push("stored cache key does not match persisted proof fields".to_string());
                taxonomy.push("hash_mismatch".to_string());
            }
        }
        if meta.get("cache_key").and_then(|v| v.as_str()) != Some(key) {
            reasons.push("cache key does not match requested key".to_string());
            taxonomy.push("hash_mismatch".to_string());
        }
        if !expected_adapter_id.is_empty()
            && meta.get("adapter_id").and_then(|v| v.as_str()) != Some(expected_adapter_id)
        {
            reasons.push("adapter_id mismatch".to_string());
            taxonomy.push("adapter_mismatch".to_string());
        }
        if !expected_adapter_version.is_empty()
            && meta.get("adapter_version").and_then(|v| v.as_str())
                != Some(expected_adapter_version)
        {
            reasons.push("adapter_version mismatch".to_string());
            taxonomy.push("adapter_mismatch".to_string());
        }
        if !cache_metadata_version_supported(&meta) {
            reasons.push("cache metadata version is unsupported".to_string());
            taxonomy.push("schema_mismatch".to_string());
        }
        if !cache_entry_has_required_proof(&meta) {
            reasons.push("cache entry is missing required proof fields".to_string());
            taxonomy.push("policy_mismatch".to_string());
        }
    }
    if manifest_path.exists() {
        manifest = serde_json::from_str::<Value>(
            &fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        let parsed: CacheEntryManifest =
            serde_json::from_value(manifest.clone()).map_err(|_| ExitCode::from(3))?;
        if !cache_entry_manifest_version_supported(&parsed) {
            reasons.push("cache manifest version is unsupported".to_string());
            taxonomy.push("schema_mismatch".to_string());
        }
        if parsed.cache_key != key {
            reasons.push("cache manifest key does not match requested key".to_string());
            taxonomy.push("hash_mismatch".to_string());
        }
    }
    let proof_valid = verify_cache_entry_cli(
        entry.as_path(),
        key,
        expected_adapter_id,
        expected_adapter_version,
    )?;
    let eligible = reasons.is_empty() && proof_valid;
    if !proof_valid {
        taxonomy.push("artifact_corrupt".to_string());
    }
    if !eligible && reasons.is_empty() {
        reasons.push("output proof verification failed".to_string());
    }
    taxonomy.sort();
    taxonomy.dedup();
    Ok(json!({
        "key": key,
        "eligible": eligible,
        "entry_dir": entry,
        "meta": meta,
        "manifest": manifest,
        "reasons": reasons,
        "taxonomy": taxonomy,
        "key_components": key_components,
        "proof_verified": proof_valid
    }))
}

pub(crate) fn explain_run_node_cache_miss(
    run_dir: &Path,
    node_id: &str,
    cache_dir_override: Option<&Path>,
) -> Result<Value, ExitCode> {
    let manifest: Value = serde_json::from_str(&read_file(&run_dir.join("manifest.json"))?)
        .map_err(|_| ExitCode::from(3))?;
    let snapshot = load_snapshot(run_dir)?;
    let node =
        snapshot.graph.nodes.iter().find(|node| node.id == node_id).ok_or(ExitCode::from(3))?;
    let trace: Value =
        serde_json::from_str(&read_file(&run_dir.join("nodes").join(node_id).join("trace.json"))?)
            .map_err(|_| ExitCode::from(3))?;
    let cache_identity: CacheIdentity =
        serde_json::from_value(trace.get("cache_identity").cloned().ok_or(ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let cache_mode =
        manifest.get("cache_mode").and_then(Value::as_str).unwrap_or("Off").to_string();
    let cache_dir = cache_dir_override
        .map(Path::to_path_buf)
        .or_else(|| manifest.get("cache_dir").and_then(Value::as_str).map(PathBuf::from))
        .or_else(env_cache_dir);

    let (outcome, reasons, taxonomy, comparison_entry, exact_entry_report) = if !node.cache.enabled
    {
        (
            "non_cacheable".to_string(),
            vec![node
                .cache
                .reason
                .clone()
                .unwrap_or_else(|| "policy-based cache bypass".to_string())],
            vec!["policy_bypass".to_string()],
            Value::Null,
            Value::Null,
        )
    } else if cache_mode == "Off" {
        (
            "miss".to_string(),
            vec!["policy-based cache bypass".to_string()],
            vec!["policy_bypass".to_string()],
            Value::Null,
            Value::Null,
        )
    } else {
        if let Some(cache_dir) = cache_dir.as_ref() {
            let exact_entry_exists = cache_entry_exists(cache_dir, &cache_identity.cache_key);
            let exact_entry_report = if exact_entry_exists {
                explain_cache_key(
                    cache_dir,
                    &cache_identity.cache_key,
                    trace.get("adapter_id").and_then(Value::as_str).unwrap_or_default(),
                    trace.get("adapter_version").and_then(Value::as_str).unwrap_or_default(),
                )?
            } else {
                Value::Null
            };

            if exact_entry_exists
                && exact_entry_report.get("eligible").and_then(Value::as_bool) == Some(false)
            {
                let taxonomy = exact_entry_report
                    .get("taxonomy")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_else(|| vec!["corrupt_entry".to_string()]);
                let reasons = exact_entry_report
                    .get("reasons")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                    })
                    .filter(|items| !items.is_empty())
                    .unwrap_or_else(|| vec!["missing/corrupt cache entry".to_string()]);
                (
                    "unsafe_reuse_refused".to_string(),
                    reasons,
                    taxonomy,
                    json!({ "key": cache_identity.cache_key, "valid": false }),
                    exact_entry_report,
                )
            } else if exact_entry_exists
                && exact_entry_report.get("eligible").and_then(Value::as_bool) == Some(true)
                && trace.get("status").and_then(Value::as_str) == Some("cached")
            {
                (
                    "hit".to_string(),
                    vec!["cache key matched all compatibility factors".to_string()],
                    vec!["hit".to_string()],
                    json!({ "key": cache_identity.cache_key, "valid": true }),
                    exact_entry_report,
                )
            } else {
                let mut candidates = cache_candidates_for_node(cache_dir, node_id)?;
                candidates.retain(|candidate| candidate.key != cache_identity.cache_key);
                if candidates.is_empty() {
                    (
                        "miss".to_string(),
                        vec!["missing/corrupt cache entry".to_string()],
                        vec!["missing_entry".to_string()],
                        Value::Null,
                        exact_entry_report,
                    )
                } else {
                    candidates.sort_by(|left, right| {
                        let left_score = matching_factor_score(&cache_identity, &trace, left);
                        let right_score = matching_factor_score(&cache_identity, &trace, right);
                        right_score
                            .cmp(&left_score)
                            .then_with(|| right.valid.cmp(&left.valid))
                            .then_with(|| left.key.cmp(&right.key))
                    });
                    let candidate = candidates.remove(0);
                    let reasons = compare_cache_candidate(&cache_identity, &trace, &candidate);
                    let reason_messages =
                        reasons.iter().map(|reason| reason.message.clone()).collect::<Vec<_>>();
                    let taxonomy =
                        reasons.iter().map(|reason| reason.code.to_string()).collect::<Vec<_>>();
                    (
                        if candidate.valid {
                            "miss".to_string()
                        } else {
                            "unsafe_reuse_refused".to_string()
                        },
                        if reason_messages.is_empty() {
                            vec!["missing/corrupt cache entry".to_string()]
                        } else {
                            reason_messages
                        },
                        if taxonomy.is_empty() {
                            vec!["missing_entry".to_string()]
                        } else {
                            taxonomy
                        },
                        json!({ "key": candidate.key, "valid": candidate.valid }),
                        exact_entry_report,
                    )
                }
            }
        } else {
            (
                "miss".to_string(),
                vec!["missing cache entry directory".to_string()],
                vec!["missing_entry".to_string()],
                Value::Null,
                Value::Null,
            )
        }
    };

    Ok(json!({
        "mode": "node",
        "run_dir": run_dir,
        "node_id": node_id,
        "cache_dir": cache_dir,
        "cache_mode": cache_mode,
        "cache_key": cache_identity.cache_key,
        "outcome": outcome,
        "reasons": reasons,
        "taxonomy": taxonomy,
        "comparison_entry": comparison_entry,
        "cache_identity": cache_identity,
        "exact_entry_report": exact_entry_report,
    }))
}

pub(crate) fn cache_stats(cache_dir: &Path) -> Result<Value, ExitCode> {
    if !cache_dir.exists() {
        return Ok(json!({
            "entries": 0,
            "bytes": 0u64,
            "invalid_entries": 0,
            "hit_potential": "none"
        }));
    }
    let mut entries = 0u64;
    let mut bytes = 0u64;
    let mut invalid_entries = 0u64;
    for dirent in fs::read_dir(cache_dir).map_err(|_| ExitCode::from(3))? {
        let dirent = dirent.map_err(|_| ExitCode::from(3))?;
        if !dirent.path().is_dir() {
            continue;
        }
        entries += 1;
        let key = dirent.file_name().to_string_lossy().to_string();
        let path = dirent.path();
        let valid = verify_cache_entry_cli(&path, &key, "", "")?;
        if !valid {
            invalid_entries += 1;
        }
        bytes += dir_size_bytes(&path)?;
    }
    let hit_potential = if entries == 0 {
        "none"
    } else if invalid_entries == 0 {
        "high"
    } else if invalid_entries * 2 < entries {
        "medium"
    } else {
        "low"
    };
    Ok(json!({
        "entries": entries,
        "bytes": bytes,
        "invalid_entries": invalid_entries,
        "hit_potential": hit_potential
    }))
}

pub(crate) fn cache_prune_simulate(cache_dir: &Path) -> Result<Value, ExitCode> {
    if !cache_dir.exists() {
        return Ok(json!({"would_remove": [], "reason": "cache directory missing"}));
    }
    let mut would_remove = Vec::new();
    for dirent in fs::read_dir(cache_dir).map_err(|_| ExitCode::from(3))? {
        let dirent = dirent.map_err(|_| ExitCode::from(3))?;
        if !dirent.path().is_dir() {
            continue;
        }
        let key = dirent.file_name().to_string_lossy().to_string();
        let valid = verify_cache_entry_cli(&dirent.path(), &key, "", "")?;
        if !valid {
            would_remove.push(key);
        }
    }
    would_remove.sort();
    Ok(json!({
        "would_remove": would_remove,
        "policy": "invalid entries only (simulation)"
    }))
}

pub(crate) fn cache_diff(cache_dir: &Path, key_a: &str, key_b: &str) -> Result<Value, ExitCode> {
    fn load_meta(entry: &Path) -> Result<Value, ExitCode> {
        let meta_path = entry.join("meta.json");
        if !meta_path.exists() {
            return Ok(json!({}));
        }
        serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))
    }

    fn load_manifest(entry: &Path) -> Result<Value, ExitCode> {
        let manifest_path = entry.join("manifest.json");
        if !manifest_path.exists() {
            return Ok(json!({}));
        }
        serde_json::from_str::<Value>(
            &fs::read_to_string(&manifest_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))
    }

    let a_path = cache_dir.join(key_a);
    let b_path = cache_dir.join(key_b);
    let a_exists = a_path.exists();
    let b_exists = b_path.exists();
    if !a_exists || !b_exists {
        return Ok(json!({
            "key_a": key_a,
            "key_b": key_b,
            "comparable": false,
            "reason": "missing cache entry",
            "missing": {
                "key_a": !a_exists,
                "key_b": !b_exists
            }
        }));
    }
    let a_meta = load_meta(&a_path)?;
    let b_meta = load_meta(&b_path)?;
    let mut differences = Vec::new();
    for field in [
        "cache_key",
        "node_fingerprint",
        "node_definition_fingerprint",
        "declared_environment_fingerprint",
        "input_lineage_fingerprint",
        "adapter_id",
        "adapter_version",
        "produces_outputs_schema_version",
        "policy_fingerprint",
        "execution_contract_fingerprint",
        "backend_class",
        "cache_metadata_version",
        "source_run_id",
        "cache_source",
    ] {
        if a_meta.get(field) != b_meta.get(field) {
            differences.push(json!({
                "field": field,
                "a": a_meta.get(field).cloned().unwrap_or(Value::Null),
                "b": b_meta.get(field).cloned().unwrap_or(Value::Null),
            }));
        }
    }
    let a_manifest = load_manifest(&a_path)?;
    let b_manifest = load_manifest(&b_path)?;
    if a_manifest != b_manifest {
        differences.push(json!({
            "field": "manifest",
            "a": a_manifest,
            "b": b_manifest,
        }));
    }
    let a_valid = verify_cache_entry_cli(&a_path, key_a, "", "")?;
    let b_valid = verify_cache_entry_cli(&b_path, key_b, "", "")?;
    Ok(json!({
        "key_a": key_a,
        "key_b": key_b,
        "comparable": true,
        "valid": {
            "key_a": a_valid,
            "key_b": b_valid
        },
        "differences": differences
    }))
}

fn dir_size_bytes(path: &Path) -> Result<u64, ExitCode> {
    let mut total = 0u64;
    let mut entries: Vec<_> =
        fs::read_dir(path).map_err(|_| ExitCode::from(3))?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let p = entry.path();
        if p.is_dir() {
            total += dir_size_bytes(&p)?;
        } else {
            total += fs::metadata(&p).map_err(|_| ExitCode::from(3))?.len();
        }
    }
    Ok(total)
}

fn copy_dir_all(src: &Path, dst: &Path) -> std::io::Result<()> {
    fs::create_dir_all(dst)?;
    let mut entries: Vec<_> = fs::read_dir(src)?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|e| e.file_name());
    for entry in entries {
        let ty = entry.file_type()?;
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst_path)?;
        } else {
            fs::copy(entry.path(), dst_path)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        explain_cache_key, explain_run_node_cache_miss, pack_cache_entry, unpack_cache_entry,
    };
    use bijux_dag_artifacts::CacheIdentity;
    use bijux_dag_runtime::{cache_key_explanation, CacheEntryManifest, CacheKeyInput};
    use serde_json::json;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::process::ExitCode;

    fn manifest_json(
        cache_key: &str,
        node_id: &str,
        outputs: serde_json::Value,
    ) -> serde_json::Value {
        json!({
            "manifest_version": bijux_dag_runtime::CACHE_ENTRY_MANIFEST_VERSION,
            "cache_key": cache_key,
            "node_id": node_id,
            "outputs": outputs
        })
    }

    fn current_key_input() -> CacheKeyInput {
        CacheKeyInput {
            execution_fingerprint: "exec-current".to_string(),
            node_definition_fingerprint: "node-current".to_string(),
            declared_environment_fingerprint: "env-current".to_string(),
            input_lineage_fingerprint: "inputs-current".to_string(),
            adapter_id: "shell".to_string(),
            adapter_version: "1.0.0".to_string(),
            adapter_binary_sha256: None,
            output_schema_version: "schema/v1".to_string(),
            policy_fingerprint: "policy-current".to_string(),
            execution_contract_fingerprint: "exec-contract-current".to_string(),
            backend_class: "local".to_string(),
        }
    }

    fn current_identity() -> CacheIdentity {
        let key_input = current_key_input();
        CacheIdentity {
            cache_key: cache_key_explanation(&key_input).key,
            node_definition_fingerprint: key_input.node_definition_fingerprint,
            declared_environment_fingerprint: key_input.declared_environment_fingerprint,
            input_lineage_fingerprint: key_input.input_lineage_fingerprint,
            adapter_binary_sha256: key_input.adapter_binary_sha256,
            params_fingerprint: "params-current".to_string(),
            command_fingerprint: Some("command-current".to_string()),
            policy_fingerprint: key_input.policy_fingerprint,
            execution_contract_fingerprint: key_input.execution_contract_fingerprint,
            backend_class: key_input.backend_class,
        }
    }

    fn write_run_fixture(
        root: &Path,
        cache_dir: &Path,
        cache_mode: &str,
        cache_enabled: bool,
        cache_reason: Option<&str>,
        identity: &CacheIdentity,
    ) -> PathBuf {
        let run_dir = root.join("run");
        fs::create_dir_all(run_dir.join("nodes/node")).expect("mkdir nodes");
        fs::write(
            run_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "manifest_version":"run-manifest/v0.1",
                "run_id":"run-1",
                "created_unix_ms":1,
                "started_unix_ms":2,
                "finished_unix_ms":3,
                "graph_snapshot":"graph.snapshot.json",
                "status":"success",
                "spec":"bijux-dag/v0.1",
                "graph_fingerprint":"graph-1",
                "tool_version":"0.1.0",
                "jobs":1,
                "adapters":[],
                "outputs":[],
                "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
                "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
                "cache_mode": cache_mode,
                "cache_dir": cache_dir.display().to_string()
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            run_dir.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph":{
                    "spec":"bijux-dag/v0.1",
                    "meta":{"name":"cache-explain","owners":[],"tags":[]},
                    "inputs":{},
                    "nodes":[{
                        "id":"node",
                        "kind":"shell",
                        "inputs":[],
                        "outputs":[{"name":"value","path":"value.txt"}],
                        "params":{"argv":["/bin/sh","-c","printf '%s' ok > ../outputs/value.txt"]},
                        "cache":{"enabled":cache_enabled,"reason":cache_reason},
                        "effects":["filesystem"]
                    }],
                    "edges":[]
                },
                "graph_fingerprint":"graph-1"
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(
            run_dir.join("nodes/node/trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"node",
                "status":"success",
                "started_unix_ms":1,
                "finished_unix_ms":2,
                "attempt":1,
                "fingerprint":"exec-current",
                "adapter_id":"shell",
                "adapter_version":"1.0.0",
                "adapter_outputs_schema_version":"schema/v1",
                "cache_identity": identity
            }))
            .expect("trace"),
        )
        .expect("write trace");
        run_dir
    }

    fn cache_meta(
        key_input: &CacheKeyInput,
        cache_key: &str,
        params_fingerprint: &str,
        command_fingerprint: Option<&str>,
    ) -> serde_json::Value {
        json!({
            "cache_metadata_version": bijux_dag_runtime::CACHE_METADATA_VERSION,
            "cache_key": cache_key,
            "node_id": "node",
            "node_fingerprint": key_input.execution_fingerprint,
            "node_definition_fingerprint": key_input.node_definition_fingerprint,
            "declared_environment_fingerprint": key_input.declared_environment_fingerprint,
            "input_lineage_fingerprint": key_input.input_lineage_fingerprint,
            "params_fingerprint": params_fingerprint,
            "command_fingerprint": command_fingerprint,
            "adapter_id": key_input.adapter_id,
            "adapter_version": key_input.adapter_version,
            "produces_outputs_schema_version": key_input.output_schema_version,
            "policy_fingerprint": key_input.policy_fingerprint,
            "execution_contract_fingerprint": key_input.execution_contract_fingerprint,
            "backend_class": key_input.backend_class,
            "created_unix_ms": 1,
            "cache_source": "local",
            "schema_version": "v0.1"
        })
    }

    fn write_valid_cache_entry(
        cache_dir: &Path,
        key_input: &CacheKeyInput,
        params_fingerprint: &str,
        command_fingerprint: Option<&str>,
    ) -> String {
        let cache_key = cache_key_explanation(key_input).key;
        let entry = cache_dir.join(&cache_key);
        fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
        fs::write(
            entry.join("meta.json"),
            serde_json::to_vec_pretty(&cache_meta(
                key_input,
                &cache_key,
                params_fingerprint,
                command_fingerprint,
            ))
            .expect("meta"),
        )
        .expect("write meta");
        fs::write(
            entry.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest_json(
                &cache_key,
                "node",
                json!([{
                    "name":"value",
                    "path":"value.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "required": true
                }]),
            ))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(entry.join("outputs/value.txt"), b"ok").expect("payload");
        fs::write(
            entry.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{
                    "name":"value",
                    "path":"value.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "size_bytes":2,
                    "sha256": bijux_dag_artifacts::hash::sha256_hex(b"ok"),
                    "node_id":"node",
                    "node_fingerprint": key_input.execution_fingerprint
                }]
            }))
            .expect("index"),
        )
        .expect("write index");
        cache_key
    }

    #[test]
    fn explain_cache_key_reports_taxonomy_and_components() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let key_input = CacheKeyInput {
            execution_fingerprint: "exec-a".to_string(),
            node_definition_fingerprint: "node-a".to_string(),
            declared_environment_fingerprint: "env-a".to_string(),
            input_lineage_fingerprint: "inputs-a".to_string(),
            adapter_id: "shell".to_string(),
            adapter_version: "1.0.0".to_string(),
            adapter_binary_sha256: None,
            output_schema_version: "schema/v1".to_string(),
            policy_fingerprint: "policy-a".to_string(),
            execution_contract_fingerprint: "exec-contract-a".to_string(),
            backend_class: "local".to_string(),
        };
        let cache_key = cache_key_explanation(&key_input).key;
        let entry = cache_dir.join(&cache_key);
        fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
        fs::write(
            entry.join("meta.json"),
            serde_json::to_vec_pretty(&json!({
                "cache_key": cache_key,
                "cache_metadata_version": bijux_dag_runtime::CACHE_METADATA_VERSION,
                "node_fingerprint": key_input.execution_fingerprint,
                "node_definition_fingerprint": key_input.node_definition_fingerprint,
                "declared_environment_fingerprint": key_input.declared_environment_fingerprint,
                "input_lineage_fingerprint": key_input.input_lineage_fingerprint,
                "adapter_id": key_input.adapter_id,
                "adapter_version": key_input.adapter_version,
                "adapter_binary_sha256": key_input.adapter_binary_sha256,
                "produces_outputs_schema_version": key_input.output_schema_version,
                "policy_fingerprint": key_input.policy_fingerprint,
                "execution_contract_fingerprint": key_input.execution_contract_fingerprint,
                "backend_class": key_input.backend_class
            }))
            .expect("meta"),
        )
        .expect("write meta");
        fs::write(
            entry.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest_json(
                &cache_key,
                "n1",
                json!([{
                    "name":"report",
                    "path":"report.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "required": true
                }]),
            ))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(
            entry.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{"name":"report","path":"report.txt","kind":"file","media_type":"text/plain","size_bytes":7,"sha256":"deadbeef","node_id":"n1","node_fingerprint":"exec-a"}]
            }))
            .expect("index"),
        )
        .expect("write index");
        fs::write(entry.join("outputs/report.txt"), b"payload").expect("payload");

        let report =
            explain_cache_key(&cache_dir, &cache_key, "shell", "1.0.0").expect("explain cache key");
        assert_eq!(report["eligible"], false);
        assert!(report["taxonomy"].as_array().is_some_and(|items| !items.is_empty()));
        assert!(report["key_components"].is_object());
    }

    #[test]
    fn explain_run_node_cache_miss_reports_changed_params() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let mut prior = current_key_input();
        prior.node_definition_fingerprint = "node-before-params".to_string();
        write_valid_cache_entry(
            &cache_dir,
            &prior,
            "params-before",
            identity.command_fingerprint.as_deref(),
        );

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "changed_params");
    }

    #[test]
    fn explain_run_node_cache_miss_reports_changed_input_hashes() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let mut prior = current_key_input();
        prior.input_lineage_fingerprint = "inputs-before".to_string();
        write_valid_cache_entry(
            &cache_dir,
            &prior,
            &identity.params_fingerprint,
            identity.command_fingerprint.as_deref(),
        );

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "changed_input_hashes");
    }

    #[test]
    fn explain_run_node_cache_miss_reports_changed_command() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let mut prior = current_key_input();
        prior.node_definition_fingerprint = "node-before-command".to_string();
        write_valid_cache_entry(
            &cache_dir,
            &prior,
            &identity.params_fingerprint,
            Some("command-before"),
        );

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "changed_command");
    }

    #[test]
    fn explain_run_node_cache_miss_reports_changed_adapter_identity() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let mut prior = current_key_input();
        prior.adapter_version = "0.9.0".to_string();
        write_valid_cache_entry(
            &cache_dir,
            &prior,
            &identity.params_fingerprint,
            identity.command_fingerprint.as_deref(),
        );

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "changed_adapter_identity");
    }

    #[test]
    fn explain_run_node_cache_miss_reports_corrupt_entry() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let exact_key = write_valid_cache_entry(
            &cache_dir,
            &current_key_input(),
            &identity.params_fingerprint,
            identity.command_fingerprint.as_deref(),
        );
        fs::remove_file(cache_dir.join(&exact_key).join("manifest.json")).expect("remove manifest");

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "unsafe_reuse_refused");
        assert!(report["reasons"].as_array().is_some_and(|items| {
            items.iter().filter_map(|item| item.as_str()).any(|reason| reason.contains("manifest"))
        }));
    }

    #[test]
    fn explain_run_node_cache_miss_reports_missing_entry() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "ReadWrite", true, None, &identity);

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "missing_entry");
    }

    #[test]
    fn explain_run_node_cache_miss_reports_policy_bypass() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let identity = current_identity();
        let run_dir = write_run_fixture(tmp.path(), &cache_dir, "Off", true, None, &identity);

        let report = explain_run_node_cache_miss(&run_dir, "node", None).expect("explain");
        assert_eq!(report["outcome"], "miss");
        assert_eq!(report["taxonomy"][0], "policy_bypass");
    }

    #[test]
    fn cache_pack_unpack_preserves_metadata_and_rejects_corruption() {
        let tmp = tempfile::tempdir().expect("tmp");
        let entry = tmp.path().join("entry");
        let key_input = CacheKeyInput {
            execution_fingerprint: "exec-key".to_string(),
            node_definition_fingerprint: "node-key".to_string(),
            declared_environment_fingerprint: "env-key".to_string(),
            input_lineage_fingerprint: "inputs-key".to_string(),
            adapter_id: "shell".to_string(),
            adapter_version: "1.0.0".to_string(),
            adapter_binary_sha256: None,
            output_schema_version: "schema/v1".to_string(),
            policy_fingerprint: "policy-a".to_string(),
            execution_contract_fingerprint: "exec-contract-a".to_string(),
            backend_class: "local".to_string(),
        };
        let cache_key = cache_key_explanation(&key_input).key;
        fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
        fs::write(
            entry.join("meta.json"),
            serde_json::to_vec_pretty(&json!({
                "cache_key": cache_key,
                "cache_metadata_version": bijux_dag_runtime::CACHE_METADATA_VERSION,
                "node_fingerprint": key_input.execution_fingerprint,
                "node_definition_fingerprint": key_input.node_definition_fingerprint,
                "declared_environment_fingerprint": key_input.declared_environment_fingerprint,
                "input_lineage_fingerprint": key_input.input_lineage_fingerprint,
                "adapter_id": key_input.adapter_id,
                "adapter_version": key_input.adapter_version,
                "adapter_binary_sha256": key_input.adapter_binary_sha256,
                "produces_outputs_schema_version": key_input.output_schema_version,
                "policy_fingerprint": key_input.policy_fingerprint,
                "execution_contract_fingerprint": key_input.execution_contract_fingerprint,
                "backend_class": key_input.backend_class
            }))
            .expect("meta"),
        )
        .expect("write meta");
        fs::write(
            entry.join("manifest.json"),
            serde_json::to_vec_pretty(&manifest_json(
                &cache_key,
                "n1",
                json!([{
                    "name":"data",
                    "path":"data.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "required": true
                }]),
            ))
            .expect("manifest"),
        )
        .expect("write manifest");
        fs::write(entry.join("outputs/data.txt"), b"payload").expect("payload");
        fs::write(
            entry.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{
                    "name":"data",
                    "path":"data.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "size_bytes":7,
                    "sha256": bijux_dag_artifacts::hash::sha256_hex(b"payload"),
                    "node_id":"n1",
                    "node_fingerprint":"exec-key"
                }]
            }))
            .expect("index"),
        )
        .expect("write index");

        let pack = tmp.path().join("entry.tgz");
        pack_cache_entry(&entry, &pack).expect("pack entry");
        let unpack_dir = tmp.path().join("cache");
        unpack_cache_entry(&pack, &unpack_dir).expect("unpack entry");

        let unpacked_meta: serde_json::Value = serde_json::from_slice(
            &fs::read(unpack_dir.join(&cache_key).join("meta.json")).expect("read unpacked meta"),
        )
        .expect("parse unpacked meta");
        let unpacked_manifest: CacheEntryManifest = serde_json::from_slice(
            &fs::read(unpack_dir.join(&cache_key).join("manifest.json"))
                .expect("read unpacked manifest"),
        )
        .expect("parse unpacked manifest");
        assert_eq!(unpacked_meta["adapter_id"], "shell");
        assert_eq!(unpacked_meta["cache_source"], "pack");
        assert_eq!(unpacked_manifest.cache_key, cache_key);

        fs::write(&pack, b"corrupt-pack").expect("corrupt pack");
        let corrupt = unpack_cache_entry(&pack, &unpack_dir);
        assert!(matches!(corrupt, Err(code) if code == ExitCode::from(3)));
    }
}
