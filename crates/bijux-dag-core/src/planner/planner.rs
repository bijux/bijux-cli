//! Planner lowering and execution-plan contract.

use crate::expansion::expand_graph;
use crate::{
    node_io_contract, BranchSpec, CacheBehavior, Edge, EdgeKind, Effect, FileOutput, Graph,
    GraphError, Node, NodeIoContract, NodeKind, ParamValue, Resources, RetryPolicy,
    SemanticNodeKind, TriggerRule,
};
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
    pub executor_kind: String,
    pub semantic_kind: SemanticNodeKind,
    pub deps: Vec<String>,
    pub io_contract: NodeIoContract,
    pub outputs: Vec<FileOutput>,
    pub side_effects: Vec<Effect>,
    pub retry: RetryPolicy,
    pub cache: CacheBehavior,
    pub trigger_rule: TriggerRule,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub resources: Option<Resources>,
    #[serde(default)]
    pub branch: Option<PlannedBranchContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedEdge {
    #[serde(default)]
    pub id: Option<String>,
    pub kind: EdgeKind,
    #[serde(default)]
    pub decision: Option<String>,
    pub from: String,
    pub from_port: String,
    pub to: String,
    pub to_port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannedBranchContract {
    pub decisions: Vec<String>,
    #[serde(default)]
    pub default_decision: Option<String>,
    pub decision_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BranchPathAnalysis {
    pub branch_node_id: String,
    pub decision: String,
    pub direct_targets: Vec<String>,
    pub reachable_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub planner_contract_version: String,
    pub spec: String,
    pub graph_fingerprint: String,
    pub planner_fingerprint: String,
    pub execution_fingerprint: String,
    pub evidence_fingerprint: String,
    pub nodes: Vec<PlannedNode>,
    pub edges: Vec<PlannedEdge>,
    pub ordering: Vec<String>,
    pub branch_paths: Vec<BranchPathAnalysis>,
    pub omitted_from_execution_identity: Vec<String>,
    pub diagnostics: Vec<PlannerDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ExecutionIdentityNode {
    id: String,
    kind: String,
    semantic_kind: SemanticNodeKind,
    deps: Vec<String>,
    outputs: Vec<FileOutput>,
    params: ParamValue,
    trigger_rule: TriggerRule,
    retry: RetryPolicy,
    cache: CacheBehavior,
    timeout_ms: Option<u64>,
    resources: Option<Resources>,
    effects: Vec<Effect>,
    env_allowlist: Vec<String>,
    branch: Option<PlannedBranchContract>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EvidenceIdentityNode {
    id: String,
    tags: Vec<String>,
    group: Option<String>,
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
    #[error("planner unsupported node kinds: {0:?}")]
    UnsupportedNodeKinds(Vec<String>),
    #[error("planner topology failure: {0}")]
    Topology(String),
    #[error("planner fingerprint failed: {0}")]
    Fingerprint(String),
}

pub fn lower_graph_to_execution_plan(
    graph: &Graph,
    mut options: PlanOptions,
) -> Result<ExecutionPlan, PlannerError> {
    let expanded = expand_graph(graph).map_err(|_| PlannerError::ValidationFailed)?;
    let validation_diags = expanded.validate_with_warnings();
    if validation_diags.iter().any(|d| d.severity == crate::Severity::Error) {
        return Err(PlannerError::ValidationFailed);
    }

    if options.supported_kinds.is_empty() {
        options.supported_kinds =
            ["const", "shell", "python", "http", "file_transform", "container"]
                .into_iter()
                .map(str::to_string)
                .collect();
    }

    let canonical = expanded.canonicalize();

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
    let mut unsupported_nodes = Vec::new();
    for node in &selected_nodes {
        let kind = node.kind.as_str().to_string();
        if !options.supported_kinds.contains(&kind) {
            unsupported_nodes.push(format!("{}:{kind}", node.id));
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
    if !unsupported_nodes.is_empty() {
        return Err(PlannerError::UnsupportedNodeKinds(unsupported_nodes));
    }

    let ordering = topo_order_selected(&selected_nodes, &selected_edges)?;
    let planned_nodes = to_planned_nodes(&selected_nodes, &selected_edges);
    let planned_edges = selected_edges
        .iter()
        .map(|e| PlannedEdge {
            id: e.id.clone(),
            kind: e.kind.clone(),
            decision: e.decision.clone(),
            from: e.from.node_id.clone(),
            from_port: e.from.port.clone(),
            to: e.to.node_id.clone(),
            to_port: e.to.port.clone(),
        })
        .collect::<Vec<_>>();
    let branch_paths = branch_path_analysis(&selected_nodes, &selected_edges);

    let graph_fingerprint =
        canonical.graph_fingerprint().map_err(|e| PlannerError::Fingerprint(e.to_string()))?;
    let planner_fingerprint = planner_fingerprint(&planned_nodes, &planned_edges, &ordering)?;
    let execution_fingerprint =
        execution_fingerprint(&canonical, &selected_nodes, &selected_edges, &planned_nodes)?;
    let evidence_fingerprint =
        evidence_fingerprint(&canonical, &selected_nodes, &selected_edges, &planned_nodes)?;

    Ok(ExecutionPlan {
        planner_contract_version: PLANNER_CONTRACT_VERSION.to_string(),
        spec: canonical.spec,
        graph_fingerprint,
        planner_fingerprint,
        execution_fingerprint,
        evidence_fingerprint,
        nodes: planned_nodes,
        edges: planned_edges,
        ordering,
        branch_paths,
        omitted_from_execution_identity: vec![
            "graph.meta".to_string(),
            "node.tags".to_string(),
            "node.group".to_string(),
        ],
        diagnostics,
    })
}

fn to_planned_nodes(nodes: &[Node], edges: &[Edge]) -> Vec<PlannedNode> {
    let mut deps = BTreeMap::<String, BTreeSet<String>>::new();
    let helper_graph = Graph {
        spec: String::new(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: nodes.to_vec(),
        edges: edges.to_vec(),
    };
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
            executor_kind: n.kind.as_str().to_string(),
            semantic_kind: n.semantic_kind.clone(),
            deps: deps.get(&n.id).cloned().unwrap_or_default().into_iter().collect(),
            io_contract: node_io_contract(&helper_graph, &n.id).unwrap_or_else(|| NodeIoContract {
                inputs: Vec::new(),
                param_bindings: Vec::new(),
                env_bindings: Vec::new(),
                outputs: Vec::new(),
            }),
            outputs: n.outputs.clone(),
            side_effects: n.effects.clone(),
            retry: n.retry.clone(),
            cache: n.cache.clone(),
            trigger_rule: n.trigger_rule.clone(),
            timeout_ms: n.timeout_ms,
            resources: n.resources.clone(),
            branch: n.branch.as_ref().map(planned_branch_contract),
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

fn execution_fingerprint(
    graph: &Graph,
    nodes: &[Node],
    edges: &[Edge],
    planned_nodes: &[PlannedNode],
) -> Result<String, PlannerError> {
    let mut hasher = Sha256::new();
    let identity_nodes = execution_identity_nodes(nodes, planned_nodes);
    let identity_edges = edges
        .iter()
        .map(|edge| {
            (
                edge.id.clone(),
                edge.kind.clone(),
                edge.decision.clone(),
                edge.from.node_id.clone(),
                edge.from.port.clone(),
                edge.to.node_id.clone(),
                edge.to.port.clone(),
            )
        })
        .collect::<Vec<_>>();
    let payload = serde_json::to_vec(&(
        &graph.spec,
        &graph.inputs,
        graph.nondeterminism_allowed,
        &identity_nodes,
        &identity_edges,
    ))
    .map_err(|e| PlannerError::Fingerprint(e.to_string()))?;
    hasher.update(payload);
    Ok(format!("{:x}", hasher.finalize()))
}

fn evidence_fingerprint(
    graph: &Graph,
    nodes: &[Node],
    edges: &[Edge],
    planned_nodes: &[PlannedNode],
) -> Result<String, PlannerError> {
    let mut hasher = Sha256::new();
    let execution_nodes = execution_identity_nodes(nodes, planned_nodes);
    let evidence_nodes = nodes
        .iter()
        .map(|node| EvidenceIdentityNode {
            id: node.id.clone(),
            tags: node.tags.clone(),
            group: node.group.clone(),
        })
        .collect::<Vec<_>>();
    let identity_edges = edges
        .iter()
        .map(|edge| {
            (
                edge.id.clone(),
                edge.kind.clone(),
                edge.decision.clone(),
                edge.from.node_id.clone(),
                edge.from.port.clone(),
                edge.to.node_id.clone(),
                edge.to.port.clone(),
            )
        })
        .collect::<Vec<_>>();
    let payload = serde_json::to_vec(&(
        &graph.spec,
        &graph.inputs,
        graph.nondeterminism_allowed,
        &execution_nodes,
        &evidence_nodes,
        &identity_edges,
    ))
    .map_err(|e| PlannerError::Fingerprint(e.to_string()))?;
    hasher.update(payload);
    Ok(format!("{:x}", hasher.finalize()))
}

fn execution_identity_nodes(
    nodes: &[Node],
    planned_nodes: &[PlannedNode],
) -> Vec<ExecutionIdentityNode> {
    let mut planned_by_id = BTreeMap::new();
    for planned in planned_nodes {
        planned_by_id.insert(planned.id.as_str(), planned);
    }
    nodes
        .iter()
        .filter_map(|node| {
            planned_by_id.get(node.id.as_str()).map(|planned| ExecutionIdentityNode {
                id: node.id.clone(),
                kind: node.kind.as_str().to_string(),
                semantic_kind: node.semantic_kind.clone(),
                deps: planned.deps.clone(),
                outputs: node.outputs.clone(),
                params: node.params.clone(),
                trigger_rule: node.trigger_rule.clone(),
                retry: node.retry.clone(),
                cache: node.cache.clone(),
                timeout_ms: node.timeout_ms,
                resources: node.resources.clone(),
                effects: node.effects.clone(),
                env_allowlist: node.env_allowlist.clone(),
                branch: node.branch.as_ref().map(planned_branch_contract),
            })
        })
        .collect()
}

pub fn planner_diagnostics_from_error(error: &PlannerError) -> Vec<PlannerDiagnostic> {
    match error {
        PlannerError::UnsupportedNodeKinds(nodes) => nodes
            .iter()
            .map(|entry| {
                let (node_id, kind) = entry.split_once(':').unwrap_or((entry.as_str(), "unknown"));
                PlannerDiagnostic {
                    id: "P4013".to_string(),
                    severity: PlannerSeverity::Error,
                    message: format!(
                        "node kind '{kind}' is not supported by runtime planner contract"
                    ),
                    node_id: Some(node_id.to_string()),
                }
            })
            .collect(),
        _ => vec![PlannerDiagnostic {
            id: "P4000".to_string(),
            severity: PlannerSeverity::Error,
            message: error.to_string(),
            node_id: None,
        }],
    }
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
    matches!(
        kind,
        NodeKind::Const
            | NodeKind::Shell
            | NodeKind::Python
            | NodeKind::Http
            | NodeKind::FileTransform
            | NodeKind::Container
    )
}

fn planned_branch_contract(branch: &BranchSpec) -> PlannedBranchContract {
    PlannedBranchContract {
        decisions: branch.decisions.clone(),
        default_decision: branch.default_decision.clone(),
        decision_output: branch.decision_output.clone(),
    }
}

fn branch_path_analysis(nodes: &[Node], edges: &[Edge]) -> Vec<BranchPathAnalysis> {
    let adjacency = edges.iter().fold(BTreeMap::<String, Vec<&Edge>>::new(), |mut map, edge| {
        map.entry(edge.from.node_id.clone()).or_default().push(edge);
        map
    });

    let mut analyses = Vec::new();
    for node in nodes {
        if node.semantic_kind != SemanticNodeKind::Branch {
            continue;
        }
        let Some(branch) = &node.branch else {
            continue;
        };
        for decision in &branch.decisions {
            let direct_targets = adjacency
                .get(&node.id)
                .into_iter()
                .flat_map(|edges| edges.iter())
                .filter(|edge| {
                    edge.kind == EdgeKind::Conditional
                        && edge.decision.as_deref() == Some(decision.as_str())
                })
                .map(|edge| edge.to.node_id.clone())
                .collect::<BTreeSet<_>>();

            let mut reachable = BTreeSet::new();
            let mut frontier = direct_targets.iter().cloned().collect::<Vec<_>>();
            while let Some(current) = frontier.pop() {
                if !reachable.insert(current.clone()) {
                    continue;
                }
                if let Some(next_edges) = adjacency.get(&current) {
                    for edge in next_edges {
                        frontier.push(edge.to.node_id.clone());
                    }
                }
            }

            analyses.push(BranchPathAnalysis {
                branch_node_id: node.id.clone(),
                decision: decision.clone(),
                direct_targets: direct_targets.into_iter().collect(),
                reachable_nodes: reachable.into_iter().collect(),
            });
        }
    }
    analyses
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
