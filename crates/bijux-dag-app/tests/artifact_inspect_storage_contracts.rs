use base64 as _;
use bijux_dag_app::inspect_artifact;
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

fn write_manifest(run: &std::path::Path) {
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
}

#[test]
fn artifact_inspect_reports_missing_payload_when_metadata_entry_exists() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("outputs")).expect("mkdir");
    write_manifest(&run);

    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files":[{
                "node_id":"extract",
                "node_fingerprint":"fp",
                "name":"data",
                "kind":"file",
                "media_type":"text/csv",
                "size_bytes": 17,
                "sha256":"abc",
                "path":"nodes/extract/outputs/data.csv"
            }]
        }))
        .expect("index"),
    )
    .expect("write index");

    let inspected = inspect_artifact(&run, "extract:data.csv").expect("inspect");
    assert_eq!(inspected["payload_missing"], true);
    assert!(inspected["size_bytes"].is_null());
}

#[test]
fn artifact_inspect_reports_recorded_size_for_present_payload() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    let payload = b"a,b\n1,2\n";
    fs::create_dir_all(run.join("outputs")).expect("mkdir");
    fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir node outputs");
    write_manifest(&run);
    fs::write(run.join("nodes/extract/outputs/data.csv"), payload).expect("write payload");

    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files":[{
                "node_id":"extract",
                "node_fingerprint":"fp",
                "name":"data",
                "kind":"file",
                "media_type":"text/csv",
                "size_bytes": payload.len(),
                "sha256":"abc",
                "path":"nodes/extract/outputs/data.csv"
            }]
        }))
        .expect("index"),
    )
    .expect("write index");

    let inspected = inspect_artifact(&run, "extract:data.csv").expect("inspect");
    assert_eq!(inspected["payload_missing"], false);
    assert_eq!(inspected["size_bytes"], payload.len());
}

#[test]
fn artifact_inspect_rejects_corrupted_outputs_index_hash_rows() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("outputs")).expect("mkdir");
    write_manifest(&run);

    fs::write(run.join("outputs/index.json"), "{not-json").expect("write corrupted index");

    let err = inspect_artifact(&run, "extract:data.csv").expect_err("inspect should fail");
    assert_eq!(err, std::process::ExitCode::from(3));
}
