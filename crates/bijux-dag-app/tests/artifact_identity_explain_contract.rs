use base64 as _;
use bijux_dag_app::inspect_artifact;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tar as _;
use tempfile as _;
use thiserror as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn required_fields(schema_rel: &str) -> Vec<String> {
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(repo_root().join(schema_rel)).expect("schema file"),
    )
    .expect("schema parse");
    schema
        .get("required")
        .and_then(Value::as_array)
        .expect("required")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn setup_run_with_lineage() -> (tempfile::TempDir, PathBuf) {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs");

    let payload = b"a,b\n1,2\n";
    fs::write(run.join("nodes/extract/outputs/data.csv"), payload).expect("write artifact");
    let sha = bijux_dag_artifacts::hash::sha256_hex(payload);

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
            "graph_fingerprint":"graph-1",
            "tool_version":"0.1.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true}
        }))
        .expect("manifest"),
    )
    .expect("write manifest");

    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files":[{
                "node_id":"extract",
                "node_fingerprint":"fp-extract",
                "sha256":sha,
                "path":"nodes/extract/outputs/data.csv"
            }]
        }))
        .expect("index"),
    )
    .expect("write index");

    fs::write(
        run.join("lineage.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version":"lineage/v1",
            "edges":[
                {
                    "artifact_id":"extract:data.csv",
                    "producer_node_id":"extract",
                    "upstream_artifact_ids":["source:input.csv"]
                },
                {
                    "artifact_id":"train:model.bin",
                    "producer_node_id":"train",
                    "upstream_artifact_ids":["extract:data.csv"]
                }
            ]
        }))
        .expect("lineage"),
    )
    .expect("write lineage");

    (tmp, run)
}

#[test]
fn artifact_inspect_schema_required_fields_lockstep() {
    let (_tmp, run) = setup_run_with_lineage();
    let inspected = inspect_artifact(&run, "extract:data.csv").expect("inspect");

    for field in required_fields("configs/schema/operator/artifact_inspect.schema.json") {
        assert!(
            inspected.get(&field).is_some(),
            "artifact inspect output missing required field `{field}`"
        );
    }
}

#[test]
fn artifact_identity_explain_covers_provenance_and_lineage_traversal() {
    let (_tmp, run) = setup_run_with_lineage();
    let inspected = inspect_artifact(&run, "extract:data.csv").expect("inspect");

    assert_eq!(inspected["node_id"], "extract");
    assert_eq!(inspected["provenance"]["run_id"], "run-1");
    assert_eq!(
        inspected["lineage"]["upstream_artifact_ids"][0],
        "source:input.csv"
    );
    assert_eq!(
        inspected["lineage"]["downstream_artifact_ids"][0],
        "train:model.bin"
    );

    let explain = &inspected["identity_explain"];
    assert_eq!(explain["artifact_id"], "extract:data.csv");
    assert_eq!(explain["composed_from"]["run_id"], "run-1");
    assert_eq!(explain["composed_from"]["node_id"], "extract");
    assert_eq!(explain["hash_algorithm"], "sha256");
}
