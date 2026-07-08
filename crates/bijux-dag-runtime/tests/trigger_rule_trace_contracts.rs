use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{CacheMode, Runtime, RuntimeConfig};
use serde_json::{json, Value};
use std::fs;

fn read_trace(run_dir: &std::path::Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace")
}

fn parent_statuses(trace: &Value) -> Vec<(String, String)> {
    let mut statuses = trace["trigger_evaluation"]["parent_statuses"]
        .as_array()
        .expect("parent statuses")
        .iter()
        .map(|status| {
            (
                status["node_id"].as_str().unwrap_or_default().to_string(),
                status["status"].as_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<Vec<_>>();
    statuses.sort();
    statuses
}

fn assert_trigger_evaluation(
    trace: &Value,
    trigger_rule: &str,
    satisfied: bool,
    reason: &str,
    expected_statuses: &[(&str, &str)],
) {
    assert_eq!(trace["trigger_evaluation"]["trigger_rule"], trigger_rule);
    assert_eq!(trace["trigger_evaluation"]["satisfied"], satisfied);
    assert_eq!(trace["trigger_evaluation"]["reason"], reason);

    let mut expected = expected_statuses
        .iter()
        .map(|(node_id, status)| ((*node_id).to_string(), (*status).to_string()))
        .collect::<Vec<_>>();
    expected.sort();
    assert_eq!(parent_statuses(trace), expected);
}

fn branch_join_graph(trigger_rule: &str, decision: &str) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "outputs": [{"name": "out", "path": "seed/out.txt"}],
                "params": {"value": 1}
            },
            {
                "id": "decide",
                "kind": "const",
                "semantic_kind": "branch",
                "inputs": ["in"],
                "outputs": [{"name": "decision", "path": "decide/decision.txt"}],
                "params": {"value": decision},
                "branch": {"decisions": ["left", "right"], "decision_output": "decision"}
            },
            {
                "id": "left",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "left/out.txt"}],
                "params": {"value": "left"},
                "trigger_rule": "any_success"
            },
            {
                "id": "right",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "right/out.txt"}],
                "params": {"value": "right"},
                "trigger_rule": "any_success"
            },
            {
                "id": "join",
                "kind": "const",
                "inputs": ["left_gate", "right_gate"],
                "outputs": [{"name": "out", "path": "join/out.txt"}],
                "params": {"value": "join"},
                "trigger_rule": trigger_rule
            }
        ],
        "edges": [
            {"from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "decide", "port": "in"}},
            {"kind": "conditional", "decision": "left", "from": {"node_id": "decide", "port": "decision"}, "to": {"node_id": "left", "port": "in"}},
            {"kind": "conditional", "decision": "right", "from": {"node_id": "decide", "port": "decision"}, "to": {"node_id": "right", "port": "in"}},
            {"kind": "control", "from": {"node_id": "left", "port": "out"}, "to": {"node_id": "join", "port": "left_gate"}},
            {"kind": "control", "from": {"node_id": "right", "port": "out"}, "to": {"node_id": "join", "port": "right_gate"}}
        ]
    })
    .to_string()
}

fn failure_join_graph(trigger_rule: &str) -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "outputs": [{"name": "out", "path": "seed/out.txt"}],
                "params": {"value": 1}
            },
            {
                "id": "steady",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "steady/out.txt"}],
                "params": {"value": "steady"}
            },
            {
                "id": "fragile",
                "kind": "shell",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "fragile/out.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "exit 1"]},
                "effects": ["filesystem"]
            },
            {
                "id": "join",
                "kind": "const",
                "inputs": ["steady_gate", "fragile_gate"],
                "outputs": [{"name": "out", "path": "join/out.txt"}],
                "params": {"value": "join"},
                "trigger_rule": trigger_rule
            }
        ],
        "edges": [
            {"from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "steady", "port": "in"}},
            {"from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "fragile", "port": "in"}},
            {"kind": "control", "from": {"node_id": "steady", "port": "out"}, "to": {"node_id": "join", "port": "steady_gate"}},
            {"kind": "control", "from": {"node_id": "fragile", "port": "out"}, "to": {"node_id": "join", "port": "fragile_gate"}}
        ]
    })
    .to_string()
}

fn cached_trigger_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "outputs": [{"name": "out", "path": "seed/out.txt"}],
                "params": {"value": "seed"}
            },
            {
                "id": "consume",
                "kind": "const",
                "inputs": ["seed_gate"],
                "outputs": [{"name": "out", "path": "consume/out.txt"}],
                "params": {"value": "consume"},
                "trigger_rule": "all_success"
            }
        ],
        "edges": [
            {"kind": "control", "from": {"node_id": "seed", "port": "out"}, "to": {"node_id": "consume", "port": "seed_gate"}}
        ]
    })
    .to_string()
}

fn retry_trigger_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "worker",
                "kind": "shell",
                "outputs": [{"name": "out", "path": "out.txt"}],
                "params": {
                    "argv": [
                        "/bin/sh",
                        "-c",
                        "if [ ! -f marker ]; then touch marker; exit 1; fi; printf 'ok' > ../outputs/out.txt"
                    ]
                },
                "retry": {
                    "max_attempts": 1,
                    "backoff_ms": 0
                },
                "effects": ["filesystem"]
            },
            {
                "id": "consume",
                "kind": "const",
                "inputs": ["worker_gate"],
                "outputs": [{"name": "out", "path": "consume/out.txt"}],
                "params": {"value": "consume"},
                "trigger_rule": "all_success"
            }
        ],
        "edges": [
            {"kind": "control", "from": {"node_id": "worker", "port": "out"}, "to": {"node_id": "consume", "port": "worker_gate"}}
        ]
    })
    .to_string()
}

#[test]
fn runtime_records_none_failed_trace_for_branch_skip() {
    let graph = parse_graph_strict(&branch_join_graph("none_failed", "left")).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let join = read_trace(&run_dir, "join");
    assert_eq!(join["status"], "success");
    assert_trigger_evaluation(
        &join,
        "none_failed",
        true,
        "requires upstream completion without failures",
        &[("left", "success"), ("right", "skipped")],
    );
}

#[test]
fn runtime_blocks_none_failed_after_upstream_failure_and_records_trace_decision() {
    let graph = parse_graph_strict(&failure_join_graph("none_failed")).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run failure dag");

    let join = read_trace(&run_dir, "join");
    assert_eq!(join["status"], "failed");
    assert_eq!(join["failure"]["code"], "UPSTREAM_FAILED");
    assert_trigger_evaluation(
        &join,
        "none_failed",
        false,
        "requires upstream completion without failures",
        &[("fragile", "failed"), ("steady", "success")],
    );
}

#[test]
fn runtime_records_all_done_trace_when_failed_upstream_is_allowed() {
    let graph = parse_graph_strict(&failure_join_graph("all_done")).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run failure dag");

    let join = read_trace(&run_dir, "join");
    assert_eq!(join["status"], "success");
    assert_trigger_evaluation(
        &join,
        "all_done",
        true,
        "accepts any terminal upstream status",
        &[("fragile", "failed"), ("steady", "success")],
    );
}

#[test]
fn runtime_records_cached_upstream_as_all_success_input() {
    let graph = parse_graph_strict(&cached_trigger_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let cache_dir = tempfile::tempdir().expect("cache");

    let _ = runtime
        .run(
            &graph,
            out_dir.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache_dir.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("seed cache");

    let run_dir = runtime
        .run(
            &graph,
            out_dir.path(),
            RuntimeConfig {
                cache_mode: CacheMode::ReadWrite,
                cache_dir: Some(cache_dir.path().to_path_buf()),
                ..RuntimeConfig::default()
            },
        )
        .expect("cache hit");

    let seed = read_trace(&run_dir, "seed");
    let consume = read_trace(&run_dir, "consume");
    assert_eq!(seed["status"], "cached");
    assert_eq!(consume["status"], "cached");
    assert_trigger_evaluation(
        &consume,
        "all_success",
        true,
        "requires every upstream to complete in success or cached status",
        &[("seed", "cached")],
    );
}

#[test]
fn runtime_records_retry_success_before_all_success_downstream() {
    let graph = parse_graph_strict(&retry_trigger_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run retry dag");

    let worker = read_trace(&run_dir, "worker");
    let consume = read_trace(&run_dir, "consume");
    assert_eq!(worker["status"], "success");
    assert_eq!(worker["attempt"], 2);
    assert_eq!(consume["status"], "success");
    assert_trigger_evaluation(
        &consume,
        "all_success",
        true,
        "requires every upstream to complete in success or cached status",
        &[("worker", "success")],
    );
}
