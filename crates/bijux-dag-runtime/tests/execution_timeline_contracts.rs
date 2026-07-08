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
use std::path::Path;

fn read_timeline(run_dir: &Path) -> Vec<Value> {
    serde_json::from_str::<Value>(
        &fs::read_to_string(run_dir.join("observability.timeline.json")).expect("timeline"),
    )
    .expect("parse timeline")["entries"]
        .as_array()
        .expect("timeline entries")
        .clone()
}

fn timeline_index(entries: &[Value], label: &str, node_id: Option<&str>) -> usize {
    entries
        .iter()
        .position(|entry| {
            entry["label"] == label
                && node_id.map(|expected| entry["node_id"] == expected).unwrap_or(true)
        })
        .unwrap_or_else(|| panic!("missing timeline entry {label} for {node_id:?}"))
}

fn const_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "const1",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "value.txt"}],
                "params": {"value": "hello"}
            }
        ],
        "edges": []
    })
    .to_string()
}

fn failure_and_completion_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "fail",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "fail.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "printf broken >&2; exit 7"]},
                "effects": ["filesystem"]
            },
            {
                "id": "independent",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "value", "path": "independent.txt"}],
                "params": {"value": "ready"}
            }
        ],
        "edges": []
    })
    .to_string()
}

fn branch_skip_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "inputs": [],
                "outputs": [{"name": "out", "path": "seed/out"}],
                "params": {"value": 1}
            },
            {
                "id": "decide",
                "kind": "const",
                "semantic_kind": "branch",
                "inputs": ["in"],
                "outputs": [{"name": "decision", "path": "decide/decision.txt"}],
                "params": {"value": "left"},
                "branch": {
                    "decisions": ["left", "right"],
                    "default_decision": "left",
                    "decision_output": "decision"
                }
            },
            {
                "id": "left",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "left/out"}],
                "params": {"value": "left"},
                "trigger_rule": "any_success"
            },
            {
                "id": "right",
                "kind": "const",
                "inputs": ["in"],
                "outputs": [{"name": "out", "path": "right/out"}],
                "params": {"value": "right"},
                "trigger_rule": "any_success"
            }
        ],
        "edges": [
            {
                "from": {"node_id": "seed", "port": "out"},
                "to": {"node_id": "decide", "port": "in"}
            },
            {
                "id": "branch-left",
                "kind": "conditional",
                "decision": "left",
                "from": {"node_id": "decide", "port": "decision"},
                "to": {"node_id": "left", "port": "in"}
            },
            {
                "id": "branch-right",
                "kind": "conditional",
                "decision": "right",
                "from": {"node_id": "decide", "port": "decision"},
                "to": {"node_id": "right", "port": "in"}
            }
        ]
    })
    .to_string()
}

fn cacheable_shell_graph() -> String {
    json!({
        "spec": "bijux-dag/v0.1",
        "nodes": [
            {
                "id": "node",
                "kind": "shell",
                "inputs": [],
                "outputs": [{"name": "value", "path": "value.txt"}],
                "params": {"argv": ["/bin/sh", "-c", "printf '%s' ok > ../outputs/value.txt"]},
                "effects": ["filesystem"]
            }
        ],
        "edges": []
    })
    .to_string()
}

#[test]
fn runtime_timeline_orders_core_lifecycle_events_for_successful_nodes() {
    let graph = parse_graph_strict(&const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");

    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");
    let timeline = read_timeline(&run_path);

    let run_started = timeline_index(&timeline, "run_started", None);
    let ready = timeline_index(&timeline, "node_ready", Some("const1"));
    let scheduled = timeline_index(&timeline, "node_scheduled", Some("const1"));
    let started = timeline_index(&timeline, "node_started", Some("const1"));
    let completed = timeline_index(&timeline, "node_completed", Some("const1"));
    let run_completed = timeline_index(&timeline, "run_completed", None);

    assert!(run_started < ready);
    assert!(ready < scheduled);
    assert!(scheduled < started);
    assert!(started < completed);
    assert!(completed < run_completed);
}

#[test]
fn runtime_timeline_records_failed_and_completed_terminal_nodes_in_one_stream() {
    let graph = parse_graph_strict(&failure_and_completion_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");

    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");
    let timeline = read_timeline(&run_path);

    let failed = timeline_index(&timeline, "node_failed", Some("fail"));
    let completed = timeline_index(&timeline, "node_completed", Some("independent"));
    let run_completed = timeline_index(&timeline, "run_completed", None);

    assert!(failed < run_completed);
    assert!(completed < run_completed);
}

#[test]
fn runtime_timeline_records_skipped_terminal_nodes_for_pruned_branches() {
    let graph = parse_graph_strict(&branch_skip_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp");

    let run_path = runtime.run(&graph, out.path(), RuntimeConfig::default()).expect("run");
    let timeline = read_timeline(&run_path);

    let skipped = timeline_index(&timeline, "node_skipped", Some("right"));
    let run_completed = timeline_index(&timeline, "run_completed", None);
    assert!(skipped < run_completed);
}

#[test]
fn runtime_timeline_marks_cached_nodes_without_fabricating_execution_start() {
    let graph = parse_graph_strict(&cacheable_shell_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let out = tempfile::tempdir().expect("temp out");
    let cache = tempfile::tempdir().expect("temp cache");

    let config = || RuntimeConfig {
        cache_mode: CacheMode::ReadWrite,
        cache_dir: Some(cache.path().to_path_buf()),
        ..RuntimeConfig::default()
    };

    runtime.run(&graph, out.path(), config()).expect("seed cache");
    let cached_run = runtime.run(&graph, out.path(), config()).expect("cache hit");
    let timeline = read_timeline(&cached_run);

    let ready = timeline_index(&timeline, "node_ready", Some("node"));
    let scheduled = timeline_index(&timeline, "node_scheduled", Some("node"));
    let cached = timeline_index(&timeline, "node_cached", Some("node"));
    let run_completed = timeline_index(&timeline, "run_completed", None);

    assert!(ready < scheduled);
    assert!(scheduled < cached);
    assert!(cached < run_completed);
    assert!(
        !timeline.iter().any(|entry| {
            entry["label"] == "node_started" && entry["node_id"] == "node"
        }),
        "cache hits should not fabricate node_started timeline entries"
    );
}
