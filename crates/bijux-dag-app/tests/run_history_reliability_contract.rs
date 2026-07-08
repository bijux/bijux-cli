use base64 as _;
use bijux_dag_app::{dag_command, dag_run, doctor_run, runs_history};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;

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
            "node_counts":{"success":1,"failed":1,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "run_metadata":{
              "submission_source":"manual",
              "trigger_source":"cli",
              "operator":"tester",
              "labels":[]
            }
        }))
        .expect("manifest"),
    )
    .expect("write");
    fs::write(path.join("snapshot.json"), "{}").expect("snapshot");
    fs::write(path.join("outputs.index.json"), "{\"files\":[]}").expect("outputs");
}

#[test]
fn run_history_traversal_is_deterministic_across_calls() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-c"), "run-c", "success");
    write_manifest(&root.join("run-a"), "run-a", "success");
    write_manifest(&root.join("run-b"), "run-b", "failed");

    let first = runs_history(&root).expect("history");
    let second = runs_history(&root).expect("history");
    assert_eq!(first, second, "history traversal must be deterministic");

    let ids = first["runs"]
        .as_array()
        .expect("rows")
        .iter()
        .map(|row| row["run_id"].as_str().unwrap_or_default().to_string())
        .collect::<Vec<_>>();
    assert_eq!(ids, vec!["run-a", "run-b", "run-c"]);
}

#[test]
fn missing_traces_referenced_by_manifest_are_reported_cleanly() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-missing-trace");
    write_manifest(&run, "run-missing-trace", "failed");
    fs::create_dir_all(run.join("nodes")).expect("nodes dir");

    let report = doctor_run(&run);
    let findings = report["findings"].as_array().expect("findings");
    assert!(
        findings.iter().filter_map(|v| v.as_str()).any(|v| v.contains("trace")),
        "doctor should report missing traces when manifest indicates executed nodes"
    );
}

#[test]
fn latest_alias_updates_do_not_corrupt_history_rows() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-1"), "run-1", "success");
    write_manifest(&root.join("run-2"), "run-2", "success");

    let latest = root.join("latest");
    fs::write(&latest, "run-2").expect("write alias file");
    let before = runs_history(&root).expect("history before");

    fs::write(&latest, "run-1").expect("update alias file");
    let after = runs_history(&root).expect("history after");

    assert_eq!(before, after, "latest alias changes must not mutate history");
}

#[test]
fn replay_creates_new_run_linked_to_source_ancestry() {
    let tmp = tempfile::tempdir().expect("tmp");
    let dag = tmp.path().join("graph.json");
    let out = tmp.path().join("runs");
    fs::create_dir_all(&out).expect("out");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"replay-ancestry","owners":[],"tags":[]},
          "nodes":[{"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}}],
          "edges":[]
        }"#,
    )
    .expect("dag");

    let run_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "run",
            dag.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
            "--run-id",
            "source-run",
        ])
        .expect("parse run");
    assert_eq!(dag_run(&run_matches).expect("run"), std::process::ExitCode::SUCCESS);

    let replay_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "replay",
            out.join("run-source-run").to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
            "--run-id",
            "replay-run",
        ])
        .expect("parse replay");
    assert_eq!(dag_run(&replay_matches).expect("replay"), std::process::ExitCode::SUCCESS);

    let replay_manifest: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(out.join("run-replay-run").join("manifest.json")).expect("manifest"),
    )
    .expect("json");
    let metadata = replay_manifest["run_metadata"].clone();
    assert_eq!(metadata["parent_run_id"], "source-run");
    assert_eq!(metadata["source_run_id"], "source-run");
}

#[test]
fn run_history_remains_stable_after_workspace_relocation() {
    let tmp = tempfile::tempdir().expect("tmp");
    let source_root = tmp.path().join("runs-src");
    write_manifest(&source_root.join("run-1"), "run-1", "success");
    write_manifest(&source_root.join("run-2"), "run-2", "failed");

    let before = runs_history(&source_root).expect("history before relocation");
    let relocated_root = tmp.path().join("runs-relocated");
    fs::rename(&source_root, &relocated_root).expect("relocate runs root");

    let after = runs_history(&relocated_root).expect("history after relocation");
    assert_eq!(before, after, "run history should be path-relocation stable");
}

#[test]
fn run_history_survives_partial_artifact_gc_without_panic() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-gc");
    write_manifest(&run, "run-gc", "success");

    fs::remove_file(run.join("outputs.index.json")).expect("remove outputs index");
    fs::remove_file(run.join("snapshot.json")).expect("remove snapshot");

    let history = runs_history(&root).expect("history should recover after artifact gc");
    let rows = history["runs"].as_array().expect("rows");
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0]["run_id"], "run-gc");
}
