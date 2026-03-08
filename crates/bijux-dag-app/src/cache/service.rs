use crate::ExitCode;
use bijux_dag_artifacts::OutputsIndex;
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
    builder
        .append_dir_all(".", entry)
        .map_err(|_| ExitCode::from(3))?;
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
    let key = meta
        .get("node_fingerprint")
        .and_then(|v| v.as_str())
        .ok_or(ExitCode::from(3))?
        .to_string();
    let adapter_id = meta
        .get("adapter_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let adapter_version = meta
        .get("adapter_version")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if !verify_cache_entry_cli(tmp.path(), &key, &adapter_id, &adapter_version)? {
        return Err(ExitCode::from(3));
    }
    if let Some(obj) = meta.as_object_mut() {
        obj.insert(
            "cache_source".to_string(),
            Value::String("pack".to_string()),
        );
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
    if !entry.exists() {
        reasons.push("missing cache entry directory".to_string());
        return Ok(json!({
            "key": key,
            "eligible": false,
            "reasons": reasons
        }));
    }
    let meta_path = entry.join("meta.json");
    let index_path = entry.join("outputs").join("index.json");
    if !meta_path.exists() {
        reasons.push("missing meta.json".to_string());
    }
    if !index_path.exists() {
        reasons.push("missing outputs/index.json".to_string());
    }
    let mut meta = Value::Null;
    if meta_path.exists() {
        meta = serde_json::from_str::<Value>(
            &fs::read_to_string(&meta_path).map_err(|_| ExitCode::from(3))?,
        )
        .map_err(|_| ExitCode::from(3))?;
        if meta.get("node_fingerprint").and_then(|v| v.as_str()) != Some(key) {
            reasons.push("node_fingerprint mismatch".to_string());
        }
        if !expected_adapter_id.is_empty()
            && meta.get("adapter_id").and_then(|v| v.as_str()) != Some(expected_adapter_id)
        {
            reasons.push("adapter_id mismatch".to_string());
        }
        if !expected_adapter_version.is_empty()
            && meta.get("adapter_version").and_then(|v| v.as_str())
                != Some(expected_adapter_version)
        {
            reasons.push("adapter_version mismatch".to_string());
        }
    }
    let eligible = reasons.is_empty()
        && verify_cache_entry_cli(
            entry.as_path(),
            key,
            expected_adapter_id,
            expected_adapter_version,
        )?;
    if !eligible && reasons.is_empty() {
        reasons.push("output proof verification failed".to_string());
    }
    Ok(json!({
        "key": key,
        "eligible": eligible,
        "entry_dir": entry,
        "meta": meta,
        "reasons": reasons
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
    let mut entries: Vec<_> = fs::read_dir(path)
        .map_err(|_| ExitCode::from(3))?
        .filter_map(|e| e.ok())
        .collect();
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
