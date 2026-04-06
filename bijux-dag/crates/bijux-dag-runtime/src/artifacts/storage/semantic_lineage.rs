use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SemanticDependencyClass {
    Data,
    Control,
    Quality,
    Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineageConfidence {
    ExactDeclared,
    InferredHigh,
    InferredMedium,
    InferredLow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactSemanticTag {
    Model,
    Dataset,
    Report,
    Checkpoint,
    MetricBundle,
    ComplianceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactRelationshipType {
    DerivedFrom,
    ValidatedBy,
    ApprovedBy,
    SupersededBy,
    PromotedFrom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticRelationship {
    pub from_id: String,
    pub to_id: String,
    pub dependency_class: SemanticDependencyClass,
    pub relationship_type: ArtifactRelationshipType,
    pub confidence: LineageConfidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldLevelLineageHook {
    pub dataset_id: String,
    pub field_name: String,
    pub upstream_fields: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossRunLineageStitch {
    pub schedule_name: String,
    pub run_ids: Vec<String>,
    pub stitched_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageSummaryNode {
    pub node_id: String,
    pub child_count: usize,
    pub hidden_edges: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageSummary {
    pub total_nodes: usize,
    pub total_edges: usize,
    pub summarized_nodes: Vec<LineageSummaryNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageImpactReport {
    pub changed_input: String,
    pub affected_runs: BTreeSet<String>,
    pub affected_datasets: BTreeSet<String>,
    pub affected_artifacts: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReverseImpactReport {
    pub result_id: String,
    pub upstream_trust_inputs: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum LineageExportFormat {
    Json,
    JsonLines,
    GraphMl,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageMaterializationRule {
    pub cache_enabled: bool,
    pub max_cache_age_minutes: u32,
    pub invalidate_on_relationship_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetentionProtectionRule {
    pub protect_promoted_ancestry: bool,
    pub protect_policy_dependency_paths: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayRecommendation {
    pub target_id: String,
    pub minimal_recompute_upstream: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticLineageExplain {
    pub subject_id: String,
    pub explanation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageConflict {
    pub relation_key: String,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageReconciliationPlan {
    pub import_bundle_id: String,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageQualityScore {
    pub completeness: u8,
    pub exactness: u8,
    pub verification_coverage: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyLineageHookInput {
    pub relationship_count: usize,
    pub has_policy_dependencies: bool,
    pub quality: LineageQualityScore,
}

pub fn summarize_lineage(
    relationships: &[SemanticRelationship],
    summarize_threshold: usize,
) -> LineageSummary {
    let mut child_counts: BTreeMap<String, usize> = BTreeMap::new();
    for relation in relationships {
        *child_counts.entry(relation.from_id.clone()).or_default() += 1;
    }

    let mut summarized_nodes = Vec::new();
    for (node_id, child_count) in child_counts {
        if child_count >= summarize_threshold {
            summarized_nodes.push(LineageSummaryNode {
                node_id,
                child_count,
                hidden_edges: child_count.saturating_sub(summarize_threshold),
            });
        }
    }

    LineageSummary {
        total_nodes: relationships
            .iter()
            .flat_map(|relation| [relation.from_id.clone(), relation.to_id.clone()])
            .collect::<BTreeSet<_>>()
            .len(),
        total_edges: relationships.len(),
        summarized_nodes,
    }
}

pub fn detect_lineage_conflicts(relationships: &[SemanticRelationship]) -> Vec<LineageConflict> {
    let mut grouped: BTreeMap<(String, String), BTreeSet<ArtifactRelationshipType>> =
        BTreeMap::new();
    for relation in relationships {
        grouped
            .entry((relation.from_id.clone(), relation.to_id.clone()))
            .or_default()
            .insert(relation.relationship_type.clone());
    }

    let mut conflicts = Vec::new();
    for ((from_id, to_id), relation_types) in grouped {
        if relation_types.len() > 1 {
            conflicts.push(LineageConflict {
                relation_key: format!("{from_id}->{to_id}"),
                reasons: vec!["multiple relationship types detected for same edge".to_string()],
            });
        }
    }
    conflicts
}

pub fn lineage_quality_score(
    relationships: &[SemanticRelationship],
    verified_edges: usize,
) -> LineageQualityScore {
    let total = relationships.len().max(1);
    let exact = relationships
        .iter()
        .filter(|relation| relation.confidence == LineageConfidence::ExactDeclared)
        .count();

    LineageQualityScore {
        completeness: ((relationships.len() * 100) / total) as u8,
        exactness: ((exact * 100) / total) as u8,
        verification_coverage: ((verified_edges.min(total) * 100) / total) as u8,
    }
}

pub fn policy_hook_allows_operation(input: &PolicyLineageHookInput) -> bool {
    if input.has_policy_dependencies && input.quality.verification_coverage < 70 {
        return false;
    }
    input.relationship_count > 0
}

pub fn export_lineage_format(format: &LineageExportFormat) -> &'static str {
    match format {
        LineageExportFormat::Json => "application/json",
        LineageExportFormat::JsonLines => "application/x-ndjson",
        LineageExportFormat::GraphMl => "application/graphml+xml",
    }
}

pub fn recommended_replay_set(
    relationships: &[SemanticRelationship],
    target_id: &str,
) -> ReplayRecommendation {
    let upstream: Vec<String> = relationships
        .iter()
        .filter(|relation| relation.to_id == target_id)
        .map(|relation| relation.from_id.clone())
        .collect();

    ReplayRecommendation {
        target_id: target_id.to_string(),
        minimal_recompute_upstream: upstream,
    }
}
