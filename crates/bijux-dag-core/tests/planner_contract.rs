use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{
    graph_lowering_boundary_note, lower_graph_to_execution_plan, parse_graph_strict,
    planner_identity_for_graph, PlanOptions, PlannerError,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

fn graph_from(json: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(json).expect("parse graph")
}

#[test]
fn semantically_identical_graphs_lower_to_same_plan_identity() {
    let a = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"x","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}
          ]
        }"#,
    );

    let b = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"x2","description":"cosmetic","owners":[],"tags":[]},
          "nodes":[
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"},"tags":["cosmetic"]},
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}}
          ],
          "edges":[
            {"to":{"node_id":"b","port":"in"},"from":{"node_id":"a","port":"out"}}
          ]
        }"#,
    );

    let (a_graph, a_plan) = planner_identity_for_graph(&a).expect("identity a");
    let (b_graph, b_plan) = planner_identity_for_graph(&b).expect("identity b");
    assert_ne!(a_graph, b_graph);
    assert_eq!(a_plan, b_plan);
}

#[test]
fn selection_is_applied_after_validation_before_planning() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"s","owners":[],"tags":[]},
          "nodes":[
            {"id":"good","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"bad","kind":"const","inputs":[],"outputs":[{"name":"bad","path":"../escape"}],"params":{"value":"2"}}
          ],
          "edges":[]
        }"#,
    );
    assert!(graph.is_err(), "validation occurs before selection pruning");
    assert!(graph_lowering_boundary_note().contains("after graph validation"));
}

#[test]
fn planner_output_ordering_is_deterministic() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"ord","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}},
            {"id":"c","kind":"const","inputs":["x","y"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":"3"}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"x"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"y"}}
          ]
        }"#,
    );

    let first = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("first");
    let second = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("second");
    assert_eq!(first.ordering, second.ordering);
}

#[test]
fn unsupported_runtime_kind_is_planner_error() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"k","owners":[],"tags":[]},
          "nodes":[
            {"id":"x","kind":"custom","inputs":[],"outputs":[{"name":"out","path":"x/out"}],"params":{"value":"1"}}
          ],
          "edges":[]
        }"#,
    );

    let error =
        lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect_err("planner error");
    assert!(matches!(error, PlannerError::UnsupportedNodeKind(_)));
}

#[test]
fn fan_structures_and_selector_pruned_graphs_lower() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"shapes","owners":[],"tags":[]},
          "nodes":[
            {"id":"root","kind":"const","inputs":[],"outputs":[{"name":"out","path":"root/out"}],"params":{"value":"1"}},
            {"id":"l","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"l/out"}],"params":{"value":"l"}},
            {"id":"r","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"r/out"}],"params":{"value":"r"}},
            {"id":"join","kind":"const","inputs":["x","y"],"outputs":[{"name":"out","path":"join/out"}],"params":{"value":"join"}},
            {"id":"isolated","kind":"const","inputs":[],"outputs":[{"name":"out","path":"isolated/out"}],"params":{"value":"i"}}
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"l","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"r","port":"in"}},
            {"from":{"node_id":"l","port":"out"},"to":{"node_id":"join","port":"x"}},
            {"from":{"node_id":"r","port":"out"},"to":{"node_id":"join","port":"y"}}
          ]
        }"#,
    );

    let full = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("full plan");
    assert!(full.nodes.len() >= 5);

    let selected_nodes =
        ["root", "l", "join"].into_iter().map(str::to_string).collect::<BTreeSet<_>>();
    let pruned = lower_graph_to_execution_plan(
        &graph,
        PlanOptions { selected_nodes, ..PlanOptions::default() },
    )
    .expect("pruned plan");

    assert!(pruned.nodes.iter().all(|n| ["root", "l", "join"].contains(&n.id.as_str())));
    let join = pruned
        .nodes
        .iter()
        .find(|node| node.id == "join")
        .expect("join should remain in pruned plan");
    assert_eq!(
        join.deps,
        vec!["l".to_string()],
        "pruned plan should keep only selected dependency inputs"
    );
}

#[test]
fn execution_plan_shape_matches_schema_required_fields() {
    let graph = graph_from(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"shape","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}}
          ],
          "edges":[]
        }"#,
    );
    let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/schema/execution_plan.schema.json");
    let schema_text = fs::read_to_string(schema_path).expect("execution plan schema");
    let schema: serde_json::Value = serde_json::from_str(&schema_text).expect("schema parse");
    let required = schema
        .get("required")
        .and_then(serde_json::Value::as_array)
        .expect("schema required fields");
    let plan_value = serde_json::to_value(&plan).expect("plan value");
    for field in required.iter().filter_map(serde_json::Value::as_str) {
        assert!(
            plan_value.get(field).is_some(),
            "plan must include schema required field `{field}`"
        );
    }
}
