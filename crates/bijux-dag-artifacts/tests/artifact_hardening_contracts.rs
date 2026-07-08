use bijux_dag_artifacts::{
    build_cleanup_plan, finalize_run_manifest, finalize_run_manifest_with_mode, verify_run_dir,
    write_incomplete_run_marker, write_json_atomic_durable, Manifest, RunFinalizationMode,
    RunOutputsIndex, VerificationMode,
};
use hex as _;
use serde as _;
use sha2 as _;
use std::fs;
use std::path::PathBuf;
use thiserror as _;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sample_manifest(run_id: &str) -> Manifest {
    Manifest {
        manifest_version: "run-manifest/v0.1".to_string(),
        run_id: run_id.to_string(),
        created_unix_ms: 1,
        started_unix_ms: 2,
        finished_unix_ms: 3,
        graph_snapshot: "graph.snapshot.json".to_string(),
        status: "success".to_string(),
        spec: "bijux-dag/v0.1".to_string(),
        graph_fingerprint: "fp".to_string(),
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        planner_fingerprint: None,
        execution_fingerprint: None,
        evidence_fingerprint: None,
        tool_version: "0.1.0".to_string(),
        jobs: 1,
        adapters: vec![],
        outputs: vec![],
        node_counts: bijux_dag_artifacts::NodeCounts {
            success: 1,
            failed: 0,
            skipped: 0,
            cached: 0,
            cancelled: 0,
        },
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: true,
            deny_env: true,
            deny_clock: true,
            clean_env: true,
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
fn manifest_and_atomic_write_contracts_hold() {
    let dir = tempfile::tempdir().unwrap();
    let manifest_path = dir.path().join("manifest.json");
    let payload = serde_json::to_value(sample_manifest("run-1")).unwrap();
    assert_eq!(payload["planner_contract_version"], "bijux-dag-planner/v1");
    write_json_atomic_durable(&manifest_path, &payload).unwrap();
    assert!(manifest_path.exists());
}

#[test]
fn incomplete_and_finalized_markers_are_written() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&sample_manifest("run-2")).unwrap(),
    )
    .unwrap();
    write_incomplete_run_marker(dir.path(), "interrupted").unwrap();
    assert!(dir.path().join(".run-incomplete.json").exists());
    finalize_run_manifest(dir.path()).unwrap();
    assert!(dir.path().join("manifest.finalized.json").exists());
    assert!(dir.path().join(".run-complete.json").exists());
}

#[test]
fn incomplete_finalization_preserves_incomplete_marker_without_completion_marker() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&sample_manifest("run-4")).unwrap(),
    )
    .unwrap();
    write_incomplete_run_marker(dir.path(), "run timed out").unwrap();
    finalize_run_manifest_with_mode(dir.path(), RunFinalizationMode::Incomplete).unwrap();
    assert!(dir.path().join("manifest.finalized.json").exists());
    assert!(dir.path().join(".run-incomplete.json").exists());
    assert!(!dir.path().join(".run-complete.json").exists());
}

#[test]
fn strict_and_standard_verification_behave_as_expected() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("trace")).unwrap();
    fs::write(
        dir.path().join("manifest.json"),
        serde_json::to_vec_pretty(&sample_manifest("run-3")).unwrap(),
    )
    .unwrap();
    fs::write(dir.path().join("outputs.index.json"), b"{\"files\":[]}").unwrap();

    let standard = verify_run_dir(dir.path(), VerificationMode::Standard).unwrap();
    assert!(standard.valid);

    let strict = verify_run_dir(dir.path(), VerificationMode::Strict).unwrap();
    assert!(!strict.valid);
}

#[test]
fn import_export_and_replay_artifact_payloads_validate() {
    let valid_outputs = serde_json::json!({"files":[]});
    let parsed: RunOutputsIndex = serde_json::from_value(valid_outputs).unwrap();
    assert!(parsed.files.is_empty());

    let invalid_raw = fs::read_to_string(
        workspace_root().join("evidence/fault/corrupt_runs/invalid_outputs_index.json"),
    )
    .unwrap();
    assert!(serde_json::from_str::<RunOutputsIndex>(&invalid_raw).is_err());
}

#[test]
fn corruption_fixtures_are_detected_and_cleanup_plan_is_bounded() {
    let corrupted = fs::read_to_string(
        workspace_root().join("evidence/fault/corrupt_runs/missing_manifest_version.json"),
    )
    .unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&corrupted).unwrap();
    assert!(parsed.get("manifest_version").is_none());

    let plan = build_cleanup_plan(
        &["run-1".to_string(), "scratch-file".to_string(), "cache-abc".to_string()],
        &["run-", "cache-"],
    );
    assert_eq!(plan.retained.len(), 2);
    assert_eq!(plan.prunable.len(), 1);
}
