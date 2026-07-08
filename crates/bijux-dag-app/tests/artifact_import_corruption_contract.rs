use base64 as _;
use bijux_dag_app::{dag_command, dag_run, inspect_artifact};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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

#[test]
fn import_verify_reports_corrupt_bundle_without_panicking() {
    let tmp = tempfile::tempdir().expect("tmp");
    let bundle = tmp.path().join("corrupt.bundle.json");
    fs::write(
        &bundle,
        serde_json::to_vec_pretty(&json!({
            "bundle_version":"export-bundle/v0.1",
            "export_mode":"with-files",
            "manifest": {"manifest_version": "run-manifest/v0.1"},
            "graph_snapshot": {"spec":"bijux-dag/v0.1","nodes":[],"edges":[]},
            "node_traces": {},
            "outputs": {}
        }))
        .expect("bundle"),
    )
    .expect("write");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "import",
            bundle.to_string_lossy().as_ref(),
            "--verify-only",
        ])
        .expect("parse import");
    let code = dag_run(&matches);
    assert!(code.is_err(), "corrupt bundle should not import successfully");
}

#[test]
fn corrupted_outputs_index_is_reported_as_error() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("outputs")).expect("mkdir");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-1",
            "created_unix_ms":1,
            "started_unix_ms":1,
            "finished_unix_ms":2,
            "graph_snapshot":"graph.snapshot.json",
            "status":"success",
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"g",
            "tool_version":"0.1.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
        }))
        .expect("manifest"),
    )
    .expect("write");
    fs::write(run.join("outputs/index.json"), "{not-json").expect("corrupt index");

    let inspected = inspect_artifact(&run, "extract:data.csv");
    assert!(inspected.is_err(), "corrupt outputs index must be rejected");
}
