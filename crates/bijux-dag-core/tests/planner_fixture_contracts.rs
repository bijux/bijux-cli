use criterion as _;
use hex as _;
use serde as _;
use serde_json::{self, Value};
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{
    lower_graph_to_execution_plan, parse_graph_strict, PlanOptions, PlannerError,
};
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn snapshot_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("snapshots")
        .join(name)
}

fn load_graph(name: &str) -> bijux_dag_core::Graph {
    let payload = fs::read_to_string(snapshot_path(name)).expect("snapshot fixture");
    parse_graph_strict(&payload).expect("parse graph fixture")
}

#[test]
fn planner_fixtures_cover_capability_resource_retry_and_replay_oriented_graphs() {
    let resource_heavy = load_graph("resource_heavy.dag.json");
    let retry_heavy = load_graph("retry_heavy.dag.json");
    let replay_oriented = load_graph("replay_oriented.dag.json");

    let resource_plan = lower_graph_to_execution_plan(&resource_heavy, PlanOptions::default())
        .expect("resource-heavy lowers");
    assert!(!resource_plan.nodes.is_empty());

    let retry_plan =
        lower_graph_to_execution_plan(&retry_heavy, PlanOptions::default()).expect("retry lowers");
    assert!(retry_plan.nodes.iter().any(|node| node.retry.max_attempts > 0));

    let replay_plan = lower_graph_to_execution_plan(&replay_oriented, PlanOptions::default())
        .expect("replay oriented lowers");
    assert!(replay_plan.ordering.contains(&"source".to_string()));
    assert!(replay_plan.ordering.contains(&"replay_check".to_string()));
}

#[test]
fn planner_capability_restrictions_and_dependency_closure_are_enforced() {
    let replay_oriented = load_graph("replay_oriented.dag.json");

    let mut only_const = BTreeSet::new();
    only_const.insert("const".to_string());
    let err = lower_graph_to_execution_plan(
        &replay_oriented,
        PlanOptions {
            supported_kinds: only_const,
            ..PlanOptions::default()
        },
    )
    .expect_err("shell node should be rejected when shell is unsupported");
    assert!(matches!(err, PlannerError::UnsupportedNodeKind(kind) if kind == "shell"));

    let mut selected = BTreeSet::new();
    selected.insert("source".to_string());
    selected.insert("replay_check".to_string());
    let plan = lower_graph_to_execution_plan(
        &replay_oriented,
        PlanOptions {
            selected_nodes: selected,
            ..PlanOptions::default()
        },
    )
    .expect("selected plan");

    let replay_node = plan
        .nodes
        .iter()
        .find(|node| node.id == "replay_check")
        .expect("selected node present");
    assert_eq!(replay_node.deps, vec!["source".to_string()]);
}

#[test]
fn planner_json_dump_and_schema_compatibility_are_stable() {
    let graph = load_graph("diamond.dag.json");
    let first = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
    let second = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");

    let first_dump = serde_json::to_string_pretty(&first).expect("dump");
    let second_dump = serde_json::to_string_pretty(&second).expect("dump");
    assert_eq!(first_dump, second_dump, "plan dump must be stable");

    let schema_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/schema/execution_plan.schema.json");
    let schema_text = fs::read_to_string(schema_path).expect("schema file");
    let schema: Value = serde_json::from_str(&schema_text).expect("schema parse");
    let required = schema
        .get("required")
        .and_then(Value::as_array)
        .expect("required fields");

    let plan_value = serde_json::to_value(&first).expect("plan json");
    for field in required.iter().filter_map(Value::as_str) {
        assert!(
            plan_value.get(field).is_some(),
            "execution plan missing required schema field `{field}`"
        );
    }
}
