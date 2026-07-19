use base64 as _;
use bijux_dag_app::{explain_run_id, inspect_summary, runs_history, runs_history_query};
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
use std::path::Path;
use std::time::Instant;
use tar as _;
use tempfile as _;
use thiserror as _;

fn write_manifest(
    run_dir: &Path,
    run_id: &str,
    graph_fingerprint: &str,
    env_fingerprint: &str,
    parent_run_id: Option<&str>,
    source_run_id: Option<&str>,
) {
    fs::create_dir_all(run_dir).expect("mkdir");
    fs::write(
        run_dir.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id": run_id,
            "created_unix_ms": 1,
            "started_unix_ms": 2,
            "finished_unix_ms": 3,
            "graph_snapshot":"graph.snapshot.json",
            "status":"success",
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint": graph_fingerprint,
            "tool_version":"0.1.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "run_metadata":{
              "submission_source":"manual",
              "trigger_source":"cli",
              "environment_fingerprint": env_fingerprint,
              "parent_run_id": parent_run_id,
              "source_run_id": source_run_id
            }
        }))
        .expect("manifest"),
    )
    .expect("write manifest");
    fs::write(run_dir.join("graph.snapshot.json"), "{}").expect("snapshot");
    fs::write(run_dir.join("snapshot.json"), "{}").expect("legacy snapshot");
    fs::write(run_dir.join("outputs.index.json"), "{\"files\":[]}").expect("outputs index");
    fs::create_dir_all(run_dir.join("outputs")).expect("outputs dir");
    fs::write(run_dir.join("outputs").join("index.json"), "{\"files\":[]}").expect("outputs");
}

fn identity_projection(root: &Path, run_id: &str) -> Value {
    let explained = explain_run_id(root, run_id).expect("explain run id");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(root.join(run_id).join("manifest.json")).expect("manifest"),
    )
    .expect("parse manifest");
    json!({
      "run_id": explained["run_id"],
      "graph_fingerprint": manifest["graph_fingerprint"],
      "environment_fingerprint": manifest["run_metadata"]["environment_fingerprint"],
      "parent_run_id": explained["parent_run_id"],
      "source_run_id": explained["source_run_id"]
    })
}

fn required_fields(schema_rel: &str) -> Vec<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let schema: Value =
        serde_json::from_str(&fs::read_to_string(root.join(schema_rel)).expect("schema"))
            .expect("schema json");
    schema["required"]
        .as_array()
        .expect("required")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect()
}

#[test]
fn run_identity_projection_is_stable_across_repeated_inspection() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-a"), "run-a", "graph-a", "env-a", None, None);
    let first = identity_projection(&root, "run-a");
    let second = identity_projection(&root, "run-a");
    assert_eq!(first, second);
}

#[test]
fn run_identity_projection_changes_when_graph_environment_or_ancestry_changes() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(
        &root.join("run-a"),
        "run-a",
        "graph-a",
        "env-a",
        Some("parent-a"),
        Some("source-a"),
    );
    write_manifest(
        &root.join("run-b"),
        "run-b",
        "graph-b",
        "env-b",
        Some("parent-b"),
        Some("source-b"),
    );
    let a = identity_projection(&root, "run-a");
    let b = identity_projection(&root, "run-b");
    assert_ne!(a["graph_fingerprint"], b["graph_fingerprint"]);
    assert_ne!(a["environment_fingerprint"], b["environment_fingerprint"]);
    assert_ne!(a["parent_run_id"], b["parent_run_id"]);
    assert_ne!(a["source_run_id"], b["source_run_id"]);
}

#[test]
fn run_summary_and_detail_output_fields_are_schema_lockstep_stable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_manifest(&root.join("run-detail"), "run-detail", "graph-x", "env-x", None, None);

    let summary = inspect_summary(&root.join("run-detail")).expect("inspect summary");
    for field in required_fields("configs/dag/schema/operator/run_inspect.schema.json") {
        assert!(summary.get(&field).is_some(), "summary missing {field}");
    }

    let history = runs_history(&root).expect("history");
    let row = history["runs"]
        .as_array()
        .expect("rows")
        .iter()
        .find(|r| r["run_id"] == "run-detail")
        .expect("row");
    let root_repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let history_schema: Value = serde_json::from_str(
        &fs::read_to_string(root_repo.join("configs/dag/schema/operator/run_history.schema.json"))
            .expect("history schema"),
    )
    .expect("parse history schema");
    let item_required = history_schema["properties"]["runs"]["items"]["required"]
        .as_array()
        .expect("item required");
    for field in item_required.iter().filter_map(Value::as_str) {
        assert!(row.get(field).is_some(), "detail row missing {field}");
    }
}

#[test]
fn run_history_stress_suite_many_runs_is_deterministic() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    for idx in 0..400_u32 {
        write_manifest(
            &root.join(format!("run-{idx:04}")),
            &format!("run-{idx:04}"),
            "graph-stress",
            "env-stress",
            if idx > 0 { Some("run-0000") } else { None },
            None,
        );
    }
    let first = runs_history(&root).expect("first history");
    let second = runs_history(&root).expect("second history");
    assert_eq!(first, second);
    assert_eq!(first["runs"].as_array().expect("rows").len(), 400);
}

#[test]
fn corrupted_manifest_and_missing_metadata_recover_without_panic() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    fs::create_dir_all(root.join("run-corrupt")).expect("mkdir");
    fs::write(root.join("run-corrupt").join("manifest.json"), "{not-json").expect("write");

    write_manifest(
        &root.join("run-missing-metadata"),
        "run-missing-metadata",
        "graph-ok",
        "env-ok",
        None,
        None,
    );
    let mut manifest: Value = serde_json::from_str(
        &fs::read_to_string(root.join("run-missing-metadata").join("manifest.json"))
            .expect("manifest"),
    )
    .expect("json");
    manifest.as_object_mut().unwrap().remove("run_metadata");
    fs::write(
        root.join("run-missing-metadata").join("manifest.json"),
        serde_json::to_vec_pretty(&manifest).expect("encode"),
    )
    .expect("write");

    let history = runs_history(&root).expect("history");
    assert_eq!(history["runs"].as_array().expect("rows").len(), 2);
}

#[test]
fn run_history_query_performance_contract_on_large_fixture_set() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    for idx in 0..500_u32 {
        write_manifest(
            &root.join(format!("run-{idx:04}")),
            &format!("run-{idx:04}"),
            "graph-perf",
            "env-perf",
            None,
            None,
        );
    }

    let start = Instant::now();
    let payload =
        runs_history_query(&root, Some("success"), None, Some((100, 150))).expect("history query");
    let elapsed = start.elapsed();

    assert_eq!(payload["runs"].as_array().expect("rows").len(), 150);
    assert!(
        elapsed.as_millis() < 1500,
        "history query performance budget exceeded: {} ms",
        elapsed.as_millis()
    );
}

#[test]
fn run_manifest_regression_corpus_fixture_is_stable_and_parseable() {
    let corpus: Value = serde_json::from_str(include_str!(
        "../../../evidence/dag/cache/replay/run_manifest_regression_corpus.json"
    ))
    .expect("corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 3);
    for case in cases {
        let manifest = case["manifest"].clone();
        assert!(manifest["run_id"].is_string());
        assert!(manifest["graph_fingerprint"].is_string());
        assert!(manifest["status"].is_string());
    }
}
