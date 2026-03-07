use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use serde_json::Value;
use std::fs;
use std::path::Path;

fn load_fixture(name: &str) -> Value {
    let path = Path::new("tests/fixtures/battle_workflows").join(name);
    let raw = fs::read_to_string(&path).expect("battle workflow fixture should exist");
    serde_json::from_str(&raw).expect("battle workflow fixture should be valid json")
}

fn assert_shape(doc: &Value) {
    assert!(doc.get("scenario").and_then(Value::as_str).is_some());
    assert!(doc.get("graph").and_then(Value::as_str).is_some());
    assert!(doc.get("nodes").and_then(Value::as_u64).is_some());
    assert!(doc.get("focus").and_then(Value::as_array).is_some());
    assert!(doc.get("expectations").and_then(Value::as_object).is_some());
}

#[test]
fn battle_workflow_harness_covers_required_scenarios() {
    let required = [
        "medium_workflow.json",
        "failure_heavy_workflow.json",
        "artifact_heavy_workflow.json",
        "cache_invalidation_workflow.json",
        "replay_divergence_workflow.json",
        "scheduler_fairness_workflow.json",
        "import_export_workflow.json",
        "corruption_workflow.json",
        "operator_inspection_workflow.json",
        "large_dag_workflow.json",
        "resource_contention_workflow.json",
        "multi_root_workflow.json",
        "branch_join_workflow.json",
        "retry_storm_workflow.json",
        "timeout_workflow.json",
        "policy_violation_workflow.json",
        "secret_leakage_workflow.json",
        "operator_debugging_workflow.json",
    ];

    for scenario in required {
        let doc = load_fixture(scenario);
        assert_shape(&doc);
    }
}
