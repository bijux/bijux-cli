use base64 as _;
use bijux_dag_app::{inspect_summary, runs_history, runs_summary};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn required_schema_fields(schema_rel: &str) -> Vec<String> {
    let root = repo_root();
    let schema: Value = serde_json::from_str(
        &fs::read_to_string(root.join(schema_rel)).expect("schema should exist"),
    )
    .expect("schema parse");
    schema
        .get("required")
        .and_then(Value::as_array)
        .expect("required fields")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

fn write_manifest(run: &Path, run_id: &str) {
    fs::create_dir_all(run.join("nodes/a")).expect("mkdir");
    fs::write(
        run.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version": "run-manifest/v0.1",
            "run_id": run_id,
            "created_unix_ms": 1,
            "started_unix_ms": 2,
            "finished_unix_ms": 3,
            "graph_snapshot": "snapshot.json",
            "status": "success",
            "spec": "bijux-dag/v0.1",
            "graph_fingerprint": "graph-1",
            "tool_version": "0.1.0",
            "jobs": 1,
            "adapters": [],
            "outputs": [],
            "node_counts": {"success": 1, "failed": 0, "skipped": 0, "cached": 0},
            "policy": {"deny_network": true, "deny_env": true, "deny_clock": true, "clean_env": true},
            "run_metadata": {
                "submission_source": "manual",
                "trigger_source": "cli",
                "operator": "tester",
                "labels": [],
                "parent_run_id": "run-parent",
                "source_run_id": "run-source"
            }
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(run.join("snapshot.json"), "{}").expect("snapshot");
    fs::write(run.join("outputs.index.json"), "{\"files\":[]}").expect("outputs");
}

#[test]
fn run_show_and_inspect_outputs_cover_required_schema_fields() {
    let tmp = tempfile::tempdir().expect("tmp");
    let run = tmp.path().join("run-1");
    write_manifest(&run, "run-1");

    let summary = inspect_summary(&run).expect("summary");
    for schema_rel in [
        "configs/dag/schema/operator/run_show.schema.json",
        "configs/dag/schema/operator/run_inspect.schema.json",
    ] {
        for field in required_schema_fields(schema_rel) {
            assert!(
                summary.get(&field).is_some(),
                "summary output must include required field `{field}` from {schema_rel}"
            );
        }
    }
}

#[test]
fn run_history_output_covers_required_schema_fields() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-a"), "run-a");

    let history = runs_history(&root).expect("history");
    let rows = history["runs"].as_array().expect("rows");
    assert_eq!(rows.len(), 1);
    let row = &rows[0];

    let root = repo_root();
    let history_schema: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/schema/operator/run_history.schema.json"))
            .expect("history schema"),
    )
    .expect("parse");
    let item_required = history_schema["properties"]["runs"]["items"]["required"]
        .as_array()
        .expect("item required");
    for field in item_required.iter().filter_map(Value::as_str) {
        assert!(row.get(field).is_some(), "history row must include `{field}`");
    }
}

#[test]
fn run_summary_output_covers_required_schema_fields() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-a"), "run-a");

    let summary = runs_summary(&root).expect("summary");
    for field in required_schema_fields("configs/dag/schema/operator/run_summary.schema.json") {
        assert!(
            summary.get(&field).is_some(),
            "run summary output must include required field `{field}`"
        );
    }
}
