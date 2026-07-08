use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn output_path_string(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn run_dag(args: &[&str], cwd: &Path) -> (i32, String, String) {
    support::run_dag_command(args, cwd)
}

fn run_json(args: &[&str], cwd: &Path) -> Value {
    let (code, stdout, stderr) = run_dag(args, cwd);
    assert!(code == 0, "command failed: args={args:?} code={code} stdout={stdout} stderr={stderr}");
    serde_json::from_str(&stdout).expect("parse json envelope")
}

fn write_timeline_run(root: &Path, run_id: &str) -> PathBuf {
    let run_dir = root.join(run_id);
    fs::create_dir_all(run_dir.join("nodes").join("worker")).expect("mkdir worker");
    fs::write(
        run_dir.join("observability.timeline.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "v0.1",
            "entries": [
                {
                    "unix_ms": 100u64,
                    "category": "run",
                    "label": "run_started",
                    "node_id": null,
                    "source_event": "run_started"
                },
                {
                    "unix_ms": 110u64,
                    "category": "ready",
                    "label": "node_ready",
                    "node_id": "worker",
                    "source_event": "node_ready"
                },
                {
                    "unix_ms": 130u64,
                    "category": "failure",
                    "label": "node_failed",
                    "node_id": "worker",
                    "status": "failed",
                    "reason": "execution_failed",
                    "source_event": "node_finished"
                },
                {
                    "unix_ms": 140u64,
                    "category": "run",
                    "label": "run_completed",
                    "node_id": null,
                    "source_event": "run_finished"
                }
            ]
        }))
        .expect("timeline"),
    )
    .expect("write timeline");
    run_dir
}

#[test]
fn runs_timeline_command_accepts_query_flags() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    write_timeline_run(temp.path(), "run-flags");

    let (code, _stdout, stderr) = run_dag(
        &[
            "runs",
            "timeline",
            "run-flags",
            "--root",
            &output_path_string(temp.path()),
            "--node",
            "worker",
            "--event",
            "node_failed",
            "--since-unix-ms",
            "120",
            "--until-unix-ms",
            "135",
            "--json",
        ],
        &root,
    );

    assert_eq!(code, 0, "timeline flags should parse and execute: {stderr}");
}

#[test]
fn runs_timeline_json_output_preserves_filters_and_matches() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    write_timeline_run(temp.path(), "run-json");

    let payload = run_json(
        &[
            "runs",
            "timeline",
            "run-json",
            "--root",
            &output_path_string(temp.path()),
            "--node",
            "worker",
            "--event",
            "node_failed",
            "--since-unix-ms",
            "120",
            "--until-unix-ms",
            "135",
            "--json",
        ],
        &root,
    );

    assert_eq!(payload["command"], "dag.runs.timeline");
    assert_eq!(payload["data"]["source"], "observability_timeline");
    assert_eq!(payload["data"]["filters"]["node"], "worker");
    assert_eq!(payload["data"]["filters"]["event"], "node_failed");
    assert_eq!(payload["data"]["filters"]["since_unix_ms"], 120);
    assert_eq!(payload["data"]["filters"]["until_unix_ms"], 135);
    assert_eq!(payload["data"]["total_event_count"], 4);
    assert_eq!(payload["data"]["matched_event_count"], 1);
    assert_eq!(payload["data"]["events"][0]["label"], "node_failed");
    assert_eq!(payload["data"]["events"][0]["reason"], "execution_failed");
    assert_eq!(payload["data"]["events"][0]["source_event"], "node_finished");
}

#[test]
fn runs_timeline_human_output_shows_timestamp_and_cause() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("tmp");
    write_timeline_run(temp.path(), "run-human");

    let (code, stdout, stderr) = run_dag(
        &[
            "runs",
            "timeline",
            "run-human",
            "--root",
            &output_path_string(temp.path()),
            "--node",
            "worker",
            "--event",
            "node_failed",
            "--since-unix-ms",
            "120",
            "--until-unix-ms",
            "135",
        ],
        &root,
    );

    assert_eq!(code, 0, "timeline command should succeed: {stderr}");
    assert!(stdout.contains("source: observability_timeline"));
    assert!(stdout.contains("matched: 1/4 events"));
    assert!(stdout
        .contains("filters: node=worker event=node_failed since_unix_ms=120 until_unix_ms=135"));
    assert!(stdout.contains("timestamp_unix_ms=130"));
    assert!(stdout.contains("cause=execution_failed"));
    assert!(stdout.contains("source_event=node_finished"));
}
