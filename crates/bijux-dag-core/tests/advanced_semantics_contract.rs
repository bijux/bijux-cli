#![cfg(feature = "experimental-public-api")]

use criterion as _;
use hex as _;
use serde as _;
use serde_json as _;
use serde_yaml as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::experimental::{
    classify_compatibility, complexity_score, enforce_late_binding_immutability, explain_graph,
    migration_patch, normalize_semantic_graph, semantic_diff, static_analysis,
    CompatibilityClassification, DynamicEdgeExpansionRule, LateBindingRule, SemanticDiffClass,
};

fn read_fixture(name: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read fixture {}: {err}", path.display()))
}

#[test]
fn validates_semantic_normalization_and_late_binding_rules() {
    let text = read_fixture("partitioned_map_reduce.json");
    let graph = bijux_dag_core::parse_graph_strict(&text).expect("fixture parse should pass");
    let normalized = normalize_semantic_graph(
        &graph,
        vec!["partition-contract:v1".to_string()],
        vec![DynamicEdgeExpansionRule {
            rule_id: "expand-partitions".to_string(),
            source_node_id: "map".to_string(),
            deterministic: true,
            snapshot_captured: true,
        }],
    )
    .expect("normalization should pass");
    assert_eq!(normalized.graph.spec, bijux_dag_core::SPEC_VERSION);
    let late_binding = vec![LateBindingRule {
        binding_name: "runtime-token".to_string(),
        allowed_pre_compile: true,
        allowed_post_compile: false,
    }];
    assert!(enforce_late_binding_immutability(&late_binding).is_ok());
}

#[test]
fn computes_semantic_diff_static_analysis_and_complexity() {
    let before = bijux_dag_core::parse_graph_strict(&read_fixture("templated_composition.json"))
        .expect("before graph parse should pass");
    let after = bijux_dag_core::parse_graph_strict(&read_fixture("conditional_branch_join.json"))
        .expect("after graph parse should pass");
    let diff = semantic_diff(&before, &after);
    assert_eq!(diff.class, SemanticDiffClass::Topology);
    assert_eq!(classify_compatibility(&diff), CompatibilityClassification::CacheBreaking);
    let analysis = static_analysis(&after);
    assert!(analysis.noop_join_nodes.iter().any(|n| n == "join"));
    let complexity = complexity_score(&after);
    assert!(complexity.score > 0.0);
    let explain = explain_graph(&after);
    assert!(!explain.node_explanations.is_empty());
    let migration = migration_patch("bijux-dag/v0.1", "bijux-dag/v0.2");
    assert_eq!(migration.from_spec, "bijux-dag/v0.1");
}
