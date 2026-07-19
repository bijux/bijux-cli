use base64 as _;
use bijux_dag_app::{runs_history, runs_summary};
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

fn write_manifest(path: &std::path::Path, run_id: &str, status: &str) {
    fs::create_dir_all(path).expect("mkdir");
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id": run_id,
            "created_unix_ms": 1,
            "started_unix_ms": 1,
            "finished_unix_ms": 2,
            "graph_snapshot":"graph.snapshot.json",
            "status": status,
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"g",
            "tool_version":"0.1.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":0,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "run_metadata":{
              "submission_source":"manual",
              "trigger_source":"cli",
              "operator":"tester",
              "labels":[],
              "parent_run_id":"run-parent",
              "source_run_id":"run-import-parent"
            }
        }))
        .expect("manifest"),
    )
    .expect("write");
}

#[test]
fn run_history_rebuilds_from_raw_run_dirs_and_salvages_corrupt_entries() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-a"), "run-a", "success");
    fs::create_dir_all(root.join("run-b")).expect("mkdir");
    fs::write(root.join("run-b").join("manifest.json"), "{not-json").expect("corrupt");

    let history = runs_history(&root).expect("history");
    let rows = history["runs"].as_array().expect("rows");
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().any(|row| row["run_id"] == "run-a"));
}

#[test]
fn run_analytics_queries_do_not_mutate_authoritative_run_records() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run_path = root.join("run-immut");
    write_manifest(&run_path, "run-immut", "success");
    let manifest = run_path.join("manifest.json");
    let before = fs::read_to_string(&manifest).expect("read before");

    let _ = runs_summary(&root).expect("summary");
    let _ = runs_history(&root).expect("history");

    let after = fs::read_to_string(&manifest).expect("read after");
    assert_eq!(before, after, "analytics must not mutate manifest content");
}
