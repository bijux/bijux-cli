use bijux_dag_artifacts::RunDir;
use bijux_dag_runtime::{validate_storage_relative_path, ArtifactStore, CacheStore, StorageHealthReport};
use std::sync::Arc;

#[test]
fn storage_relative_path_validation_rejects_traversal_and_absolute_paths() {
    assert!(validate_storage_relative_path("ok/path.json").is_ok());
    assert!(validate_storage_relative_path("../escape").is_err());
    assert!(validate_storage_relative_path("/absolute").is_err());
    assert!(validate_storage_relative_path("a\\b").is_err());
}

#[test]
fn artifact_store_atomic_write_and_manifest_validation_work() {
    let temp = tempfile::tempdir().unwrap();
    let run_dir = Arc::new(RunDir::new(temp.path()).unwrap());
    let store = ArtifactStore::with_std_fs(run_dir.clone());
    run_dir
        .write_manifest(&serde_json::json!({"run_id": "run-1", "graph_fingerprint":"x"}))
        .unwrap();
    store
        .write_atomic_json("metadata/test.json", br#"{"ok":true}"#)
        .unwrap();
    let manifest = store.read_validated_run_manifest().unwrap();
    assert_eq!(manifest["run_id"], "run-1");
}

#[test]
fn cache_store_meta_validation_requires_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let cache = CacheStore::with_std_fs(temp.path().to_path_buf());
    cache
        .write_cache_meta_atomic("entry-1", &serde_json::json!({"fingerprint":"abc","version":"v1"}))
        .unwrap();
    let parsed = cache.read_validated_cache_meta("entry-1").unwrap();
    assert_eq!(parsed["fingerprint"], "abc");
}

#[test]
fn health_report_detects_missing_outputs_index() {
    let temp = tempfile::tempdir().unwrap();
    let run_dir = Arc::new(RunDir::new(temp.path()).unwrap());
    let store = ArtifactStore::with_std_fs(run_dir.clone());
    run_dir
        .write_manifest(&serde_json::json!({"run_id": "run-2"}))
        .unwrap();
    let report: StorageHealthReport = store.verify_health().unwrap();
    assert!(!report.healthy);
    assert!(report.anomalies.iter().any(|a| a.contains("outputs.index")));
}
