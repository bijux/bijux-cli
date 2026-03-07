use crate::Graph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConditionalExecution {
    pub condition_id: String,
    pub expression: String,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchDecisionNode {
    pub node_id: String,
    pub output_contract: String,
    pub true_target_group: String,
    pub false_target_group: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JoinSemantics {
    pub join_node_id: String,
    pub reconciliation_mode: String,
    pub deterministic_ordering: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PartitionSemantics {
    pub partition_key: String,
    pub partition_source: String,
    pub stable_partition_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MapFanOutSemantics {
    pub map_node_id: String,
    pub partition: PartitionSemantics,
    pub expansion_rule_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReduceFanInSemantics {
    pub reduce_node_id: String,
    pub aggregation_order: String,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WindowingSemantics {
    pub window_unit: String,
    pub boundary_start_unix_ms: u128,
    pub boundary_end_unix_ms: u128,
    pub inclusive_end: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphTemplate {
    pub template_id: String,
    pub frozen_expansion_contract: String,
    pub parameters: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubgraphEmbedding {
    pub parent_graph_id: String,
    pub embedded_graph_id: String,
    pub entry_nodes: Vec<String>,
    pub exit_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphCompositionContract {
    pub base_graph_id: String,
    pub inherited_graph_ids: Vec<String>,
    pub immutable_after_compile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ParameterBindingSemantics {
    pub graph_bindings: BTreeMap<String, String>,
    pub node_bindings: BTreeMap<String, BTreeMap<String, String>>,
    pub runtime_bindings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LateBindingRule {
    pub binding_name: String,
    pub allowed_pre_compile: bool,
    pub allowed_post_compile: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DynamicEdgeExpansionRule {
    pub rule_id: String,
    pub source_node_id: String,
    pub deterministic: bool,
    pub snapshot_captured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticNodeExistenceExplanation {
    pub node_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticEdgeExistenceExplanation {
    pub from_node_id: String,
    pub to_node_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticOrderingExplanation {
    pub node_id: String,
    pub ordered_after: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphExplainabilityModel {
    pub node_explanations: Vec<SemanticNodeExistenceExplanation>,
    pub edge_explanations: Vec<SemanticEdgeExistenceExplanation>,
    pub ordering_explanations: Vec<SemanticOrderingExplanation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StaticAnalysisReport {
    pub unreachable_nodes: Vec<String>,
    pub dead_branch_nodes: Vec<String>,
    pub noop_join_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SemanticDiffClass {
    Topology,
    Policy,
    MetadataOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityClassification {
    Safe,
    ReplaySafe,
    CacheBreaking,
    ScheduleBreaking,
    PolicyBreaking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SemanticDiffReport {
    pub class: SemanticDiffClass,
    pub compatibility: CompatibilityClassification,
    pub changed_nodes: Vec<String>,
    pub changed_edges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSemanticGraph {
    pub graph: Graph,
    pub partition_expansion_contracts: Vec<String>,
    pub dynamic_edge_rules: Vec<DynamicEdgeExpansionRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GraphMigrationPatch {
    pub from_spec: String,
    pub to_spec: String,
    pub steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphComplexityScore {
    pub node_count: usize,
    pub edge_count: usize,
    pub branch_factor: f64,
    pub depth_estimate: usize,
    pub score: f64,
}

pub fn enforce_late_binding_immutability(rules: &[LateBindingRule]) -> Result<(), String> {
    for rule in rules {
        if rule.allowed_post_compile {
            return Err(format!(
                "late binding '{}' violates snapshot immutability",
                rule.binding_name
            ));
        }
    }
    Ok(())
}

pub fn normalize_semantic_graph(
    graph: &Graph,
    partition_contracts: Vec<String>,
    dynamic_edge_rules: Vec<DynamicEdgeExpansionRule>,
) -> Result<NormalizedSemanticGraph, String> {
    for rule in &dynamic_edge_rules {
        if !rule.deterministic || !rule.snapshot_captured {
            return Err(format!(
                "dynamic edge rule '{}' must be deterministic and snapshot-captured",
                rule.rule_id
            ));
        }
    }
    Ok(NormalizedSemanticGraph {
        graph: graph.clone().canonicalize(),
        partition_expansion_contracts: partition_contracts,
        dynamic_edge_rules,
    })
}

pub fn semantic_diff(before: &Graph, after: &Graph) -> SemanticDiffReport {
    let before_nodes: BTreeSet<String> = before.nodes.iter().map(|n| n.id.clone()).collect();
    let after_nodes: BTreeSet<String> = after.nodes.iter().map(|n| n.id.clone()).collect();
    let changed_nodes = before_nodes
        .symmetric_difference(&after_nodes)
        .cloned()
        .collect::<Vec<_>>();
    let before_edges: BTreeSet<String> = before
        .edges
        .iter()
        .map(|e| format!("{}->{}", e.from.node_id, e.to.node_id))
        .collect();
    let after_edges: BTreeSet<String> = after
        .edges
        .iter()
        .map(|e| format!("{}->{}", e.from.node_id, e.to.node_id))
        .collect();
    let changed_edges = before_edges
        .symmetric_difference(&after_edges)
        .cloned()
        .collect::<Vec<_>>();
    if !changed_nodes.is_empty() || !changed_edges.is_empty() {
        return SemanticDiffReport {
            class: SemanticDiffClass::Topology,
            compatibility: CompatibilityClassification::CacheBreaking,
            changed_nodes,
            changed_edges,
        };
    }
    if before.nondeterminism_allowed != after.nondeterminism_allowed {
        return SemanticDiffReport {
            class: SemanticDiffClass::Policy,
            compatibility: CompatibilityClassification::PolicyBreaking,
            changed_nodes,
            changed_edges,
        };
    }
    SemanticDiffReport {
        class: SemanticDiffClass::MetadataOnly,
        compatibility: CompatibilityClassification::ReplaySafe,
        changed_nodes,
        changed_edges,
    }
}

pub fn classify_compatibility(diff: &SemanticDiffReport) -> CompatibilityClassification {
    diff.compatibility.clone()
}

pub fn migration_patch(from_spec: &str, to_spec: &str) -> GraphMigrationPatch {
    GraphMigrationPatch {
        from_spec: from_spec.to_string(),
        to_spec: to_spec.to_string(),
        steps: vec![
            "parse previous graph snapshot".to_string(),
            "apply schema translation rules".to_string(),
            "recompute canonical graph fingerprint".to_string(),
            "emit migration diagnostics".to_string(),
        ],
    }
}

pub fn explain_graph(graph: &Graph) -> GraphExplainabilityModel {
    let node_explanations = graph
        .nodes
        .iter()
        .map(|n| SemanticNodeExistenceExplanation {
            node_id: n.id.clone(),
            reason: "declared in graph snapshot".to_string(),
        })
        .collect();
    let edge_explanations = graph
        .edges
        .iter()
        .map(|e| SemanticEdgeExistenceExplanation {
            from_node_id: e.from.node_id.clone(),
            to_node_id: e.to.node_id.clone(),
            reason: "declared dependency edge".to_string(),
        })
        .collect();
    let ordering_explanations = graph
        .nodes
        .iter()
        .map(|n| {
            let predecessors = graph
                .edges
                .iter()
                .filter(|e| e.to.node_id == n.id)
                .map(|e| e.from.node_id.clone())
                .collect::<Vec<_>>();
            SemanticOrderingExplanation {
                node_id: n.id.clone(),
                ordered_after: predecessors,
                reason: "topological precedence".to_string(),
            }
        })
        .collect();
    GraphExplainabilityModel {
        node_explanations,
        edge_explanations,
        ordering_explanations,
    }
}

pub fn static_analysis(graph: &Graph) -> StaticAnalysisReport {
    let mut indegree: BTreeMap<String, usize> = graph.nodes.iter().map(|n| (n.id.clone(), 0)).collect();
    let mut adj: BTreeMap<String, Vec<String>> = graph.nodes.iter().map(|n| (n.id.clone(), vec![])).collect();
    for edge in &graph.edges {
        *indegree.entry(edge.to.node_id.clone()).or_insert(0) += 1;
        adj.entry(edge.from.node_id.clone())
            .or_default()
            .push(edge.to.node_id.clone());
    }
    let mut queue: VecDeque<String> = indegree
        .iter()
        .filter_map(|(id, &d)| if d == 0 { Some(id.clone()) } else { None })
        .collect();
    let mut reachable = BTreeSet::new();
    while let Some(node) = queue.pop_front() {
        if !reachable.insert(node.clone()) {
            continue;
        }
        for next in adj.get(&node).cloned().unwrap_or_default() {
            let deg = indegree.entry(next.clone()).or_insert(0);
            *deg = deg.saturating_sub(1);
            if *deg == 0 {
                queue.push_back(next);
            }
        }
    }
    let unreachable_nodes = graph
        .nodes
        .iter()
        .filter_map(|n| (!reachable.contains(&n.id)).then_some(n.id.clone()))
        .collect::<Vec<_>>();
    let dead_branch_nodes = graph
        .nodes
        .iter()
        .filter(|n| n.group.as_deref() == Some("dead-branch"))
        .map(|n| n.id.clone())
        .collect::<Vec<_>>();
    let noop_join_nodes = graph
        .nodes
        .iter()
        .filter(|n| n.tags.iter().any(|t| t == "noop-join"))
        .map(|n| n.id.clone())
        .collect::<Vec<_>>();
    StaticAnalysisReport {
        unreachable_nodes,
        dead_branch_nodes,
        noop_join_nodes,
    }
}

pub fn complexity_score(graph: &Graph) -> GraphComplexityScore {
    let node_count = graph.nodes.len();
    let edge_count = graph.edges.len();
    let mut outgoing = BTreeMap::<String, usize>::new();
    for edge in &graph.edges {
        *outgoing.entry(edge.from.node_id.clone()).or_insert(0) += 1;
    }
    let branch_factor = if node_count == 0 {
        0.0
    } else {
        outgoing.values().copied().sum::<usize>() as f64 / node_count as f64
    };
    let depth_estimate = graph.topo_order().map(|o| o.len()).unwrap_or(node_count);
    let score = node_count as f64 + edge_count as f64 * 1.5 + branch_factor * 2.0;
    GraphComplexityScore {
        node_count,
        edge_count,
        branch_factor,
        depth_estimate,
        score,
    }
}
