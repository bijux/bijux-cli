use bijux_dag_artifacts::index::ArtifactId;
use bijux_dag_artifacts::platform::explain_lineage_safe_gc;
use bijux_dag_artifacts::retention::RetentionPolicy;
use bijux_dag_artifacts::{
    build_cleanup_plan, verify_run_dir, write_json_atomic_durable, VerificationMode,
};
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::fs;
use tempfile as _;
use thiserror as _;

#[test]
fn half_valid_run_dir_is_never_reported_as_valid() {
    let dir = tempfile::tempdir().expect("tmp");
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-1"
        }))
        .expect("manifest"),
    )
    .expect("write");

    let report = verify_run_dir(dir.path(), VerificationMode::Standard).expect("verify");
    assert!(!report.valid);
    assert!(
        report.anomalies.iter().any(|a| a.contains("outputs.index"))
            || report.anomalies.iter().any(|a| a.contains("trace"))
    );
}

#[test]
fn atomic_durable_write_replaces_previous_json_payload() {
    let dir = tempfile::tempdir().expect("tmp");
    let target = dir.path().join("manifest.json");

    write_json_atomic_durable(&target, &json!({"version":1,"status":"old"})).expect("first write");
    write_json_atomic_durable(&target, &json!({"version":2,"status":"new"})).expect("second write");

    let current: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&target).expect("read")).expect("json");
    assert_eq!(current["version"], 2);
    assert_eq!(current["status"], "new");
}

#[test]
fn gc_explain_and_cleanup_plan_are_dry_run_safe_and_retention_aligned() {
    let referenced = vec![ArtifactId("extract:data.csv".to_string())];
    let all =
        vec![ArtifactId("extract:data.csv".to_string()), ArtifactId("train:model.bin".to_string())];
    let explain = explain_lineage_safe_gc(&referenced, &all, "lineage-1");
    assert_eq!(explain.lineage_snapshot_id, "lineage-1");
    assert!(explain
        .entries
        .iter()
        .any(|e| e.artifact_id.0 == "train:model.bin" && e.action == "collect"));

    let policy = RetentionPolicy::default();
    let retain_prefixes = policy.retain_prefixes();
    let entries =
        vec!["run-2026-03-01".to_string(), "cache-abc".to_string(), "scratch-temp".to_string()];
    let plan = build_cleanup_plan(&entries, &retain_prefixes);
    assert!(plan.retained.iter().any(|e| e.starts_with("run-")));
    assert!(plan.retained.iter().any(|e| e.starts_with("cache-")));
    assert!(plan.prunable.iter().any(|e| e == "scratch-temp"));
}
