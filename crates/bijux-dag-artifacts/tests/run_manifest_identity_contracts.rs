use bijux_dag_artifacts::{Manifest, RunMetadata, RunSummary};
use hex as _;
use serde as _;
use sha2 as _;
use std::sync::Arc;
use std::thread;
use tempfile as _;
use thiserror as _;

mod support;

#[test]
fn minimal_and_maximal_run_manifest_fixtures_parse() {
    let minimal: Manifest = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-artifacts/tests/fixtures/run_manifest_minimal.json",
    );
    let maximal: Manifest = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-artifacts/tests/fixtures/run_manifest_maximal.json",
    );

    assert_eq!(minimal.manifest_version, "run-manifest/v0.1");
    assert_eq!(maximal.manifest_version, "run-manifest/v0.1");
    assert!(maximal.run_metadata.is_some());
    assert!(maximal.run_summary.is_some());
}

#[test]
fn run_metadata_supports_parent_and_source_run_identity_links() {
    let metadata = RunMetadata {
        submission_source: "manual".to_string(),
        trigger_source: "cli".to_string(),
        operator: "tester".to_string(),
        labels: vec!["x".to_string()],
        parent_run_id: Some("run-parent".to_string()),
        source_run_id: Some("run-source".to_string()),
        graph_inputs: std::collections::BTreeMap::from([
            ("region".to_string(), serde_json::json!("eu-west-1")),
            ("attempts".to_string(), serde_json::json!(2)),
        ]),
    };
    let value = serde_json::to_value(metadata).expect("serialize metadata");
    assert_eq!(value["parent_run_id"], "run-parent");
    assert_eq!(value["source_run_id"], "run-source");
    assert_eq!(value["graph_inputs"]["region"], "eu-west-1");
    assert_eq!(value["graph_inputs"]["attempts"], 2);
}

#[test]
fn run_summary_shape_is_stable() {
    let summary = RunSummary {
        total_nodes: 4,
        success: 2,
        failed: 1,
        skipped: 0,
        cached: 0,
        cancelled: 1,
        promoted_outputs: Vec::new(),
    };
    let value = serde_json::to_value(summary).expect("serialize summary");
    for key in ["total_nodes", "success", "failed", "skipped", "cached", "cancelled"] {
        assert!(value.get(key).is_some(), "run_summary missing required key: {key}");
    }
    assert!(value.get("promoted_outputs").is_none());
}

#[test]
fn concurrent_run_dir_creation_with_unique_ids_is_race_safe() {
    let tmp = tempfile::tempdir().expect("tmp");
    let base = Arc::new(tmp.path().to_path_buf());
    let mut handles = Vec::new();
    for i in 0..16u32 {
        let base = Arc::clone(&base);
        handles.push(thread::spawn(move || {
            let run_id = format!("race-{i}");
            let run = bijux_dag_artifacts::RunDir::create_with_id(&*base, &run_id)
                .expect("create run dir");
            run.finalize().expect("finalize");
            run_id
        }));
    }
    let mut ids = std::collections::BTreeSet::new();
    for handle in handles {
        ids.insert(handle.join().expect("join"));
    }
    assert_eq!(ids.len(), 16);
}
