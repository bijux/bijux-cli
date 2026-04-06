//! Planner lowering and execution-plan contract.

use crate::{Edge, FileOutput, Graph, GraphError, Node, NodeKind, RetryPolicy};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

pub const PLANNER_CONTRACT_VERSION: &str = "bijux-dag-planner/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PlannerSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerDiagnostic {
    pub id: String,
    pub severity: PlannerSeverity,
    pub message: String,
    #[serde(default)]
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedNode {
    pub id: String,
    pub kind: String,
    pub deps: Vec<String>,
    pub outputs: Vec<FileOutput>,
    pub retry: RetryPolicy,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedEdge {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub planner_contract_version: String,
    pub spec: String,
    pub graph_fingerprint: String,
    pub planner_fingerprint: String,
    pub nodes: Vec<PlannedNode>,
    pub edges: Vec<PlannedEdge>,
    pub ordering: Vec<String>,
    pub erased_fields: Vec<String>,
    pub diagnostics: Vec<PlannerDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlanOptions {
    #[serde(default)]
    pub selected_nodes: BTreeSet<String>,
    #[serde(default)]
    pub supported_kinds: BTreeSet<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum PlannerError {
    #[error("planner validation failed")]
    ValidationFailed,
    #[error("planner unsupported node kind: {0}")]
    UnsupportedNodeKind(String),
    #[error("planner topology failure: {0}")]
    Topology(String),
    #[error("planner fingerprint failed: {0}")]
    Fingerprint(String),
}

pub fn lower_graph_to_execution_plan(
    graph: &Graph,
    mut options: PlanOptions,
) -> Result<ExecutionPlan, PlannerError> {
    let validation_diags = graph.validate_with_warnings();
    if validation_diags.iter().any(|d| d.severity == crate::Severity::Error) {
        return Err(PlannerError::ValidationFailed);
    }

    if options.supported_kinds.is_empty() {
        options.supported_kinds =
            ["const", "shell", "container"].into_iter().map(str::to_string).collect();
    }

    let canonical = graph.canonicalize();

    let selected = if options.selected_nodes.is_empty() {
        canonical.nodes.iter().map(|n| n.id.clone()).collect::<BTreeSet<_>>()
    } else {
        options.selected_nodes
    };

    let selected_nodes =
        canonical.nodes.iter().filter(|n| selected.contains(&n.id)).cloned().collect::<Vec<_>>();

    let selected_edges = canonical
        .edges
        .iter()
        .filter(|e| selected.contains(&e.from.node_id) && selected.contains(&e.to.node_id))
        .cloned()
        .collect::<Vec<_>>();

    let mut diagnostics = Vec::new();
    for node in &selected_nodes {
        let kind = node.kind.as_str().to_string();
        if !options.supported_kinds.contains(&kind) {
            diagnostics.push(PlannerDiagnostic {
                id: "P4013".to_string(),
                severity: PlannerSeverity::Error,
                message: format!("node kind '{kind}' is not supported by runtime planner contract"),
                node_id: Some(node.id.clone()),
            });
            return Err(PlannerError::UnsupportedNodeKind(kind));
        }
        if node.outputs.is_empty() {
            diagnostics.push(PlannerDiagnostic {
                id: "P4016".to_string(),
                severity: PlannerSeverity::Warning,
                message: "node has no declared outputs and is treated as execution no-op"
                    .to_string(),
                node_id: Some(node.id.clone()),
            });
        }
    }

    let ordering = topo_order_selected(&selected_nodes, &selected_edges)?;
    let planned_nodes = to_planned_nodes(&selected_nodes, &selected_edges);
    let planned_edges = selected_edges
        .iter()
        .map(|e| PlannedEdge { from: e.from.node_id.clone(), to: e.to.node_id.clone() })
        .collect::<Vec<_>>();

    let graph_fingerprint =
        canonical.graph_fingerprint().map_err(|e| PlannerError::Fingerprint(e.to_string()))?;
    let planner_fingerprint = planner_fingerprint(&planned_nodes, &planned_edges, &ordering)?;

    Ok(ExecutionPlan {
        planner_contract_version: PLANNER_CONTRACT_VERSION.to_string(),
        spec: canonical.spec,
        graph_fingerprint,
        planner_fingerprint,
        nodes: planned_nodes,
        edges: planned_edges,
        ordering,
        erased_fields: vec![
            "graph.meta".to_string(),
            "graph.inputs".to_string(),
            "graph.nondeterminism_allowed".to_string(),
            "node.tags".to_string(),
            "node.group".to_string(),
            "node.params".to_string(),
            "node.resources".to_string(),
        ],
        diagnostics,
    })
}

fn to_planned_nodes(nodes: &[Node], edges: &[Edge]) -> Vec<PlannedNode> {
    let mut deps = BTreeMap::<String, BTreeSet<String>>::new();
    for n in nodes {
        deps.insert(n.id.clone(), BTreeSet::new());
    }
    for edge in edges {
        deps.entry(edge.to.node_id.clone()).or_default().insert(edge.from.node_id.clone());
    }
    nodes
        .iter()
        .map(|n| PlannedNode {
            id: n.id.clone(),
            kind: n.kind.as_str().to_string(),
            deps: deps.get(&n.id).cloned().unwrap_or_default().into_iter().collect(),
            outputs: n.outputs.clone(),
            retry: n.retry.clone(),
            timeout_ms: n.timeout_ms,
        })
        .collect()
}

fn topo_order_selected(nodes: &[Node], edges: &[Edge]) -> Result<Vec<String>, PlannerError> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut outgoing = BTreeMap::<String, BTreeSet<String>>::new();
    for node in nodes {
        indegree.insert(node.id.clone(), 0);
        outgoing.insert(node.id.clone(), BTreeSet::new());
    }
    for edge in edges {
        *indegree.entry(edge.to.node_id.clone()).or_insert(0) += 1;
        outgoing.entry(edge.from.node_id.clone()).or_default().insert(edge.to.node_id.clone());
    }

    let mut ready = indegree
        .iter()
        .filter_map(|(id, count)| if *count == 0 { Some(id.clone()) } else { None })
        .collect::<BTreeSet<_>>();

    let mut order = Vec::new();
    while let Some(next) = ready.iter().next().cloned() {
        ready.remove(&next);
        order.push(next.clone());
        for child in outgoing.get(&next).cloned().unwrap_or_default() {
            let entry = indegree.entry(child.clone()).or_insert(0);
            if *entry > 0 {
                *entry -= 1;
            }
            if *entry == 0 {
                ready.insert(child);
            }
        }
    }

    if order.len() != nodes.len() {
        return Err(PlannerError::Topology(
            "selected graph contains cycle or unresolved dependency".to_string(),
        ));
    }
    Ok(order)
}

fn planner_fingerprint(
    nodes: &[PlannedNode],
    edges: &[PlannedEdge],
    ordering: &[String],
) -> Result<String, PlannerError> {
    let mut hasher = Sha256::new();
    let payload = serde_json::to_vec(&(nodes, edges, ordering))
        .map_err(|e| PlannerError::Fingerprint(e.to_string()))?;
    hasher.update(payload);
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn planner_diagnostics_from_error(error: &PlannerError) -> Vec<PlannerDiagnostic> {
    vec![PlannerDiagnostic {
        id: "P4000".to_string(),
        severity: PlannerSeverity::Error,
        message: error.to_string(),
        node_id: None,
    }]
}

pub fn graph_lowering_boundary_note() -> &'static str {
    "Selection/filtering is applied after graph validation and before execution planning."
}

pub fn planner_identity_for_graph(graph: &Graph) -> Result<(String, String), PlannerError> {
    let plan = lower_graph_to_execution_plan(graph, PlanOptions::default())?;
    Ok((plan.graph_fingerprint, plan.planner_fingerprint))
}

pub fn can_runtime_execute_plan_without_raw_graph() -> bool {
    true
}

pub fn node_kind_supported(kind: &NodeKind) -> bool {
    matches!(kind, NodeKind::Const | NodeKind::Shell | NodeKind::Container)
}

pub fn planner_alignment_required_schema() -> &'static str {
    "configs/dag/schema/execution_plan.schema.json"
}

pub fn planner_alignment_required_doc() -> &'static str {
    "docs/spec/PLANNER_CONTRACT.md"
}

pub fn planner_alignment_required_test() -> &'static str {
    "crates/bijux-dag-core/tests/planner_contract.rs"
}

pub fn map_planner_error_to_graph_error(error: PlannerError) -> GraphError {
    GraphError::InvalidSpec(error.to_string())
}
