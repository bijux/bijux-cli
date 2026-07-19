use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_artifacts::{Manifest, NodeCounts, PolicyInfo, RunDir};
use bijux_dag_runtime::{
    validate_storage_relative_path, ArtifactStore, CacheStore, StorageHealthReport,
};
use std::sync::Arc;

fn sample_manifest(run_id: &str) -> Manifest {
    Manifest {
        manifest_version: "manifest/v1".to_string(),
        run_id: run_id.to_string(),
        created_unix_ms: 0,
        started_unix_ms: 0,
        finished_unix_ms: 0,
        graph_snapshot: "{}".to_string(),
        status: "success".to_string(),
        spec: "v0.1".to_string(),
        graph_fingerprint: "x".to_string(),
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        planner_fingerprint: None,
        execution_fingerprint: None,
        evidence_fingerprint: None,
        tool_version: "test".to_string(),
        jobs: 1,
        adapters: Vec::new(),
        outputs: Vec::new(),
        node_counts: NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
        policy: PolicyInfo {
            deny_network: false,
            deny_env: false,
            deny_clock: false,
            clean_env: false,
            container_image_reference_policy:
                bijux_dag_artifacts::ContainerImageReferencePolicy::RequireDigest,
        },
        cache_mode: None,
        cache_dir: None,
        run_timeout_ms: None,
        run_timeout_behavior: None,
        run_cancellation_cause: None,
        run_metadata: None,
        run_summary: None,
    }
}

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
    let run_dir = Arc::new(RunDir::create(temp.path()).unwrap());
    let store = ArtifactStore::with_std_fs(run_dir.clone());
    run_dir.write_manifest(&sample_manifest("run-1")).unwrap();
    store.write_atomic_json("test.json", br#"{"ok":true}"#).unwrap();
    let manifest = store.read_validated_run_manifest().unwrap();
    assert_eq!(manifest["run_id"], "run-1");
}

#[test]
fn cache_store_meta_validation_requires_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let cache = CacheStore::with_std_fs(temp.path().to_path_buf());
    cache
        .write_cache_meta_atomic(
            "entry-1",
            &serde_json::json!({"fingerprint":"abc","version":"v1"}),
        )
        .unwrap();
    let parsed = cache.read_validated_cache_meta("entry-1").unwrap();
    assert_eq!(parsed["fingerprint"], "abc");
}

#[test]
fn health_report_detects_missing_outputs_index() {
    let temp = tempfile::tempdir().unwrap();
    let run_dir = Arc::new(RunDir::create(temp.path()).unwrap());
    let store = ArtifactStore::with_std_fs(run_dir.clone());
    run_dir.write_manifest(&sample_manifest("run-2")).unwrap();
    let report: StorageHealthReport = store.verify_health().unwrap();
    assert!(!report.healthy);
    assert!(report.anomalies.iter().any(|a| a.contains("outputs.index")));
}
