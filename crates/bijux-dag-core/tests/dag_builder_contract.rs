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
    compile_graph, dry_run_preview, lint_graph, simulate_graph, DagBuilder, Effect, NodeBuilder,
    NodeKind, NodeOutputRef, RefSpec, SubgraphDefinition, SubgraphInstance,
};
use serde_json::json;
use std::collections::BTreeMap;

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

#[test]
fn builder_authors_reusable_subgraph_instances() {
    let definition = SubgraphDefinition {
        graph: DagBuilder::new()
            .graph_input("sample_name", json!("tumor"))
            .node(
                NodeBuilder::new("extract", NodeKind::Const)
                    .output("sheet", "extract/sheet.txt")
                    .build(),
            )
            .node(
                NodeBuilder::new("align", NodeKind::Const)
                    .input("sheet")
                    .output("bam", "align/result.bam")
                    .build(),
            )
            .edge("extract", "sheet", "align", "sheet")
            .build(),
        outputs: BTreeMap::from([(
            "aligned".to_string(),
            NodeOutputRef { node_id: "align".to_string(), output_name: "bam".to_string() },
        )]),
    };
    let instance = SubgraphInstance {
        id: "tumor_align".to_string(),
        subgraph: "align_block".to_string(),
        input_bindings: BTreeMap::from([(
            "sample_name".to_string(),
            bijux_dag_core::ParamValue::Ref(RefSpec {
                graph_input: Some("sample".to_string()),
                node_output: None,
                path_var: None,
            }),
        )]),
    };

    let graph = DagBuilder::new()
        .graph_input("sample", json!("tumor"))
        .subgraph_definition("align_block", definition)
        .subgraph_instance(instance)
        .node(
            NodeBuilder::new("consume", NodeKind::Const)
                .input("bam")
                .output("out", "consume/out.txt")
                .build(),
        )
        .edge("tumor_align", "aligned", "consume", "bam")
        .build();

    let compiled = compile_graph(&graph).expect("compile reusable subgraph graph");
    assert!(compiled.normalized_graph.nodes.iter().any(|node| node.id == "tumor_align__align"));
}
