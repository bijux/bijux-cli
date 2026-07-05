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
                "to_state": "eligible",
                "cause": "scheduler_eligible",
                "unix_ms": trace["lifecycle_transitions"][0]["unix_ms"],
            },
            {
                "from_state": "eligible",
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
