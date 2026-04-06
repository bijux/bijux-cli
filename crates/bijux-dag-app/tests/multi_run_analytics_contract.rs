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

use bijux_dag_app::{runs_compare, runs_failures, runs_flakes, runs_summary, runs_trend};
use serde_json::json;
use std::fs;
use std::path::Path;

fn write_run(
    base: &Path,
    run_id: &str,
    status: &str,
    graph_fp: &str,
    attempt: u64,
    failure_kind: Option<&str>,
) {
    let run = base.join(run_id);
    fs::create_dir_all(run.join("nodes").join("n1")).expect("mkdir node");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": run_id,
            "status": status,
            "graph_fingerprint": graph_fp,
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1200u64,
            "node_counts": {"success": 1, "failed": if status == "failed" { 1 } else { 0 }}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run.join("snapshot.json"),
        serde_json::to_vec_pretty(&json!({"graph":{"nodes":[{"id":"n1"}],"edges":[]}}))
            .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run.join("outputs.index.json"),
        serde_json::to_vec_pretty(&json!({"files":[{"path":"a"}]})).expect("outputs"),
    )
    .expect("write outputs");
    let trace = if status == "failed" {
        json!({
            "status": "failed",
            "attempt": attempt,
            "started_unix_ms": 1010u64,
            "finished_unix_ms": 1190u64,
            "failure": {"kind": failure_kind.unwrap_or("unknown")}
        })
    } else {
        json!({
            "status": "success",
            "attempt": attempt,
            "started_unix_ms": 1010u64,
            "finished_unix_ms": 1100u64,
            "cache_hit": true
        })
    };
    fs::write(
        run.join("nodes").join("n1").join("trace.json"),
        serde_json::to_vec_pretty(&trace).expect("trace"),
    )
    .expect("write trace");
}

#[test]
fn analytics_commands_return_expected_aggregates() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(tmp.path(), "run-a", "success", "g1", 1, None);
    write_run(tmp.path(), "run-b", "failed", "g1", 2, Some("timeout"));
    write_run(tmp.path(), "run-c", "success", "g2", 1, None);

    let summary = runs_summary(tmp.path()).expect("summary");
    assert_eq!(summary["runs"], 3);
    assert_eq!(summary["reports"]["cache_usefulness"]["total_cache_hits"], 2);
    assert_eq!(summary["reports"]["replay_equivalence"]["replay_equivalent_runs"], 2);

    let compare = runs_compare(tmp.path(), "run-a", "run-b").expect("compare");
    assert_eq!(compare["run_a"], "run-a");
    assert_eq!(compare["run_b"], "run-b");

    let trend = runs_trend(tmp.path()).expect("trend");
    assert_eq!(trend["series"].as_array().expect("series").len(), 3);

    let failures = runs_failures(tmp.path()).expect("failures");
    assert_eq!(failures["failure_distribution"]["timeout"], 1);

    let flakes = runs_flakes(tmp.path()).expect("flakes");
    assert_eq!(flakes["flakes"].as_array().expect("flakes").len(), 1);
}

#[test]
fn analytics_tolerate_incomplete_or_corrupt_history() {
    let tmp = tempfile::tempdir().expect("tmp");
    fs::create_dir_all(tmp.path().join("run-x")).expect("mkdir");
    fs::write(tmp.path().join("run-x").join("manifest.json"), "{bad").expect("manifest");
    fs::create_dir_all(tmp.path().join("run-x").join("nodes").join("n1")).expect("nodes");
    fs::write(tmp.path().join("run-x").join("nodes").join("n1").join("trace.json"), "{bad")
        .expect("trace");

    let summary = runs_summary(tmp.path()).expect("summary");
    assert_eq!(summary["runs"], 1);
    let trend = runs_trend(tmp.path()).expect("trend");
    assert_eq!(trend["series"].as_array().expect("series").len(), 1);
}

#[test]
fn analytics_do_not_mutate_authoritative_run_files() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(tmp.path(), "run-1", "success", "g1", 1, None);
    let manifest_path = tmp.path().join("run-1").join("manifest.json");
    let before = fs::read_to_string(&manifest_path).expect("before");

    let _ = runs_summary(tmp.path()).expect("summary");
    let _ = runs_trend(tmp.path()).expect("trend");
    let _ = runs_failures(tmp.path()).expect("failures");
    let _ = runs_flakes(tmp.path()).expect("flakes");

    let after = fs::read_to_string(&manifest_path).expect("after");
    assert_eq!(before, after);
}
