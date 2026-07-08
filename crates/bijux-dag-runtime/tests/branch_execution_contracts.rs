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
use bijux_dag_runtime::{Runtime, RuntimeConfig, Selector, SelectorSet};
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

fn branch_join_graph_json(decision: &str, trigger_rule: &str) -> String {
    format!(
        r#"{{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {{"id":"seed","kind":"const","outputs":[{{"name":"out","path":"seed/out"}}],"params":{{"value":1}}}},
            {{
              "id":"decide",
              "kind":"const",
              "semantic_kind":"branch",
              "inputs":["in"],
              "outputs":[{{"name":"decision","path":"decide/decision.txt"}}],
              "params":{{"value":"{decision}"}},
              "branch":{{"decisions":["left","right"],"decision_output":"decision"}}
            }},
            {{"id":"left","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"left/out"}}],"params":{{"value":"left"}},"trigger_rule":"any_success"}},
            {{"id":"right","kind":"const","inputs":["in"],"outputs":[{{"name":"out","path":"right/out"}}],"params":{{"value":"right"}},"trigger_rule":"any_success"}},
            {{"id":"join","kind":"const","inputs":["lhs","rhs"],"outputs":[{{"name":"out","path":"join/out"}}],"params":{{"value":"join"}},"trigger_rule":"{trigger_rule}"}}
          ],
          "edges":[
            {{"id":"seed-to-decide","from":{{"node_id":"seed","port":"out"}},"to":{{"node_id":"decide","port":"in"}}}},
            {{"id":"branch-left","kind":"conditional","decision":"left","from":{{"node_id":"decide","port":"decision"}},"to":{{"node_id":"left","port":"in"}}}},
            {{"id":"branch-right","kind":"conditional","decision":"right","from":{{"node_id":"decide","port":"decision"}},"to":{{"node_id":"right","port":"in"}}}},
            {{"id":"left-to-join","kind":"control","from":{{"node_id":"left","port":"out"}},"to":{{"node_id":"join","port":"lhs"}}}},
            {{"id":"right-to-join","kind":"control","from":{{"node_id":"right","port":"out"}},"to":{{"node_id":"join","port":"rhs"}}}}
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

fn run_id_from_dir(run_dir: &std::path::Path) -> String {
    run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .and_then(|value| value.strip_prefix("run-"))
        .expect("run id")
        .to_string()
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

#[test]
fn runtime_runs_branch_join_with_any_success_after_unselected_path_is_skipped() {
    let graph =
        parse_graph_strict(&branch_join_graph_json("left", "any_success")).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let left = read_trace(&run_dir, "left");
    let right = read_trace(&run_dir, "right");
    let join = read_trace(&run_dir, "join");

    assert_eq!(left["status"], "success");
    assert_eq!(right["status"], "skipped");
    assert_eq!(join["status"], "success");
}

#[test]
fn runtime_runs_branch_join_with_all_done_after_unselected_path_is_skipped() {
    let graph =
        parse_graph_strict(&branch_join_graph_json("right", "all_done")).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");
    let run_dir =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("run branch dag");

    let left = read_trace(&run_dir, "left");
    let right = read_trace(&run_dir, "right");
    let join = read_trace(&run_dir, "join");

    assert_eq!(left["status"], "skipped");
    assert_eq!(right["status"], "success");
    assert_eq!(join["status"], "success");
}

#[test]
fn runtime_replays_parent_branch_decision_for_filtered_branch_nodes() {
    let graph = parse_graph_strict(&branch_graph_json("left", Some("left"))).expect("parse graph");
    let runtime = Runtime::new();
    let out_dir = tempfile::tempdir().expect("tempdir");

    let original =
        runtime.run(&graph, out_dir.path(), RuntimeConfig::default()).expect("original run");
    let parent_run_id = run_id_from_dir(&original);

    let replay = runtime
        .run(
            &graph,
            out_dir.path(),
            RuntimeConfig {
                parent_run_id: Some(parent_run_id.clone()),
                selectors: SelectorSet {
                    include: vec![Selector::IdPrefix("seed".to_string())],
                    exclude: vec![],
                },
                partial_rerun_dependency_closure: true,
                ..RuntimeConfig::default()
            },
        )
        .expect("replay run");

    let events = read_events(&replay);
    assert!(events.iter().any(|event| {
        event["name"] == "branch_decision_replayed"
            && event["node_id"] == "decide"
            && event["details"]["decision"] == "left"
            && event["details"]["source_run_id"] == parent_run_id
    }));
}
