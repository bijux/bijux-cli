use base64 as _;
use bijux_dag_app::{explain_failure, format_inspect_human, inspect_summary};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime::{build_plan, RuntimeConfig};
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use std::fs;
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

#[test]
fn plan_output_shape_snapshot_is_stable() {
    let graph = support::graph_chain();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let rendered = serde_json::to_string_pretty(&plan).expect("serialize plan");
    assert!(rendered.contains("\"nodes\""));
    assert!(rendered.contains("\"dep_map\""));
}

#[test]
fn explain_output_shape_snapshot_is_stable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("nodes/fail")).expect("mkdir");
    fs::write(
        run.join("nodes/fail/trace.json"),
        serde_json::to_vec_pretty(&json!({
            "status":"failed",
            "failure":{
                "kind":"Policy",
                "code":"POLICY_DENIED",
                "message":"clock denied"
            }
        }))
        .expect("trace"),
    )
    .expect("write");
    let explained = explain_failure(&run).expect("explain failure");
    let rendered = serde_json::to_string_pretty(&explained).expect("json");
    assert!(rendered.contains("root_failure"));
    assert!(rendered.contains("root_failure_class"));
    assert!(rendered.contains("root_failure_message"));
    assert!(rendered.contains("failed_nodes"));
    assert!(rendered.contains("failure_classes"));
    assert!(rendered.contains("propagated_failures"));
    assert!(rendered.contains("downstream_affected_groups"));
}

#[test]
fn inspect_output_shape_snapshot_is_stable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("nodes/a")).expect("mkdir");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id":"run-1","status":"success","graph_fingerprint":"g",
            "run_dir_format":"run-dir/v0.1","started_unix_ms":1,"finished_unix_ms":2
        }))
        .expect("manifest"),
    )
    .expect("write");
    fs::write(
        run.join("snapshot.json"),
        serde_json::to_vec_pretty(&json!({"graph":{"nodes":[],"edges":[]}})).expect("snapshot"),
    )
    .expect("write");
    fs::write(
        run.join("outputs.index.json"),
        serde_json::to_vec_pretty(&json!({"files":[]})).expect("outputs"),
    )
    .expect("write");
    fs::write(
        run.join("nodes/a/trace.json"),
        serde_json::to_vec_pretty(&json!({
            "status":"failed",
            "failure":{
                "kind":"Execution",
                "code":"EXEC_FAIL",
                "message":"command exited"
            }
        }))
        .expect("trace"),
    )
    .expect("write");
    let summary = inspect_summary(&run).expect("summary");
    let text = format_inspect_human(&summary);
    assert_eq!(summary["failure_classes"], json!(["execution"]));
    assert!(text.contains("run_id"));
    assert!(text.contains("[execution]"));
    assert!(text.contains("origin"));
}
