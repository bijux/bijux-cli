use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{CacheMode, Runtime, RuntimeConfig, Selector, SelectorSet};
use serde_json::Value;
use std::fs;
use std::path::Path;

fn simple_const_graph() -> String {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "const1",
          "kind": "const",
          "inputs": [],
          "outputs": [
            {
              "name": "value",
              "path": "value.txt"
            }
          ],
          "params": {
            "value": "hello"
          }
        }
      ],
      "edges": []
    }"#
    .to_string()
}

fn read_counts(manifest: &Path) -> (u32, u32, u32, u32) {
    let data: Value =
        serde_json::from_str(&fs::read_to_string(manifest).expect("manifest")).unwrap();
    let counts = &data["node_counts"];
    (
        counts["success"].as_u64().unwrap_or(0) as u32,
        counts["failed"].as_u64().unwrap_or(0) as u32,
        counts["skipped"].as_u64().unwrap_or(0) as u32,
        counts["cached"].as_u64().unwrap_or(0) as u32,
    )
}

#[test]
fn runtime_executes_const_graph_and_emits_output_trace() {
    let graph = parse_graph_strict(&simple_const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let manifest = run_dir.join("manifest.json");
    let data: Value =
        serde_json::from_str(&fs::read_to_string(&manifest).expect("read manifest")).unwrap();
    assert_eq!(data["status"], "success");
    assert!(data["graph_fingerprint"].as_str().is_some());

    let output_file = run_dir.join("nodes").join("const1").join("outputs").join("value.txt");
    let rendered = fs::read_to_string(&output_file).expect("output file");
    assert_eq!(rendered.trim(), "\"hello\"");

    let trace_file = run_dir.join("nodes").join("const1").join("trace.json");
    let trace: Value = serde_json::from_str(&fs::read_to_string(&trace_file).expect("trace"))
        .expect("trace parse");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["node_id"], "const1");
}

#[test]
fn runtime_replay_contract_preserves_fingerprint_and_outputs() {
    let graph = parse_graph_strict(&simple_const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");
    let run_one = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("first run");
    let run_two = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("second run");

    let manifest_one: Value = serde_json::from_str(
        &fs::read_to_string(run_one.join("manifest.json")).expect("manifest one"),
    )
    .expect("manifest parse");
    let manifest_two: Value = serde_json::from_str(
        &fs::read_to_string(run_two.join("manifest.json")).expect("manifest two"),
    )
    .expect("manifest parse");
    assert_eq!(manifest_one["graph_fingerprint"], manifest_two["graph_fingerprint"]);

    let first_output =
        fs::read_to_string(run_one.join("nodes").join("const1").join("outputs").join("value.txt"))
            .expect("first output");
    let second_output =
        fs::read_to_string(run_two.join("nodes").join("const1").join("outputs").join("value.txt"))
            .expect("second output");
    assert_eq!(first_output, second_output);
}

#[test]
fn runtime_cache_contract_uses_cached_nodes_when_enabled() {
    let graph = parse_graph_strict(&simple_const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");
    let cache = temp.path().join("cache");

    let run_one = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache.clone()),
                ..RuntimeConfig::default()
            },
        )
        .expect("first cached run");

    let run_two = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache),
                ..RuntimeConfig::default()
            },
        )
        .expect("second cached run");

    let (one_success, one_failed, one_skipped, one_cached) =
        read_counts(&run_one.join("manifest.json"));
    let (_, _, _, two_cached) = read_counts(&run_two.join("manifest.json"));

    assert_eq!(one_failed, 0);
    assert_eq!(one_skipped, 0);
    assert_eq!(one_success, 1);
    assert_eq!(one_cached, 0);
    assert!(two_cached >= 1);
}

#[test]
fn run_snapshot_records_requested_selectors_and_selected_nodes() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "const1",
              "kind": "const",
              "tags": ["seed"],
              "inputs": [],
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"value": "hello"}
            }
          ],
          "edges": []
        }"#,
    )
    .expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                selectors: SelectorSet {
                    include: vec![Selector::Tag("seed".to_string())],
                    exclude: vec![],
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let snapshot: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("run.snapshot.json")).expect("run snapshot"),
    )
    .expect("snapshot parse");
    assert_eq!(snapshot["requested_selectors"], serde_json::json!(["tag:seed"]));
    assert_eq!(snapshot["selected_nodes"], serde_json::json!(["const1"]));
}
