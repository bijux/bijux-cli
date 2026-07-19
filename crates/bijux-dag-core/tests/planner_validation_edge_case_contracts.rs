use criterion as _;
use hex as _;
use serde as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{
    lower_graph_to_execution_plan, parse_graph_strict, planner_diagnostics_from_error, PlanOptions,
    PlannerError,
};
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn graph_from(payload: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(payload).expect("graph parse")
}

#[test]
fn validation_rejects_ambiguous_dependency_declarations() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"ambiguous-deps","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
            {"id":"join","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"join/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"join","port":"in"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"join","port":"in"}}
          ]
        }"#,
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1008"));
}

#[test]
fn validation_marks_unreachable_node_groups() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"unreachable-groups","owners":[],"tags":[]},
          "nodes":[
            {"id":"root","kind":"const","group":"g1","inputs":[],"outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {"id":"sink","kind":"const","group":"g1","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{"value":2}},
            {"id":"isolated","kind":"const","group":"g2","inputs":["in"],"outputs":[{"name":"out","path":"isolated/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"sink","port":"in"}}
          ]
        }"#,
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "W2001" && d.path == "/nodes/isolated"));
}

#[test]
fn validation_rejects_duplicate_node_ids_and_output_bindings() {
    let duplicate_id_graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"dup-id","owners":[],"tags":[]},
          "nodes":[
            {"id":"dup","kind":"const","inputs":[],"outputs":[{"name":"out","path":"dup/out"}],"params":{"value":1}},
            {"id":"dup","kind":"const","inputs":[],"outputs":[{"name":"out2","path":"dup/out2"}],"params":{"value":2}}
          ],
          "edges":[]
        }"#,
    );
    assert!(duplicate_id_graph.validate_with_warnings().iter().any(|d| d.code == "E1001"));

    let duplicate_output_graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"dup-output","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/path"}],"params":{"value":1}},
            {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/path"}],"params":{"value":2}}
          ],
          "edges":[]
        }"#,
    );
    assert!(duplicate_output_graph.validate_with_warnings().iter().any(|d| d.code == "E1008"));
}

#[test]
fn validation_rejects_invalid_input_binding_and_missing_environment_reference() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"invalid-bindings","owners":[],"tags":[]},
          "nodes":[
            {
              "id":"a",
              "kind":"const",
              "inputs":[],
              "outputs":[{"name":"out","path":"a/out"}],
              "params":{"from_input":{"$ref":{"graph_input":"MISSING_ENV"}}}
            },
            {
              "id":"b",
              "kind":"const",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"value":2}
            }
          ],
          "edges":[
            {"from":{"node_id":"a","port":"missing-port"},"to":{"node_id":"b","port":"in"}}
          ]
        }"#,
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1003"));
    assert!(diags.iter().any(|d| d.code == "E1020"));
}

#[test]
fn validation_rejects_unsupported_execution_mode_combinations_and_invalid_tag_filters() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"bad tags","owners":[],"tags":["not valid"]},
          "nodes":[
            {
              "id":"net",
              "kind":"shell",
              "inputs":[],
              "outputs":[{"name":"out","path":"net/out"}],
              "effects":["network","filesystem"],
              "retry":{"max_attempts":3,"backoff_ms":10},
              "params":{"argv":["echo","hi"]},
              "tags":["also bad"]
            }
          ],
          "edges":[]
        }"#,
    );
    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1011"));
    assert!(diags.iter().any(|d| d.code == "E1026"));
}

#[test]
fn planner_inclusion_exclusion_and_capability_diagnostics_are_stable() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"planner-selection","owners":[],"tags":[]},
          "nodes":[
            {"id":"source","kind":"const","inputs":[],"outputs":[{"name":"out","path":"source/out"}],"params":{"value":1}},
            {"id":"transform","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"transform/out"}],"effects":["filesystem"],"params":{"argv":["echo","x"]}},
            {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{"value":2}}
          ],
          "edges":[
            {"from":{"node_id":"source","port":"out"},"to":{"node_id":"transform","port":"in"}},
            {"from":{"node_id":"transform","port":"out"},"to":{"node_id":"sink","port":"in"}}
          ]
        }"#,
    );

    let mut selected = BTreeSet::new();
    selected.insert("source".to_string());
    selected.insert("sink".to_string());
    let pruned = lower_graph_to_execution_plan(
        &graph,
        PlanOptions { selected_nodes: selected, ..PlanOptions::default() },
    )
    .expect("pruned plan");
    assert_eq!(pruned.ordering, vec!["sink".to_string(), "source".to_string()]);

    let mut supported = BTreeSet::new();
    supported.insert("const".to_string());
    let capability_err = lower_graph_to_execution_plan(
        &graph,
        PlanOptions { supported_kinds: supported, ..PlanOptions::default() },
    )
    .expect_err("shell should be rejected");
    assert!(matches!(
        capability_err,
        PlannerError::UnsupportedNodeKinds(ref nodes)
            if nodes == &vec!["transform:shell".to_string()]
    ));

    let planner_diags = planner_diagnostics_from_error(&capability_err);
    assert_eq!(planner_diags[0].id, "P4013");
    assert_eq!(planner_diags[0].node_id.as_deref(), Some("transform"));
}

#[test]
fn planner_plan_dump_is_deterministic_and_schema_compatible_for_replay_oriented_graph() {
    let replay_fixture = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/planner/replay_oriented.dag.json");
    let payload = fs::read_to_string(replay_fixture).expect("fixture");
    let graph = parse_graph_strict(&payload).expect("parse fixture");

    let first = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
    let second = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");

    assert_eq!(
        serde_json::to_string_pretty(&first).expect("dump"),
        serde_json::to_string_pretty(&second).expect("dump")
    );

    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/schema/execution_plan.schema.json");
    let schema_text = fs::read_to_string(schema_path).expect("schema");
    let schema: Value = serde_json::from_str(&schema_text).expect("schema json");
    let required = schema["required"].as_array().expect("required");
    let plan_json = serde_json::to_value(first).expect("value");
    for field in required.iter().filter_map(Value::as_str) {
        assert!(plan_json.get(field).is_some(), "missing required field: {field}");
    }
}

#[test]
fn validation_rejects_branch_contracts_that_do_not_match_conditional_edges() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"branch-contract-mismatch","owners":[],"tags":[]},
          "nodes":[
            {
              "id":"decide",
              "kind":"shell",
              "semantic_kind":"branch",
              "inputs":["in"],
              "outputs":[{"name":"decision","path":"decide/decision.txt"}],
              "effects":["filesystem"],
              "params":{"argv":["echo","left"]},
              "branch":{
                "decisions":["left","right"],
                "default_decision":"left",
                "decision_output":"decision"
              }
            },
            {"id":"left","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":1}},
            {"id":"join","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"join/out"}],"params":{"value":2},"trigger_rule":"any_success"}
          ],
          "edges":[
            {"id":"left-only","kind":"conditional","decision":"left","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"left","port":"in"}},
            {"from":{"node_id":"decide","port":"decision"},"to":{"node_id":"join","port":"in"}}
          ]
        }"#,
    );

    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1028" && d.message.contains("right")));
    assert!(diags
        .iter()
        .any(|d| d.code == "E1030" && d.message.contains("must only drive conditional edges")));
}

#[test]
fn validation_rejects_conditional_targets_with_all_success_trigger_rule() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"conditional-trigger-mismatch","owners":[],"tags":[]},
          "nodes":[
            {
              "id":"decide",
              "kind":"shell",
              "semantic_kind":"branch",
              "inputs":["in"],
              "outputs":[{"name":"decision","path":"decide/decision.txt"}],
              "effects":["filesystem"],
              "params":{"argv":["echo","left"]},
              "branch":{
                "decisions":["left"],
                "default_decision":"left",
                "decision_output":"decision"
              }
            },
            {"id":"sink","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"sink/out"}],"params":{"value":1},"trigger_rule":"all_success"}
          ],
          "edges":[
            {"id":"left-branch","kind":"conditional","decision":"left","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"sink","port":"in"}}
          ]
        }"#,
    );

    let diags = graph.validate_with_warnings();
    assert!(diags.iter().any(|d| d.code == "E1030" && d.path == "/nodes/sink/trigger_rule"));
}
