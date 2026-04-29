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
use bijux_dag_runtime::{Runtime, RuntimeConfig};
use serde_json::Value;
use std::fs;

fn branch_graph_json(decision: &str, default_decision: Option<&str>) -> String {
    let default_fragment = default_decision
        .map(|value| format!(r#","default_decision":"{value}""#))
        .unwrap_or_default();
    format!(
        r#"{{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {{"id":"seed","kind":"const","inputs":[],"outputs":[{{"name":"out","path":"seed/out"}}],"params":{{"value":1}}}},
            {{
              "id":"decide",
              "kind":"const",
              "semantic_kind":"branch",
              "inputs":["in"],
              "outputs":[{{"name":"decision","path":"decide/decision.txt"}}],
              "params":{{"value":"{decision}"}},
              "branch":{{"decisions":["left","right"]{default_fragment},"decision_output":"decision"}}
            }},
            {{"id":"left","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"left/out"}}],"params":{{"value":"left"}},"trigger_rule":"any_success"}},
            {{"id":"right","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"right/out"}}],"params":{{"value":"right"}},"trigger_rule":"any_success"}},
            {{"id":"join","kind":"const","inputs":["lhs"],"outputs":[{{"name":"out","path":"join/out"}}],"params":{{"value":"join"}}}}
          ],
          "edges":[
            {{"id":"seed-to-decide","from":{{"node_id":"seed","port":"out"}},"to":{{"node_id":"decide","port":"in"}}}},
            {{"id":"branch-left","kind":"conditional","decision":"left","from":{{"node_id":"decide","port":"decision"}},"to":{{"node_id":"left","port":"in"}}}},
            {{"id":"branch-right","kind":"conditional","decision":"right","from":{{"node_id":"decide","port":"decision"}},"to":{{"node_id":"right","port":"in"}}}},
            {{"id":"left-to-join","kind":"control","from":{{"node_id":"left","port":"out"}},"to":{{"node_id":"join","port":"lhs"}}}}
          ]
        }}"#
    )
}

fn read_trace(run_dir: &std::path::Path, node_id: &str) -> Value {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("nodes").join(node_id).join("trace.json"))
            .expect("read trace"),
    )
    .expect("parse trace")
}

fn read_events(run_dir: &std::path::Path) -> Vec<Value> {
    serde_json::from_str(
        &fs::read_to_string(run_dir.join("observability.events.json")).expect("read events"),
    )
    .expect("parse events")
}

#[test]
fn runtime_executes_selected_branch_and_skips_unselected_path() {
    let graph = parse_graph_strict(&branch_graph_json("left", Some("left"))).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let decide = read_trace(&run_dir, "decide");
    let left = read_trace(&run_dir, "left");
    let right = read_trace(&run_dir, "right");
    let join = read_trace(&run_dir, "join");

    assert_eq!(decide["status"], "success");
    assert_eq!(decide["branch_decision"], "left");
    assert_eq!(left["status"], "success");
    assert_eq!(right["status"], "skipped");
    assert_eq!(right["skip_reason"]["reason"], "branch_decision_not_selected");
    assert_eq!(join["status"], "success");

    let events = read_events(&run_dir);
    assert!(events.iter().any(|event| {
        event["name"] == "branch_decision_selected"
            && event["node_id"] == "decide"
            && event["details"]["decision"] == "left"
    }));
}

#[test]
fn runtime_prunes_left_path_when_branch_selects_right() {
    let graph = parse_graph_strict(&branch_graph_json("right", Some("left"))).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let decide = read_trace(&run_dir, "decide");
    let left = read_trace(&run_dir, "left");
    let right = read_trace(&run_dir, "right");
    let join = read_trace(&run_dir, "join");

    assert_eq!(decide["branch_decision"], "right");
    assert_eq!(left["status"], "skipped");
    assert_eq!(right["status"], "success");
    assert_eq!(join["status"], "skipped");
    assert_eq!(join["skip_reason"]["reason"], "branch_decision_not_selected");
}

#[test]
fn runtime_uses_branch_default_when_node_outputs_unknown_decision() {
    let graph =
        parse_graph_strict(&branch_graph_json("unknown", Some("left"))).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let decide = read_trace(&run_dir, "decide");
    let left = read_trace(&run_dir, "left");
    let right = read_trace(&run_dir, "right");

    assert_eq!(decide["branch_decision"], "left");
    assert_eq!(left["status"], "success");
    assert_eq!(right["status"], "skipped");

    let events = read_events(&run_dir);
    assert!(events.iter().any(|event| {
        event["name"] == "branch_decision_selected"
            && event["node_id"] == "decide"
            && event["details"]["used_default"] == true
    }));
}
