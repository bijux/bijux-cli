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

fn reduce_collection_command(output_name: &str, empty_value: Option<&str>) -> String {
    let empty_expr =
        empty_value.map(|value| format!("'{value}'")).unwrap_or_else(|| "''".to_string());
    format!(
        "python3 -c \"import json, pathlib; manifest=json.load(open('../inputs/reduce.collection.json')); values=[]; \
base=pathlib.Path('../inputs'); collect=lambda rel: sorted((base / rel).rglob('value.txt')) if (base / rel).is_dir() else [base / rel]; \
paths=[]; [paths.extend(collect(item['local_path'])) for item in manifest['items'] if item.get('local_path')]; \
values=[path.read_text() for path in paths]; \
output=','.join(values) if values else {empty_expr}; \
(pathlib.Path('../outputs') / '{output_name}').write_text(output)\""
    )
}

fn semantic_map_reduce_graph() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "seed",
          "kind": "const",
          "outputs": [{"name": "out", "path": "seed/items.json"}],
          "params": {"value": ["alpha", "beta"]}
        },
        {
          "id": "map",
          "kind": "shell",
          "semantic_kind": "map",
          "inputs": ["in"],
          "outputs": [{"name": "out", "path": "mapped", "kind": "directory"}],
          "effects": ["filesystem"],
          "params": {
            "argv": [
              "/bin/sh",
              "-c",
              "value=$(tr -d '\"' < ../inputs/seed/in); mkdir -p ../outputs/mapped; printf '%s' \"$value\" > ../outputs/mapped/value.txt"
            ]
          }
        },
        {
          "id": "reduce",
          "kind": "shell",
          "semantic_kind": "reduce",
          "inputs": ["mapped"],
          "outputs": [{"name": "out", "path": "reduce.txt"}],
          "effects": ["filesystem"],
          "params": {
            "argv": [
              "/bin/sh",
              "-c",
              reduce_collection_command("reduce.txt", None)
            ]
          }
        }
      ],
      "edges": [
        {"from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "map", "port": "in"}},
        {"from": {"node_id": "map", "port": "out"}, "to": {"node_id": "reduce", "port": "mapped"}}
      ]
    }))
    .expect("serialize graph")
}

fn semantic_map_failure_graph() -> String {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "seed",
          "kind": "const",
          "outputs": [{"name": "out", "path": "seed/items.json"}],
          "params": {"value": ["ok", "fail", "later"]}
        },
        {
          "id": "map",
          "kind": "shell",
          "semantic_kind": "map",
          "inputs": ["in"],
          "outputs": [{"name": "out", "path": "mapped", "kind": "directory"}],
          "effects": ["filesystem"],
          "params": {
            "argv": [
              "/bin/sh",
              "-c",
              "value=$(tr -d '\"' < ../inputs/seed/in); mkdir -p ../outputs/mapped; if [ \"$value\" = fail ]; then printf 'broken item' >&2; exit 7; fi; printf '%s' \"$value\" > ../outputs/mapped/value.txt"
            ]
          }
        }
      ],
      "edges": [
        {"from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "map", "port": "in"}}
      ]
    }"#
    .to_string()
}

fn reduce_partial_graph() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "left",
          "kind": "shell",
          "outputs": [{"name": "out", "path": "left.txt"}],
          "effects": ["filesystem"],
          "params": {
            "argv": ["/bin/sh", "-c", "printf alpha > ../outputs/left.txt"]
          }
        },
        {
          "id": "right",
          "kind": "shell",
          "outputs": [{"name": "out", "path": "right.txt"}],
          "effects": ["filesystem"],
          "params": {
            "argv": ["/bin/sh", "-c", "printf broken >&2; exit 9"]
          }
        },
        {
          "id": "reduce",
          "kind": "shell",
          "semantic_kind": "reduce",
          "inputs": ["left", "right"],
          "outputs": [{"name": "out", "path": "reduce.txt"}],
          "effects": ["filesystem"],
          "params": {
            "reduce": {"mode": "partial"},
            "argv": ["/bin/sh", "-c", reduce_collection_command("reduce.txt", None)]
          }
        }
      ],
      "edges": [
        {"from": {"node_id": "left", "port": "out"}, "to": {"node_id": "reduce", "port": "left"}},
        {"from": {"node_id": "right", "port": "out"}, "to": {"node_id": "reduce", "port": "right"}}
      ]
    }))
    .expect("serialize graph")
}

fn reduce_empty_allow_graph() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "reduce",
          "kind": "shell",
          "semantic_kind": "reduce",
          "inputs": [],
          "outputs": [{"name": "out", "path": "reduce.txt"}],
          "effects": ["filesystem"],
          "params": {
            "reduce": {"empty": "allow"},
            "argv": ["/bin/sh", "-c", reduce_collection_command("reduce.txt", Some("empty"))]
          }
        }
      ],
      "edges": []
    }))
    .expect("serialize graph")
}

fn reduce_empty_skip_graph() -> String {
    serde_json::to_string_pretty(&serde_json::json!({
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "reduce",
          "kind": "shell",
          "semantic_kind": "reduce",
          "inputs": [],
          "outputs": [{"name": "out", "path": "reduce.txt"}],
          "effects": ["filesystem"],
          "params": {
            "reduce": {"empty": "skip"},
            "argv": ["/bin/sh", "-c", reduce_collection_command("reduce.txt", Some("unused"))]
          }
        }
      ],
      "edges": []
    }))
    .expect("serialize graph")
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

fn read_run_events(run_dir: &Path) -> Vec<Value> {
    fs::read_to_string(run_dir.join("run.log.jsonl"))
        .expect("run log")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<Value>(line).expect("event json"))
        .collect()
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
    assert!(run_dir.join("manifest.finalized.json").exists());
    assert!(run_dir.join(".run-complete.json").exists());
    assert!(run_dir.join("run.schema.json").exists());

    let output_file = run_dir.join("nodes").join("const1").join("outputs").join("value.txt");
    let rendered = fs::read_to_string(&output_file).expect("output file");
    assert_eq!(rendered.trim(), "\"hello\"");

    let trace_file = run_dir.join("nodes").join("const1").join("trace.json");
    let trace: Value = serde_json::from_str(&fs::read_to_string(&trace_file).expect("trace"))
        .expect("trace parse");
    assert_eq!(trace["status"], "success");
    assert_eq!(trace["node_id"], "const1");
    assert_eq!(trace["planner_contract_version"], "bijux-dag-planner/v1");
    assert!(trace["execution_fingerprint"].as_str().is_some());
    assert!(trace["evidence_fingerprint"].as_str().is_some());
}

#[test]
fn runtime_executes_semantic_map_node_and_reduce_consumes_directory_outputs() {
    let graph = parse_graph_strict(&semantic_map_reduce_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let reduce_output =
        fs::read_to_string(run_dir.join("nodes").join("reduce").join("outputs").join("reduce.txt"))
            .expect("reduce output");
    assert_eq!(reduce_output, "alpha,beta");

    let reduce_inputs: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("inputs").join("index.json"))
            .expect("reduce inputs index"),
    )
    .expect("parse reduce inputs index");
    assert_eq!(reduce_inputs["collections"][0]["semantic_kind"], "reduce");
    assert_eq!(reduce_inputs["collections"][0]["items"][0]["status"], "success");

    let reduce_summary: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("reduce.execution.json"))
            .expect("reduce summary"),
    )
    .expect("parse reduce summary");
    assert_eq!(reduce_summary["usable_input_count"], 1);
    assert_eq!(reduce_summary["failed_input_count"], 0);

    let map_summary: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("map").join("map.execution.json"))
            .expect("map summary"),
    )
    .expect("parse map summary");
    assert_eq!(map_summary["item_count"], 2);
    assert_eq!(map_summary["successful_item_count"], 2);
    assert_eq!(map_summary["failed_item_count"], 0);

    let item_root = run_dir.join("nodes").join("map").join("outputs").join("mapped").join("items");
    let mut item_values = fs::read_dir(&item_root)
        .expect("item root")
        .filter_map(|entry| entry.ok())
        .map(|entry| fs::read_to_string(entry.path().join("value.txt")).expect("item value"))
        .collect::<Vec<_>>();
    item_values.sort();
    assert_eq!(item_values, vec!["alpha".to_string(), "beta".to_string()]);
}

#[test]
fn runtime_aggregates_semantic_map_item_failures_without_hiding_successful_outputs() {
    let graph = parse_graph_strict(&semantic_map_failure_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("manifest parse");
    assert_eq!(manifest["status"], "failed");

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("map").join("trace.json")).expect("trace"),
    )
    .expect("trace parse");
    assert_eq!(trace["status"], "failed");
    assert_eq!(trace["failure"]["code"], "MAP_ITEMS_FAILED");

    let map_summary: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("map").join("map.execution.json"))
            .expect("map summary"),
    )
    .expect("parse map summary");
    assert_eq!(map_summary["item_count"], 3);
    assert_eq!(map_summary["successful_item_count"], 2);
    assert_eq!(map_summary["failed_item_count"], 1);
    assert!(map_summary["items"]
        .as_array()
        .expect("items array")
        .iter()
        .any(|item| { item["status"] == "failed" && item["failure"].is_object() }));

    let item_root = run_dir.join("nodes").join("map").join("outputs").join("mapped").join("items");
    let mut item_values = fs::read_dir(&item_root)
        .expect("item root")
        .filter_map(|entry| entry.ok())
        .map(|entry| fs::read_to_string(entry.path().join("value.txt")).expect("item value"))
        .collect::<Vec<_>>();
    item_values.sort();
    assert_eq!(item_values, vec!["later".to_string(), "ok".to_string()]);
}

#[test]
fn runtime_executes_partial_reduce_after_failed_upstream_and_records_manifest_statuses() {
    let graph = parse_graph_strict(&reduce_partial_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let reduce_output =
        fs::read_to_string(run_dir.join("nodes").join("reduce").join("outputs").join("reduce.txt"))
            .expect("reduce output");
    assert_eq!(reduce_output, "alpha");

    let reduce_trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("trace.json"))
            .expect("trace"),
    )
    .expect("trace parse");
    assert_eq!(reduce_trace["status"], "success");
    assert_eq!(reduce_trace["trigger_evaluation"]["trigger_rule"], "reduce_partial");

    let reduce_summary: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("reduce.execution.json"))
            .expect("reduce summary"),
    )
    .expect("parse reduce summary");
    assert_eq!(reduce_summary["usable_input_count"], 1);
    assert_eq!(reduce_summary["failed_input_count"], 1);
    assert!(reduce_summary["collection"]["items"]
        .as_array()
        .expect("items array")
        .iter()
        .any(|item| item["source_node_id"] == "right" && item["status"] == "failed"));
}

#[test]
fn runtime_allows_empty_reduce_collection_when_configured() {
    let graph = parse_graph_strict(&reduce_empty_allow_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let reduce_output =
        fs::read_to_string(run_dir.join("nodes").join("reduce").join("outputs").join("reduce.txt"))
            .expect("reduce output");
    assert_eq!(reduce_output, "empty");

    let reduce_summary: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("reduce.execution.json"))
            .expect("reduce summary"),
    )
    .expect("parse reduce summary");
    assert_eq!(reduce_summary["usable_input_count"], 0);
    assert_eq!(reduce_summary["empty_policy"], "allow");
}

#[test]
fn runtime_skips_empty_reduce_collection_when_configured() {
    let graph = parse_graph_strict(&reduce_empty_skip_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let reduce_trace: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join("reduce").join("trace.json"))
            .expect("trace"),
    )
    .expect("trace parse");
    assert_eq!(reduce_trace["status"], "skipped");
    assert_eq!(reduce_trace["skip_reason"]["reason"], "empty_reduce_collection");
    assert!(!run_dir.join("nodes").join("reduce").join("outputs").join("reduce.txt").exists());
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

    let trace: Value = serde_json::from_str(
        &fs::read_to_string(run_two.join("nodes").join("const1").join("trace.json"))
            .expect("cached trace"),
    )
    .expect("cached trace parse");
    assert_eq!(trace["lifecycle_state"], "cached");
    assert_eq!(
        trace["lifecycle_transitions"],
        serde_json::json!([
            {
                "from_state": "pending",
                "to_state": "ready",
                "cause": "scheduler_eligible",
                "unix_ms": trace["lifecycle_transitions"][0]["unix_ms"],
            },
            {
                "from_state": "ready",
                "to_state": "queued",
                "cause": "scheduler_queued",
                "unix_ms": trace["lifecycle_transitions"][1]["unix_ms"],
            },
            {
                "from_state": "queued",
                "to_state": "cached",
                "cause": "cached_reuse",
                "unix_ms": trace["lifecycle_transitions"][2]["unix_ms"],
            }
        ])
    );

    let cached_events = read_run_events(&run_two);
    assert!(cached_events
        .iter()
        .any(|event| { event["event"] == "node_scheduled" && event["node_id"] == "const1" }));
    assert!(!cached_events
        .iter()
        .any(|event| { event["event"] == "node_started" && event["node_id"] == "const1" }));
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
    assert_eq!(snapshot["requested_selectors"], serde_json::json!(["include:tag:seed"]));
    assert_eq!(snapshot["selected_nodes"], serde_json::json!(["const1"]));
}

#[test]
fn finalized_run_removes_incomplete_marker_and_keeps_completion_marker() {
    let graph = parse_graph_strict(&simple_const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");
    assert!(!run_dir.join(".run-incomplete.json").exists());
    assert!(run_dir.join(".run-complete.json").exists());
}

#[test]
fn runtime_rejects_selected_gpu_nodes_without_gpu_device_budget() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"train",
              "kind":"const",
              "outputs":[{"name":"out","path":"train/out"}],
              "resources":{"cpu":1,"mem_mb":64,"gpu_devices":1},
              "params":{"value":1}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let error = runtime
        .run(&graph, temp.path(), RuntimeConfig::default())
        .expect_err("gpu budget should be required")
        .to_string();

    assert!(error.contains("gpu_device_budget is unset"));
    assert!(error.contains("train=1"));
}

#[test]
fn runtime_rejects_gpu_nodes_that_exceed_runtime_budget() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"train",
              "kind":"const",
              "outputs":[{"name":"out","path":"train/out"}],
              "resources":{"cpu":1,"mem_mb":64,"gpu_devices":2},
              "params":{"value":1}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let error = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig { gpu_device_budget: Some(1), ..RuntimeConfig::default() },
        )
        .expect_err("oversized gpu request should fail")
        .to_string();

    assert!(error.contains("gpu_device_budget=1"));
    assert!(error.contains("train=2"));
}

#[test]
fn runtime_rejects_named_resources_without_runtime_capacity() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"licensed",
              "kind":"const",
              "outputs":[{"name":"out","path":"licensed/out"}],
              "resources":{"cpu":1,"mem_mb":64,"named_resources":{"license.render":1}},
              "params":{"value":1}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let error = runtime
        .run(&graph, temp.path(), RuntimeConfig::default())
        .expect_err("named resource capacity should be required")
        .to_string();

    assert!(error.contains("named resources without runtime capacity"));
    assert!(error.contains("license.render(licensed=1)"));
}

#[test]
fn runtime_rejects_named_resources_that_exceed_runtime_capacity() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"licensed",
              "kind":"const",
              "outputs":[{"name":"out","path":"licensed/out"}],
              "resources":{"cpu":1,"mem_mb":64,"named_resources":{"license.render":2}},
              "params":{"value":1}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let error = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                named_resource_capacities: std::collections::BTreeMap::from([(
                    "license.render".to_string(),
                    1,
                )]),
                ..RuntimeConfig::default()
            },
        )
        .expect_err("oversized named resource request should fail")
        .to_string();

    assert!(error.contains("more named resources than runtime capacities allow"));
    assert!(error.contains("licensed:license.render=2 exceeds capacity 1"));
}

#[test]
fn runtime_releases_named_resource_capacity_after_terminal_state() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}},"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}},"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                jobs: 2,
                named_resource_capacities: std::collections::BTreeMap::from([(
                    "database_slot".to_string(),
                    1,
                )]),
                scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
                    max_parallelism: 2,
                    cpu_budget: Some(2),
                    named_resource_capacities: std::collections::BTreeMap::from([(
                        "database_slot".to_string(),
                        1,
                    )]),
                    ..bijux_dag_runtime::SchedulerPolicy::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let (success, failed, skipped, cached) = read_counts(&run_dir.join("manifest.json"));
    assert_eq!((success, failed, skipped, cached), (2, 0, 0, 0));

    let events = read_run_events(&run_dir);
    let blocked_index = events
        .iter()
        .position(|event| {
            event["event"] == "scheduler_decision"
                && event["blocked_reasons"]["b"] == "blocked_by_named_resource:database_slot"
        })
        .expect("named resource blocking decision");
    let holder_finished_index = events
        .iter()
        .position(|event| {
            event["event"] == "node_finished"
                && event["node_id"] == "a"
                && event["status"] == "success"
        })
        .expect("holder finished");
    let released_start_index = events
        .iter()
        .position(|event| event["event"] == "node_started" && event["node_id"] == "b")
        .expect("released node started");

    assert!(blocked_index < holder_finished_index);
    assert!(holder_finished_index < released_start_index);
}

#[test]
fn runtime_requeues_parallelism_blocked_roots_after_worker_completion() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"alpha","kind":"const","outputs":[{"name":"out","path":"alpha/out"}],"params":{"value":"a"}},
            {"id":"beta","kind":"const","outputs":[{"name":"out","path":"beta/out"}],"params":{"value":"b"}},
            {"id":"gamma","kind":"const","outputs":[{"name":"out","path":"gamma/out"}],"params":{"value":"c"}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                jobs: 1,
                scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
                    max_parallelism: 3,
                    cpu_budget: Some(3),
                    ..bijux_dag_runtime::SchedulerPolicy::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let events = read_run_events(&run_dir);
    let scheduler_decisions =
        events.iter().filter(|event| event["event"] == "scheduler_decision").collect::<Vec<_>>();
    assert_eq!(scheduler_decisions.len(), 3);
    assert_eq!(scheduler_decisions[0]["batch"], serde_json::json!(["alpha"]));
    assert_eq!(
        scheduler_decisions[0]["blocked_reasons"],
        serde_json::json!({
            "beta": "blocked_by_parallelism",
            "gamma": "blocked_by_parallelism"
        })
    );
    assert_eq!(scheduler_decisions[1]["batch"], serde_json::json!(["beta"]));
    assert_eq!(
        scheduler_decisions[1]["blocked_reasons"],
        serde_json::json!({
            "gamma": "blocked_by_parallelism"
        })
    );
    assert_eq!(scheduler_decisions[2]["batch"], serde_json::json!(["gamma"]));

    let started_nodes = events
        .iter()
        .filter(|event| event["event"] == "node_started")
        .map(|event| event["node_id"].as_str().expect("node id").to_string())
        .collect::<Vec<_>>();
    assert_eq!(started_nodes, vec!["alpha", "beta", "gamma"]);

    let finished_success = events
        .iter()
        .filter(|event| event["event"] == "node_finished" && event["status"] == "success")
        .map(|event| event["node_id"].as_str().expect("node id").to_string())
        .collect::<Vec<_>>();
    assert_eq!(finished_success, vec!["alpha", "beta", "gamma"]);
}

#[test]
fn runtime_events_explain_ready_and_scheduler_blocking_reasons() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"big","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"big/out"}],"resources":{"cpu":3,"mem_mb":64},"params":{"value":2}},
            {"id":"small","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"small/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"big","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"small","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                jobs: 2,
                scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
                    max_parallelism: 2,
                    cpu_budget: Some(1),
                    memory_budget_mb: None,
                    ..bijux_dag_runtime::SchedulerPolicy::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let events = read_run_events(&run_dir);
    let root_ready = events
        .iter()
        .find(|event| event["event"] == "node_ready" && event["node_id"] == "root")
        .expect("root ready");
    assert_eq!(root_ready["reason"]["code"], "root_ready");

    let downstream_ready = events
        .iter()
        .find(|event| event["event"] == "node_ready" && event["node_id"] == "small")
        .expect("downstream ready");
    assert_eq!(downstream_ready["reason"]["code"], "dependencies_satisfied");
    assert_eq!(downstream_ready["reason"]["released_by"], "root");

    let scheduler_decision = events
        .iter()
        .find(|event| {
            event["event"] == "scheduler_decision"
                && event["blocked_reasons"]["big"] == "blocked_by_cpu"
        })
        .expect("scheduler decision");
    assert_eq!(scheduler_decision["blocked_reasons"]["big"], "blocked_by_cpu");
}

#[test]
fn runtime_events_report_memory_budget_blocking_reasons() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"memory-heavy","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"memory-heavy/out"}],"resources":{"cpu":1,"mem_mb":2048},"params":{"value":2}},
            {"id":"memory-light","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"memory-light/out"}],"resources":{"cpu":1,"mem_mb":512},"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"memory-heavy","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"memory-light","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                jobs: 2,
                scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
                    max_parallelism: 2,
                    cpu_budget: Some(2),
                    memory_budget_mb: Some(1024),
                    ..bijux_dag_runtime::SchedulerPolicy::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let events = read_run_events(&run_dir);
    let scheduler_decision = events
        .iter()
        .find(|event| {
            event["event"] == "scheduler_decision"
                && event["blocked_reasons"]["memory-heavy"] == "blocked_by_memory"
        })
        .expect("scheduler decision");
    assert_eq!(scheduler_decision["blocked_reasons"]["memory-heavy"], "blocked_by_memory");
}

#[test]
fn runtime_events_report_gpu_budget_blocking_reasons() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"gpu-a","kind":"const","inputs":["in"],"tags":["gpu"],"outputs":[{"name":"out","path":"gpu-a/out"}],"params":{"value":2}},
            {"id":"gpu-b","kind":"const","inputs":["in"],"resources":{"cpu":1,"mem_mb":64,"gpu_devices":1},"outputs":[{"name":"out","path":"gpu-b/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"gpu-a","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"gpu-b","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                jobs: 3,
                gpu_device_budget: Some(1),
                scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
                    max_parallelism: 3,
                    cpu_budget: Some(3),
                    memory_budget_mb: None,
                    gpu_device_budget: Some(1),
                    ..bijux_dag_runtime::SchedulerPolicy::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("runtime run");

    let events = read_run_events(&run_dir);
    let scheduler_decision = events
        .iter()
        .find(|event| {
            event["event"] == "scheduler_decision"
                && event["blocked_reasons"].as_object().is_some_and(|blocked_reasons| {
                    blocked_reasons.values().any(|reason| reason == "blocked_by_gpu")
                })
        })
        .expect("scheduler decision");
    let blocked_reasons =
        scheduler_decision["blocked_reasons"].as_object().expect("blocked reasons object");
    let blocked_gpu = blocked_reasons
        .iter()
        .find_map(|(node_id, reason)| (reason == "blocked_by_gpu").then_some(node_id.clone()))
        .expect("gpu-blocked node");
    assert!(blocked_gpu == "gpu-a" || blocked_gpu == "gpu-b");
}

#[test]
fn partial_rerun_requires_dependency_closure_and_records_invalidation_contract() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract/out"}],"params":{"value":1}},
            {"id":"transform","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"transform/out"}],"params":{"value":2}},
            {"id":"publish","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"publish/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"extract","port":"out"},"to":{"node_id":"transform","port":"in"}},
            {"from":{"node_id":"transform","port":"out"},"to":{"node_id":"publish","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let error = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                parent_run_id: Some("run-parent".to_string()),
                selectors: SelectorSet {
                    include: vec![Selector::IdPrefix("transform".to_string())],
                    exclude: vec![],
                },
                partial_rerun_dependency_closure: false,
                ..RuntimeConfig::default()
            },
        )
        .expect_err("partial rerun without dependency closure must fail");
    assert!(error.to_string().contains("partial rerun requires dependency closure"));

    let run_dir = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                parent_run_id: Some("run-parent".to_string()),
                selectors: SelectorSet {
                    include: vec![Selector::IdPrefix("transform".to_string())],
                    exclude: vec![],
                },
                partial_rerun_dependency_closure: true,
                ..RuntimeConfig::default()
            },
        )
        .expect("closure-enabled rerun");
    let snapshot: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("run.snapshot.json")).expect("run snapshot"),
    )
    .expect("snapshot parse");
    assert_eq!(
        snapshot["partial_rerun_contract"]["selected_nodes"],
        serde_json::json!(["transform"])
    );
    assert_eq!(
        snapshot["partial_rerun_contract"]["invalidated_downstream_nodes"],
        serde_json::json!(["publish"])
    );
    assert_eq!(snapshot["partial_rerun_contract"]["stale_downstream_reuse_forbidden"], true);
}

#[test]
fn downstream_rerun_reexecutes_selected_nodes_instead_of_reusing_cache() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"extract","kind":"const","outputs":[{"name":"out","path":"extract/out"}],"params":{"value":1}},
            {"id":"transform","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"transform/out"}],"params":{"value":2}},
            {"id":"publish","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"publish/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"extract","port":"out"},"to":{"node_id":"transform","port":"in"}},
            {"from":{"node_id":"transform","port":"out"},"to":{"node_id":"publish","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");

    let original = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig { cache_mode: CacheMode::ReadWrite, ..RuntimeConfig::default() },
        )
        .expect("original run");
    let manifest: Value = serde_json::from_str(
        &fs::read_to_string(original.join("manifest.json")).expect("manifest"),
    )
    .expect("manifest parse");
    let parent_run_id = manifest["run_id"].as_str().expect("run id").to_string();

    let replay = runtime
        .run(
            &graph,
            temp.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                parent_run_id: Some(parent_run_id.clone()),
                downstream_selection_roots: vec!["transform".to_string()],
                partial_rerun_dependency_closure: false,
                ..RuntimeConfig::default()
            },
        )
        .expect("downstream replay");

    let transform_trace: Value = serde_json::from_str(
        &fs::read_to_string(replay.join("nodes/transform/trace.json")).expect("transform trace"),
    )
    .expect("transform trace parse");
    let publish_trace: Value = serde_json::from_str(
        &fs::read_to_string(replay.join("nodes/publish/trace.json")).expect("publish trace"),
    )
    .expect("publish trace parse");
    assert_eq!(transform_trace["status"], "success");
    assert_eq!(publish_trace["status"], "success");
    assert_eq!(transform_trace["replay_provenance"]["node_action"], "reexecuted");
    assert_eq!(publish_trace["replay_provenance"]["node_action"], "reexecuted");

    let snapshot: Value = serde_json::from_str(
        &fs::read_to_string(replay.join("run.snapshot.json")).expect("run snapshot"),
    )
    .expect("snapshot parse");
    assert_eq!(
        snapshot["partial_rerun_contract"]["selected_nodes"],
        serde_json::json!(["publish", "transform"])
    );
    assert_eq!(
        snapshot["partial_rerun_contract"]["invalidated_downstream_nodes"],
        serde_json::json!([])
    );
}
