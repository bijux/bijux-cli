use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
    execution_fp: &str,
    attempt: u64,
    failure_kind: Option<&str>,
    selected_nodes: &[&str],
    graph_inputs: serde_json::Value,
    output_sha256: &str,
) {
    let run = base.join(run_id);
    fs::create_dir_all(run.join("nodes").join("n1").join("outputs")).expect("mkdir node");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": run_id,
            "status": status,
            "graph_fingerprint": graph_fp,
            "execution_fingerprint": execution_fp,
            "started_unix_ms": 1000u64,
            "finished_unix_ms": 1200u64,
            "node_counts": {"success": 1, "failed": if status == "failed" { 1 } else { 0 }},
            "run_metadata": {"graph_inputs": graph_inputs}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(
            &json!({"graph":{"nodes":[{"id":"n1"}],"edges":[]},"graph_fingerprint": graph_fp}),
        )
        .expect("snapshot"),
    )
    .expect("write snapshot");
    fs::write(
        run.join("run.snapshot.json"),
        serde_json::to_vec_pretty(&json!({"selected_nodes": selected_nodes}))
            .expect("run snapshot"),
    )
    .expect("write run snapshot");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs");
    fs::write(
        run.join("outputs").join("index.json"),
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
    fs::write(
        run.join("nodes").join("n1").join("outputs").join("index.json"),
        serde_json::to_vec_pretty(&json!({
            "files": [{
                "name": "report.txt",
                "path": "report.txt",
                "kind": "file",
                "media_type": "text/plain",
                "size_bytes": 1,
                "sha256": output_sha256,
                "node_id": "n1",
                "node_fingerprint": "node-fp"
            }]
        }))
        .expect("output index"),
    )
    .expect("write output index");
}

#[test]
fn analytics_commands_return_expected_aggregates() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(
        tmp.path(),
        "run-a",
        "success",
        "g1",
        "exec-a",
        1,
        None,
        &["n1"],
        json!({"seed": 1}),
        "sha-a",
    );
    write_run(
        tmp.path(),
        "run-b",
        "failed",
        "g1",
        "exec-b",
        2,
        Some("timeout"),
        &["n2"],
        json!({"seed": 2}),
        "sha-b",
    );
    write_run(
        tmp.path(),
        "run-c",
        "success",
        "g2",
        "exec-c",
        1,
        None,
        &["n1"],
        json!({"seed": 3}),
        "sha-c",
    );

    let summary = runs_summary(tmp.path()).expect("summary");
    assert_eq!(summary["runs"], 3);
    assert_eq!(summary["reports"]["cache_usefulness"]["total_cache_hits"], 2);
    assert_eq!(summary["reports"]["replay_equivalence"]["replay_equivalent_runs"], 2);

    let compare = runs_compare(tmp.path(), "run-a", "run-b").expect("compare");
    assert_eq!(compare["run_a"], "run-a");
    assert_eq!(compare["run_b"], "run-b");
    assert_eq!(compare["graph_fingerprint"]["equal"], true);
    assert_eq!(compare["execution_fingerprint"]["equal"], false);
    assert_eq!(compare["input_values"]["changed_inputs"], json!(["seed"]));
    assert_eq!(compare["selected_nodes"]["changed_nodes"], json!(["n1", "n2"]));
    assert_eq!(compare["node_statuses"]["changed_nodes"], json!(["n1"]));
    assert_eq!(compare["output_hashes"]["changed_outputs"], json!(["n1:report.txt"]));
    assert_eq!(compare["first_meaningful_divergence"]["dimension"], "execution_fingerprint");

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
    fs::write(tmp.path().join("run-x").join("run.snapshot.json"), "{bad").expect("run snapshot");
    fs::create_dir_all(tmp.path().join("run-x").join("nodes").join("n1")).expect("nodes");
    fs::write(tmp.path().join("run-x").join("nodes").join("n1").join("trace.json"), "{bad")
        .expect("trace");

    let summary = runs_summary(tmp.path()).expect("summary");
    assert_eq!(summary["runs"], 1);
    let compare = runs_compare(tmp.path(), "run-x", "run-x").expect("compare");
    assert_eq!(compare["execution_fingerprint"]["equal"], serde_json::Value::Null);
    assert_eq!(compare["selected_nodes"]["equal"], serde_json::Value::Null);
    let trend = runs_trend(tmp.path()).expect("trend");
    assert_eq!(trend["series"].as_array().expect("series").len(), 1);
}

#[test]
fn analytics_do_not_mutate_authoritative_run_files() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(
        tmp.path(),
        "run-1",
        "success",
        "g1",
        "exec-1",
        1,
        None,
        &["n1"],
        json!({"seed": 1}),
        "sha-1",
    );
    let manifest_path = tmp.path().join("run-1").join("manifest.json");
    let before = fs::read_to_string(&manifest_path).expect("before");

    let _ = runs_summary(tmp.path()).expect("summary");
    let _ = runs_compare(tmp.path(), "run-1", "run-1").expect("compare");
    let _ = runs_trend(tmp.path()).expect("trend");
    let _ = runs_failures(tmp.path()).expect("failures");
    let _ = runs_flakes(tmp.path()).expect("flakes");

    let after = fs::read_to_string(&manifest_path).expect("after");
    assert_eq!(before, after);
}
