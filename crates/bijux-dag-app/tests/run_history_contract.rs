use base64 as _;
use bijux_dag_app::{dag_command, dag_run, explain_run_id};
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
fn runs_history_and_id_explain_are_json_capable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-1");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version": "run-manifest/v0.1",
            "run_id": "run-1",
            "created_unix_ms": 1,
            "started_unix_ms": 1,
            "finished_unix_ms": 2,
            "graph_snapshot": "graph.snapshot.json",
            "status": "success",
            "spec": "bijux-dag/v0.1",
            "graph_fingerprint": "g",
            "tool_version": "0.1.0",
            "jobs": 1,
            "adapters": [],
            "outputs": [],
            "node_counts": {"success":1,"failed":0,"skipped":0,"cached":0},
            "policy": {"deny_network": true, "deny_env": true, "deny_clock": true, "clean_env": true},
            "run_metadata": {
                "submission_source": "manual",
                "trigger_source": "cli",
                "operator": "tester",
                "labels": [],
                "parent_run_id": "run-0",
                "source_run_id": "run-import-0"
            }
        }))
        .expect("manifest"),
    )
    .expect("write");

    let history = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "history",
            "--root",
            root.to_string_lossy().as_ref(),
        ])
        .expect("parse history");
    assert_eq!(dag_run(&history).expect("history run"), std::process::ExitCode::SUCCESS);

    let explain = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "id-explain",
            "run-1",
            "--root",
            root.to_string_lossy().as_ref(),
        ])
        .expect("parse id-explain");
    assert_eq!(dag_run(&explain).expect("explain run"), std::process::ExitCode::SUCCESS);
}

#[test]
fn run_id_explain_output_contains_identity_and_ancestry_fields() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-1");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version": "run-manifest/v0.1",
            "run_id": "run-1",
            "created_unix_ms": 1,
            "started_unix_ms": 1,
            "finished_unix_ms": 2,
            "status": "success",
            "graph_snapshot": "snapshot.json",
            "spec": "bijux-dag/v0.1",
            "graph_fingerprint": "g",
            "tool_version": "0.1.0",
            "jobs": 1,
            "adapters": [],
            "outputs": [],
            "node_counts": {"success":1,"failed":0,"skipped":0,"cached":0},
            "policy": {"deny_network": true, "deny_env": true, "deny_clock": true, "clean_env": true},
            "run_metadata": {
                "submission_source": "manual",
                "trigger_source": "cli",
                "operator": "tester",
                "labels": [],
                "parent_run_id": "run-0",
                "source_run_id": "run-import-0"
            }
        }))
        .expect("manifest"),
    )
    .expect("write");

    let explained = explain_run_id(&root, "run-1").expect("id explain");
    for field in [
        "run_id",
        "run_dir",
        "exists",
        "manifest_exists",
        "parent_run_id",
        "source_run_id",
        "immutability_contract",
    ] {
        assert!(explained.get(field).is_some(), "id explain output missing field `{field}`");
    }
}
