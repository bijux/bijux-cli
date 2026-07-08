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

use bijux_dag_runtime::{
    detect_lineage_conflicts, export_lineage_format, lineage_quality_score,
    policy_hook_allows_operation, recommended_replay_set, summarize_lineage, LineageExportFormat,
    PolicyLineageHookInput, SemanticRelationship,
};

fn load_relationships() -> Vec<SemanticRelationship> {
    let raw = std::fs::read_to_string("tests/fixtures/lineage/semantic_relationships.json")
        .expect("semantic lineage fixture");
    serde_json::from_str(&raw).expect("valid semantic lineage fixture")
}

#[test]
fn detects_conflicts_for_duplicate_edge_with_mixed_relationship_types() {
    let relationships = load_relationships();
    let conflicts = detect_lineage_conflicts(&relationships);
    assert_eq!(conflicts.len(), 1);
    assert!(conflicts[0].relation_key.contains("dataset:sales-mart:v1"));
}

#[test]
fn summarizes_lineage_with_threshold_based_compaction() {
    let relationships = load_relationships();
    let summary = summarize_lineage(&relationships, 2);
    assert_eq!(summary.total_edges, 3);
    assert!(!summary.summarized_nodes.is_empty());
}

#[test]
fn quality_score_and_policy_hook_enforce_verification_expectation() {
    let relationships = load_relationships();
    let quality = lineage_quality_score(&relationships, 2);

    let policy_input = PolicyLineageHookInput {
        relationship_count: relationships.len(),
        has_policy_dependencies: true,
        quality,
    };

    assert!(!policy_hook_allows_operation(&policy_input));
}

#[test]
fn replay_recommendation_returns_direct_upstream_dependencies() {
    let relationships = load_relationships();
    let recommendation = recommended_replay_set(&relationships, "report:weekly-sales:v1");
    assert_eq!(recommendation.target_id, "report:weekly-sales:v1");
    assert_eq!(recommendation.minimal_recompute_upstream.len(), 3);
}

#[test]
fn export_formats_map_to_expected_media_types() {
    assert_eq!(export_lineage_format(&LineageExportFormat::Json), "application/json");
    assert_eq!(export_lineage_format(&LineageExportFormat::JsonLines), "application/x-ndjson");
    assert_eq!(export_lineage_format(&LineageExportFormat::GraphMl), "application/graphml+xml");
}
