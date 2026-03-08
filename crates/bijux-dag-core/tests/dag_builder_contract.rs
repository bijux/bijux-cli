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
    dry_run_preview, lint_graph, simulate_graph, DagBuilder, Effect, NodeBuilder, NodeKind,
};
use serde_json::json;

#[test]
fn builder_constructs_graph_and_simulation_order() {
    let graph = DagBuilder::new()
        .node(
            NodeBuilder::new("extract", NodeKind::Const)
                .output("out", "extract/out.json")
                .param_literal(json!(1))
                .build(),
        )
        .node(
            NodeBuilder::new("transform", NodeKind::Shell)
                .input("in")
                .output("out", "transform/out.json")
                .effect(Effect::Filesystem)
                .build(),
        )
        .edge("extract", "out", "transform", "in")
        .build();
    let order = simulate_graph(&graph);
    assert_eq!(order, vec!["extract".to_string(), "transform".to_string()]);
}

#[test]
fn lints_graph_and_builds_dry_run_preview() {
    let graph = DagBuilder::new()
        .node(
            NodeBuilder::new("io", NodeKind::Shell)
                .output("out", "io/out.json")
                .effect(Effect::Network)
                .build(),
        )
        .build();
    let findings = lint_graph(&graph);
    assert!(!findings.is_empty());
    let preview = dry_run_preview(&graph);
    assert_eq!(preview.node_count, 1);
}
