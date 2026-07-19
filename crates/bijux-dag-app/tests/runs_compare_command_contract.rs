use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
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
use std::path::Path;

fn write_run(
    root: &Path,
    run_id: &str,
    graph_fingerprint: &str,
    execution_fingerprint: &str,
    selected_nodes: &[&str],
    status: &str,
    output_sha256: &str,
) {
    let run_dir = root.join(run_id);
    fs::create_dir_all(run_dir.join("nodes").join("build").join("outputs")).expect("mkdir outputs");
    fs::create_dir_all(run_dir.join("outputs")).expect("mkdir run outputs");
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "run_id": run_id,
            "status": status,
            "graph_fingerprint": graph_fingerprint,
            "execution_fingerprint": execution_fingerprint,
            "started_unix_ms": 10u64,
            "finished_unix_ms": 20u64,
            "node_counts": {"success": 1, "failed": 0},
            "run_metadata": {"graph_inputs": {"seed": if run_id.ends_with('a') { 1 } else { 2 }}}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run_dir.join("graph.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "graph": {"nodes": [{"id": "build"}], "edges": []},
            "graph_fingerprint": graph_fingerprint
        }))
        .expect("graph snapshot"),
    )
    .expect("write graph snapshot");
    fs::write(
        run_dir.join("run.snapshot.json"),
        serde_json::to_vec_pretty(&json!({"selected_nodes": selected_nodes}))
            .expect("run snapshot"),
    )
    .expect("write run snapshot");
    fs::write(
        run_dir.join("outputs").join("index.json"),
        serde_json::to_vec_pretty(&json!({"files": [{"path": "report.txt"}]})).expect("outputs"),
    )
    .expect("write outputs");
    fs::write(
        run_dir.join("nodes").join("build").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id": "build",
            "status": status,
            "attempt": 1,
            "started_unix_ms": 11u64,
            "finished_unix_ms": 19u64
        }))
        .expect("trace"),
    )
    .expect("write trace");
    fs::write(
        run_dir.join("nodes").join("build").join("outputs").join("index.json"),
        serde_json::to_vec_pretty(&json!({
            "files": [{
                "name": "report.txt",
                "path": "report.txt",
                "kind": "file",
                "media_type": "text/plain",
                "size_bytes": 1,
                "sha256": output_sha256,
                "node_id": "build",
                "node_fingerprint": "node-fp"
            }]
        }))
        .expect("output index"),
    )
    .expect("write output index");
}

#[test]
fn runs_compare_command_accepts_exact_comparison_surface() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(tmp.path(), "run-a", "graph-1", "exec-a", &["build"], "success", "sha-a");
    write_run(tmp.path(), "run-b", "graph-1", "exec-b", &["publish"], "success", "sha-b");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "compare",
            "run-a",
            "run-b",
            "--root",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn runs_compare_command_keeps_conservative_success_for_corrupt_optional_evidence() {
    let tmp = tempfile::tempdir().expect("tmp");
    write_run(tmp.path(), "run-a", "graph-1", "exec-a", &["build"], "success", "sha-a");
    let corrupt = tmp.path().join("run-b");
    fs::create_dir_all(corrupt.join("nodes").join("build").join("outputs")).expect("mkdir corrupt");
    fs::create_dir_all(corrupt.join("outputs")).expect("mkdir corrupt outputs");
    fs::write(corrupt.join("manifest.json"), "{bad").expect("manifest");
    fs::write(corrupt.join("graph.snapshot.json"), "{bad").expect("graph");
    fs::write(corrupt.join("run.snapshot.json"), "{bad").expect("snapshot");
    fs::write(corrupt.join("outputs").join("index.json"), "{\"files\":[]}").expect("outputs");
    fs::write(corrupt.join("nodes").join("build").join("trace.json"), "{bad").expect("trace");
    fs::write(corrupt.join("nodes").join("build").join("outputs").join("index.json"), "{bad")
        .expect("output index");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "compare",
            "run-a",
            "run-b",
            "--root",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
