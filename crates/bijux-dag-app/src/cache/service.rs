use crate::ExitCode;
use bijux_dag_artifacts::OutputsIndex;
use bijux_dag_runtime::{
    cache_entry_has_required_proof, cache_key_explanation, cache_metadata_version_supported,
    CacheKeyInput,
};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use serde_json::{json, Value};
use std::fs;
use std::path::Path;
use tar::{Archive, Builder};

const MAX_CACHE_ARCHIVE_TOTAL_BYTES: u64 = 8 * 1024 * 1024;
const MAX_CACHE_ARCHIVE_ENTRIES: usize = 10_000;

fn hash_bytes(bytes: &[u8]) -> String {
    bijux_dag_artifacts::hash::sha256_hex(bytes)
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
                let index_path = path.join("outputs").join("index.json");
                let meta_path = path.join("meta.json");
                if !index_path.exists() || !meta_path.exists() {
                    corrupt += 1;
                    corrupt_keys.push(key);
                    continue;
                }
                let data = fs::read_to_string(&index_path).map_err(|_| ExitCode::from(3))?;
                let index: OutputsIndex =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                for file in index.files {
                    let fpath = path.join("outputs").join(&file.path);
                    if !fpath.exists() {
                        corrupt += 1;
                        corrupt_keys.push(key.clone());
                        break;
                    }
                    let bytes = fs::read(&fpath).map_err(|_| ExitCode::from(3))?;
                    let sha = hash_bytes(&bytes);
                    if sha != file.sha256 {
                        corrupt += 1;
                        corrupt_keys.push(key.clone());
                        break;
                    }
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
    builder.append_dir_all(".", entry).map_err(|_| ExitCode::from(3))?;
    let enc = builder.into_inner().map_err(|_| ExitCode::from(3))?;
    enc.finish().map_err(|_| ExitCode::from(3))?;
    Ok(())
}

pub(crate) fn unpack_cache_entry(pack: &Path, cache_dir: &Path) -> Result<(), ExitCode> {
    let file = fs::File::open(pack).map_err(|_| ExitCode::from(3))?;
    let dec = GzDecoder::new(file);
    let mut archive = Archive::new(dec);
    let tmp = tempfile::tempdir().map_err(|_| ExitCode::from(3))?;
    unpack_cache_archive_bounded(&mut archive, tmp.path())?;
    let meta_path = tmp.path().join("meta.json");
    if !meta_path.exists() {
        return Err(ExitCode::from(3));
    }
    let mut meta: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    let key =
        meta.get("node_fingerprint").and_then(|v| v.as_str()).ok_or(ExitCode::from(3))?.to_string();
    let adapter_id = meta.get("adapter_id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let adapter_version =
        meta.get("adapter_version").and_then(|v| v.as_str()).unwrap_or("").to_string();
    if !verify_cache_entry_cli(tmp.path(), &key, &adapter_id, &adapter_version)? {
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
    copy_dir_all(tmp.path(), &dst).map_err(|_| ExitCode::from(3))?;
    Ok(())
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

pub(crate) fn verify_cache_entry_cli(
    entry: &Path,
    expected_key: &str,
    adapter_id: &str,
    adapter_version: &str,
) -> Result<bool, ExitCode> {
    let index_path = entry.join("outputs").join("index.json");
    let meta_path = entry.join("meta.json");
    if !index_path.exists() || !meta_path.exists() {
        return Ok(false);
    }
    let meta: Value =
        serde_json::from_str(&fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?)
            .map_err(|_| ExitCode::from(3))?;
    if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(expected_key) {
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
    for file in index.files {
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
    if !meta_path.exists() {
        reasons.push("missing meta.json".to_string());
        taxonomy.push("artifact_missing".to_string());
    }
    if !index_path.exists() {
        reasons.push("missing outputs/index.json".to_string());
        taxonomy.push("artifact_missing".to_string());
    }
    let mut meta = Value::Null;
    let mut key_components = Value::Null;
    if meta_path.exists() {
        meta = serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        let key_input = CacheKeyInput {
            node_fingerprint: meta
                .get("node_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            adapter_id: meta
                .get("adapter_id")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            adapter_version: meta
                .get("adapter_version")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            output_schema_version: meta
                .get("produces_outputs_schema_version")
                .or_else(|| meta.get("output_schema_version"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            policy_fingerprint: meta
                .get("policy_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            config_fingerprint: meta
                .get("config_fingerprint")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
            backend_class: meta
                .get("backend_class")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string(),
        };
        let explanation = cache_key_explanation(&key_input);
        key_components = json!({
            "graph_fingerprint_component": Value::Null,
            "node_fingerprint_component": meta.get("node_fingerprint").cloned().unwrap_or(Value::Null),
            "inputs_index_component": meta.get("config_fingerprint").cloned().unwrap_or(Value::Null),
            "adapter_component": {
                "adapter_id": meta.get("adapter_id").cloned().unwrap_or(Value::Null),
                "adapter_version": meta.get("adapter_version").cloned().unwrap_or(Value::Null),
                "output_schema_version": meta.get("produces_outputs_schema_version")
                    .or_else(|| meta.get("output_schema_version"))
                    .cloned()
                    .unwrap_or(Value::Null),
            },
            "policy_component": meta.get("policy_fingerprint").cloned().unwrap_or(Value::Null),
            "env_component": {
                "backend_class": meta.get("backend_class").cloned().unwrap_or(Value::Null),
                "note": "runtime env fingerprint is not currently persisted in cache meta"
            },
            "computed_cache_key": explanation.key,
            "intentional_inputs": explanation.intentional_inputs,
        });
        if meta.get("cache_key").and_then(|v| v.as_str()) != Some(key) {
            reasons.push("cache key does not match requested key".to_string());
            taxonomy.push("hash_mismatch".to_string());
        }
        if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(key) {
            reasons.push("node_fingerprint mismatch".to_string());
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
        "reasons": reasons,
        "taxonomy": taxonomy,
        "key_components": key_components,
        "proof_verified": proof_valid
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
        "node_fingerprint",
        "adapter_id",
        "adapter_version",
        "output_schema_version",
        "policy_fingerprint",
        "config_fingerprint",
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
    use super::{explain_cache_key, pack_cache_entry, unpack_cache_entry};
    use serde_json::json;
    use std::fs;
    use std::process::ExitCode;

    #[test]
    fn explain_cache_key_reports_taxonomy_and_components() {
        let tmp = tempfile::tempdir().expect("tmp");
        let cache_dir = tmp.path().join("cache");
        let entry = cache_dir.join("key-a");
        fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
        fs::write(
            entry.join("meta.json"),
            serde_json::to_vec_pretty(&json!({
                "cache_key":"key-a",
                "cache_metadata_version":"cache-meta/v0.1",
                "node_fingerprint":"node-a",
                "adapter_id":"shell",
                "adapter_version":"1.0.0",
                "produces_outputs_schema_version":"schema/v1",
                "policy_fingerprint":"policy-a",
                "config_fingerprint":"config-a",
                "backend_class":"local"
            }))
            .expect("meta"),
        )
        .expect("write meta");
        fs::write(
            entry.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{"name":"report","path":"report.txt","kind":"file","media_type":"text/plain","sha256":"deadbeef","node_id":"n1","node_fingerprint":"node-a"}]
            }))
            .expect("index"),
        )
        .expect("write index");
        fs::write(entry.join("outputs/report.txt"), b"payload").expect("payload");

        let report =
            explain_cache_key(&cache_dir, "key-a", "shell", "1.0.0").expect("explain cache key");
        assert_eq!(report["eligible"], false);
        assert!(report["taxonomy"].as_array().is_some_and(|items| !items.is_empty()));
        assert!(report["key_components"].is_object());
    }

    #[test]
    fn cache_pack_unpack_preserves_metadata_and_rejects_corruption() {
        let tmp = tempfile::tempdir().expect("tmp");
        let entry = tmp.path().join("entry");
        fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
        fs::write(
            entry.join("meta.json"),
            serde_json::to_vec_pretty(&json!({
                "cache_key":"cache-key",
                "cache_metadata_version":"cache-meta/v0.1",
                "node_fingerprint":"node-key",
                "adapter_id":"shell",
                "adapter_version":"1.0.0",
                "policy_fingerprint":"policy-a",
                "config_fingerprint":"config-a",
                "backend_class":"local"
            }))
            .expect("meta"),
        )
        .expect("write meta");
        fs::write(entry.join("outputs/data.txt"), b"payload").expect("payload");
        fs::write(
            entry.join("outputs/index.json"),
            serde_json::to_vec_pretty(&json!({
                "files":[{
                    "name":"data",
                    "path":"data.txt",
                    "kind":"file",
                    "media_type":"text/plain",
                    "sha256": bijux_dag_artifacts::hash::sha256_hex(b"payload"),
                    "node_id":"n1",
                    "node_fingerprint":"node-key"
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
            &fs::read(unpack_dir.join("node-key").join("meta.json")).expect("read unpacked meta"),
        )
        .expect("parse unpacked meta");
        assert_eq!(unpacked_meta["adapter_id"], "shell");
        assert_eq!(unpacked_meta["cache_source"], "pack");

        fs::write(&pack, b"corrupt-pack").expect("corrupt pack");
        let corrupt = unpack_cache_entry(&pack, &unpack_dir);
        assert!(matches!(corrupt, Err(code) if code == ExitCode::from(3)));
    }
}
