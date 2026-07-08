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
use std::fs;
use tar as _;
use tempfile as _;
use thiserror as _;

#[test]
fn hash_artifact_and_artifact_inspect_are_json_capable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir outputs");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs root");
    fs::write(run.join("nodes/extract/outputs/data.csv"), b"a,b\n1,2\n").expect("write payload");
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
            "graph_fingerprint": "g-1",
            "tool_version": "0.1.0",
            "jobs": 1,
            "adapters": [],
            "outputs": [],
            "node_counts": {"success":1,"failed":0,"skipped":0,"cached":0},
            "policy": {"deny_network": true, "deny_env": true, "deny_clock": true, "clean_env": true}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files": [{
                "node_id": "extract",
                "node_fingerprint": "fp-extract",
                "name": "data",
                "kind": "file",
                "media_type": "text/csv",
                "size_bytes": 8,
                "sha256": bijux_dag_artifacts::hash::sha256_hex(b"a,b\n1,2\n"),
                "path": "nodes/extract/outputs/data.csv"
            }]
        }))
        .expect("index"),
    )
    .expect("write index");

    let hash = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "hash",
            "artifact",
            run.join("nodes/extract/outputs/data.csv").to_string_lossy().as_ref(),
        ])
        .expect("parse hash artifact");
    assert_eq!(dag_run(&hash).expect("hash artifact run"), std::process::ExitCode::SUCCESS);

    let inspect = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "artifact-inspect",
            run.to_string_lossy().as_ref(),
            "extract:data.csv",
        ])
        .expect("parse artifact inspect");
    assert_eq!(dag_run(&inspect).expect("artifact inspect run"), std::process::ExitCode::SUCCESS);
}

#[test]
fn artifact_promote_is_json_capable_for_promotable_outputs() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    let deliverables = tmp.path().join("deliverables");
    fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir outputs");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs root");
    fs::write(run.join("nodes/extract/outputs/data.csv"), b"a,b\n1,2\n").expect("write payload");
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
            "graph_fingerprint": "g-1",
            "planner_contract_version": "planner-contract/v0.1",
            "tool_version": "0.1.0",
            "jobs": 1,
            "adapters": [],
            "outputs": [],
            "node_counts": {"success":1,"failed":0,"skipped":0,"cached":0,"cancelled":0},
            "policy": {"deny_network": true, "deny_env": true, "deny_clock": true, "clean_env": true}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files": [{
                "node_id": "extract",
                "node_fingerprint": "fp-extract",
                "name": "data",
                "kind": "file",
                "media_type": "text/csv",
                "size_bytes": 8,
                "sha256": bijux_dag_artifacts::hash::sha256_hex(b"a,b\n1,2\n"),
                "path": "nodes/extract/outputs/data.csv",
                "promotable": true
            }]
        }))
        .expect("index"),
    )
    .expect("write index");
    fs::write(
        run.join("lineage.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version":"lineage/v1",
            "edges":[{
                "artifact_id":"extract:data.csv",
                "producer_node_id":"extract",
                "upstream_artifact_ids":["source:input.csv"]
            }]
        }))
        .expect("lineage"),
    )
    .expect("write lineage");

    let promote = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "artifact",
            "promote",
            run.to_string_lossy().as_ref(),
            "extract:data.csv",
            "--deliverables-root",
            deliverables.to_string_lossy().as_ref(),
            "--to",
            "release",
        ])
        .expect("parse artifact promote");
    assert_eq!(dag_run(&promote).expect("artifact promote run"), std::process::ExitCode::SUCCESS);
}
