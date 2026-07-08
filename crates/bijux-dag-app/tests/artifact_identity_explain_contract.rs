use base64 as _;
use bijux_dag_app::inspect_artifact;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
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
                "name":"data",
                "kind":"file",
                "media_type":"text/csv",
                "size_bytes": 7,
                "sha256":sha,
                "path":"nodes/extract/outputs/data.csv",
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

    for field in required_fields("configs/dag/schema/operator/artifact_inspect.schema.json") {
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
    assert_eq!(inspected["promotable"], true);
    assert_eq!(inspected["provenance"]["run_id"], "run-1");
    assert_eq!(inspected["legacy_artifact_id"], "extract:data.csv");
    assert_eq!(inspected["lineage"]["upstream_artifact_ids"][0], "source:input.csv");
    assert_eq!(inspected["lineage"]["downstream_artifact_ids"][0], "train:model.bin");

    let explain = &inspected["identity_explain"];
    assert_eq!(explain["legacy_artifact_id"], "extract:data.csv");
    assert!(explain["artifact_id"]
        .as_str()
        .expect("canonical artifact id")
        .starts_with("run=run-1;node=extract;path=nodes/extract/outputs/data.csv;sha256="));
    assert_eq!(explain["composed_from"]["run_id"], "run-1");
    assert_eq!(explain["composed_from"]["node_id"], "extract");
    assert_eq!(explain["composed_from"]["output_name"], "data.csv");
    assert_eq!(explain["hash_algorithm"], "sha256");
    assert_eq!(explain["collision_safe"], true);
}

#[test]
fn provenance_traversal_is_deterministic_across_repeated_inspection() {
    let (_tmp, run) = setup_run_with_lineage();
    let first = inspect_artifact(&run, "extract:data.csv").expect("inspect first");
    let second = inspect_artifact(&run, "extract:data.csv").expect("inspect second");
    assert_eq!(
        first["lineage"]["upstream_artifact_ids"],
        second["lineage"]["upstream_artifact_ids"]
    );
    assert_eq!(
        first["lineage"]["downstream_artifact_ids"],
        second["lineage"]["downstream_artifact_ids"]
    );
}

#[test]
fn provenance_serialization_is_stable_for_repeated_inspection() {
    let (_tmp, run) = setup_run_with_lineage();
    let first = inspect_artifact(&run, "extract:data.csv").expect("inspect first");
    let second = inspect_artifact(&run, "extract:data.csv").expect("inspect second");
    let first_json = serde_json::to_string(&first).expect("serialize first");
    let second_json = serde_json::to_string(&second).expect("serialize second");
    assert_eq!(first_json, second_json);
}

#[test]
fn canonical_artifact_id_is_accepted_for_lookup() {
    let (_tmp, run) = setup_run_with_lineage();
    let legacy = inspect_artifact(&run, "extract:data.csv").expect("legacy inspect");
    let canonical_id = legacy["artifact_id"].as_str().expect("canonical id").to_string();
    let canonical = inspect_artifact(&run, &canonical_id).expect("canonical inspect");
    assert_eq!(canonical["artifact_id"], legacy["artifact_id"]);
    assert_eq!(canonical["legacy_artifact_id"], "extract:data.csv");
}

#[test]
fn provenance_query_latency_contract_on_large_lineage_snapshot() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-latency");
    fs::create_dir_all(run.join("nodes/extract/outputs")).expect("mkdir");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs");
    fs::write(run.join("nodes/extract/outputs/data.csv"), b"payload").expect("payload");
    let sha = bijux_dag_artifacts::hash::sha256_hex(b"payload");

    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id":"run-latency",
            "created_unix_ms":1,
            "started_unix_ms":1,
            "finished_unix_ms":2,
            "graph_snapshot":"graph.snapshot.json",
            "status":"success",
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"graph-latency",
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
                "name":"data",
                "kind":"file",
                "media_type":"text/csv",
                "size_bytes": 7,
                "sha256":sha,
                "path":"nodes/extract/outputs/data.csv"
            }]
        }))
        .expect("index"),
    )
    .expect("write index");

    let mut edges = Vec::new();
    edges.push(json!({
        "artifact_id":"extract:data.csv",
        "producer_node_id":"extract",
        "upstream_artifact_ids":["source:input.csv"]
    }));
    for idx in 0..1500_u32 {
        edges.push(json!({
            "artifact_id": format!("node-{idx}:out"),
            "producer_node_id": format!("node-{idx}"),
            "upstream_artifact_ids": [format!("node-{}:out", idx.saturating_sub(1))]
        }));
    }
    edges.push(json!({
        "artifact_id":"train:model.bin",
        "producer_node_id":"train",
        "upstream_artifact_ids":["extract:data.csv"]
    }));
    fs::write(
        run.join("lineage.snapshot.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version":"lineage/v1",
            "edges": edges
        }))
        .expect("lineage"),
    )
    .expect("write lineage");

    let start = Instant::now();
    let inspected = inspect_artifact(&run, "extract:data.csv").expect("inspect");
    let elapsed_ms = start.elapsed().as_millis();
    assert!(inspected["artifact_id"]
        .as_str()
        .expect("canonical artifact id")
        .starts_with("run=run-latency;node=extract;path=nodes/extract/outputs/data.csv;sha256="));
    assert!(elapsed_ms < 2000, "provenance query latency exceeded contract bound: {elapsed_ms}ms");
}
