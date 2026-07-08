use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{dag_command, dag_run};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn write_cache_entry(base: &std::path::Path, key: &str, valid: bool) {
    let entry = base.join(key);
    fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
    let content = if valid { b"ok".to_vec() } else { b"bad".to_vec() };
    fs::write(entry.join("outputs/out.txt"), &content).expect("write output");
    let index = json!({
        "files": [{
            "name": "out",
            "path": "out.txt",
            "kind": "file",
            "media_type": "text/plain",
            "size_bytes": 2,
            "sha256": if valid { sha256_hex(b"ok") } else { sha256_hex(b"other") },
            "node_id": "n",
            "node_fingerprint": key
        }]
    });
    fs::write(
        entry.join("outputs/index.json"),
        serde_json::to_vec_pretty(&index).expect("index json"),
    )
    .expect("write index");
    let meta = json!({"node_fingerprint": key, "adapter_id": "shell", "adapter_version": "1"});
    fs::write(entry.join("meta.json"), serde_json::to_vec_pretty(&meta).expect("meta json"))
        .expect("write meta");
}

#[test]
fn cache_explain_stats_and_prune_simulate_cover_valid_and_invalid_entries() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_cache_entry(tmp.path(), "key-valid", true);
    write_cache_entry(tmp.path(), "key-invalid", false);

    let cmd = dag_command();
    let explain = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "cache",
            "explain",
            "--cache-dir",
            tmp.path().to_string_lossy().as_ref(),
            "--key",
            "key-valid",
        ])
        .expect("parse explain");
    assert!(dag_run(&explain).is_ok());

    let stats = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "cache",
            "stats",
            "--cache-dir",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse stats");
    assert!(dag_run(&stats).is_ok());

    let prune = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "cache",
            "prune-simulate",
            "--cache-dir",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse prune sim");
    assert!(dag_run(&prune).is_ok());

    let diff = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "cache",
            "diff",
            "--cache-dir",
            tmp.path().to_string_lossy().as_ref(),
            "--key-a",
            "key-valid",
            "--key-b",
            "key-invalid",
        ])
        .expect("parse diff");
    assert!(dag_run(&diff).is_ok());

    let verify = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "cache",
            "verify",
            "--cache-dir",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse verify");
    assert!(dag_run(&verify).is_err());
}

#[test]
fn cache_corruption_fixtures_and_warm_cold_expectations_exist() {
    let fixture_root =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").join("evidence/cache");
    for rel in [
        "corrupt/missing_meta.json",
        "corrupt/hash_mismatch.json",
        "corrupt/missing_manifest.json",
        "corrupt/unsupported_metadata_version.json",
        "corrupt/truncated_meta.json",
        "corrupt/missing_outputs_proof.json",
    ] {
        assert!(fixture_root.join(rel).exists(), "missing fixture: {}", rel);
    }

    let warm_cold =
        fs::read_to_string(fixture_root.join("scenarios/warm_cold.json")).expect("read warm_cold");
    let parsed: serde_json::Value = serde_json::from_str(&warm_cold).expect("parse warm_cold");
    let expectations = parsed["expectations"].as_array().expect("expectation list");
    assert!(expectations.iter().any(|v| v == "warm_and_cold_outputs_semantically_equal"));
}
