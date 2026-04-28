use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::prelude::{
    canonical_json, compile_graph_strict, lower_graph_to_execution_plan, parse_graph_strict,
    planner_identity_for_graph, validate_graph, Graph, SPEC_VERSION,
};

fn simple_graph() -> String {
    serde_json::json!({
        "spec": SPEC_VERSION,
        "nodes": [
            {
                "id": "seed",
                "kind": "const",
                "outputs": [{"name": "value", "path": "seed.json"}],
                "params": {"value": {"ok": true}}
            }
        ],
        "edges": []
    })
    .to_string()
}

#[test]
fn prelude_exposes_strict_parse_validate_and_lower_contracts() {
    let graph_text = simple_graph();
    let graph = parse_graph_strict(&graph_text).expect("parse");
    let validated = validate_graph(&graph);
    assert!(validated.iter().all(|diag| diag.severity != bijux_dag_core::Severity::Error));

    let plan = lower_graph_to_execution_plan(&graph, Default::default()).expect("lower");
    assert_eq!(plan.nodes.len(), 1);

    let compiled = compile_graph_strict(&graph_text).expect("compile");
    assert_eq!(compiled.normalized_graph.spec, SPEC_VERSION);
}

#[test]
fn prelude_identity_helpers_match_graph_contract_surface() {
    let graph: Graph = parse_graph_strict(&simple_graph()).expect("parse");
    let canonical: serde_json::Value =
        serde_json::from_str(&canonical_json(&graph).expect("canonical json")).expect("json");
    assert_eq!(canonical["spec"], SPEC_VERSION);

    let (graph_fp, planner_fp) = planner_identity_for_graph(&graph).expect("planner identity");
    assert!(!graph_fp.is_empty());
    assert!(!planner_fp.is_empty());
}
