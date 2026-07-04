use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{
    dag_command, dag_run, doctor_run, explain_failure, format_inspect_human, format_show_human,
    inspect_summary, list_runs, run_timeline, run_tree,
};
use serde_json::json;
use std::fs;

fn write_run_fixture(base: &std::path::Path, run_id: &str) -> std::path::PathBuf {
    let run = base.join(run_id);
    fs::create_dir_all(run.join("nodes").join("a")).expect("mkdir node a");
    fs::create_dir_all(run.join("nodes").join("b")).expect("mkdir node b");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": run_id,
            "status": "failed",
            "graph_fingerprint": "g1",
            "run_dir_format": "run-dir/v0.1",
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1100u64,
            "node_counts": {"success": 1, "failed": 1}
        }))
        .expect("manifest json"),
    )
    .expect("write manifest");
    fs::write(
        run.join("snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph": {
                "nodes": [{"id":"a"},{"id":"b"}],
                "edges": [{"from":{"node_id":"a"}, "to":{"node_id":"b"}}]
            }
        }))
        .expect("snapshot json"),
    )
    .expect("write snapshot");
    fs::write(
        run.join("outputs.index.json"),
        serde_json::to_vec_pretty(&json!({"files":[{"path":"x"}]})).expect("outputs index"),
    )
    .expect("write outputs index");
    fs::write(
        run.join("nodes").join("a").join("trace.json"),
        serde_json::to_vec_pretty(&json!({"status":"success","started_unix_ms":1001u64,"finished_unix_ms":1050u64,"attempt":1,"cache_hit":true})).expect("trace a"),
    )
    .expect("write trace a");
    fs::write(
        run.join("nodes").join("b").join("trace.json"),
        serde_json::to_vec_pretty(&json!({"status":"failed","started_unix_ms":1055u64,"finished_unix_ms":1099u64,"attempt":2})).expect("trace b"),
    )
    .expect("write trace b");
    run
}

#[test]
fn operator_summary_and_human_output_are_stable() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-1");
    let summary = inspect_summary(&run).expect("summary");
    assert_eq!(summary["retry_count"], 1);
    assert_eq!(summary["artifact_count"], 1);
    assert_eq!(summary["integrity_state"], "healthy");
    let text = format_inspect_human(&summary);
    assert!(text.contains("run_id: \"run-1\""));
    assert!(text.contains("status: \"failed\""));
    assert!(text.contains("integrity_state: \"healthy\""));
    let show_text = format_show_human(&summary);
    assert!(show_text.contains("timing_ms"));
}

#[test]
fn operator_tree_timeline_and_failure_explain_work_from_explicit_run_dir() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-2");
    let tree = run_tree(&run).expect("tree");
    assert_eq!(tree["nodes"].as_array().expect("nodes").len(), 2);
    let timeline = run_timeline(&run).expect("timeline");
    assert_eq!(timeline["events"].as_array().expect("events").len(), 2);
    let events = timeline["events"].as_array().expect("events");
    assert_eq!(events[0]["node_id"], "a");
    assert_eq!(events[0]["cache_hit"], true);
    assert_eq!(events[0]["event_kind"], "cache_hit");
    assert_eq!(events[1]["attempt"], 2);
    assert_eq!(events[1]["event_kind"], "retry");
    let explain = explain_failure(&run).expect("explain");
    assert_eq!(explain["root_failure"], "b");
}

#[test]
fn operator_commands_tolerate_partial_corruption() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = tmp.path().join("run-bad");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(run.join("manifest.json"), "{bad-json").expect("write bad manifest");
    let summary = inspect_summary(&run).expect("summary from bad manifest");
    assert_eq!(summary["artifact_count"], 0);
    assert_eq!(summary["integrity_state"], "incomplete");
    let doctor = doctor_run(&run);
    assert_eq!(doctor["status"], "corrupt");
}

#[test]
fn operator_inspection_distinguishes_corrupt_runs() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = tmp.path().join("run-corrupt");
    fs::create_dir_all(run.join("nodes").join("a")).expect("mkdir");
    fs::write(
        run.join("snapshot.json"),
        serde_json::to_vec_pretty(&json!({"graph":{"nodes":[],"edges":[]}})).unwrap(),
    )
    .expect("write snapshot");
    fs::write(
        run.join("outputs.index.json"),
        serde_json::to_vec_pretty(&json!({"files":[]})).unwrap(),
    )
    .expect("write outputs");
    fs::write(run.join("manifest.json"), "{bad-json").expect("write bad manifest");
    let summary = inspect_summary(&run).expect("summary");
    assert_eq!(summary["integrity_state"], "corrupt");
}

#[test]
fn operator_inspection_distinguishes_unsupported_runs() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-unsupported");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": "run-unsupported",
            "status": "failed",
            "graph_fingerprint": "g1",
            "run_dir_format": "run-dir/v9.9",
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1100u64
        }))
        .expect("manifest json"),
    )
    .expect("write manifest");
    let summary = inspect_summary(&run).expect("summary");
    assert_eq!(summary["integrity_state"], "unsupported");
}

#[test]
fn operator_inspection_supports_imported_runs() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-imported");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": "run-imported",
            "status": "failed",
            "graph_fingerprint": "g1",
            "run_dir_format": "run-dir/v0.1",
            "submission_source": "import",
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1100u64
        }))
        .expect("manifest json"),
    )
    .expect("write manifest");
    let summary = inspect_summary(&run).expect("summary");
    assert_eq!(summary["integrity_state"], "healthy");
    assert_eq!(summary["run_id"], "run-imported");
}

#[test]
fn operator_timing_summary_is_trace_coherent() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-time-coherent");
    let summary = inspect_summary(&run).expect("summary");
    let started = summary["timing_ms"]["started"].as_u64().unwrap();
    let finished = summary["timing_ms"]["finished"].as_u64().unwrap();
    assert!(finished >= started);
    let timeline = run_timeline(&run).expect("timeline");
    let events = timeline["events"].as_array().unwrap();
    let min_start = events.iter().filter_map(|e| e["started_unix_ms"].as_u64()).min().unwrap();
    let max_finish = events.iter().filter_map(|e| e["finished_unix_ms"].as_u64()).max().unwrap();
    assert!(min_start >= started);
    assert!(max_finish <= finished);
}

#[test]
fn run_list_reads_only_explicit_root() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let _ = write_run_fixture(tmp.path(), "run-a");
    let _ = write_run_fixture(tmp.path(), "run-b");
    let listed = list_runs(tmp.path()).expect("list runs");
    assert_eq!(listed, vec!["run-a".to_string(), "run-b".to_string()]);
}

#[test]
fn operator_human_output_remains_concise() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let run = write_run_fixture(tmp.path(), "run-concise");
    let summary = inspect_summary(&run).expect("summary");
    let inspect = format_inspect_human(&summary);
    let show = format_show_human(&summary);
    assert!(inspect.lines().count() <= 9);
    assert!(show.lines().count() <= 9);
}

#[test]
fn operator_cli_inspect_works_without_ambient_repo_state() {
    let tmp = tempfile::tempdir().expect("tmpdir");
    let _run = write_run_fixture(tmp.path(), "run-cli");
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "inspect",
            "run-cli",
            "--root",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse");
    assert!(dag_run(&matches).is_ok());
}
