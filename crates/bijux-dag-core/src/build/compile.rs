use crate::contract::{normalize_graph_with_defaults, GraphContract};
use crate::expansion::expand_graph;
use crate::node::{derive_interface, NodeTypeRegistry, TypedNode};
use crate::resources::GraphDefaults;
use crate::{
    parse_graph_strict, Graph, GraphError, ValidationDiagnostic, CANONICALIZATION_CONTRACT_VERSION,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagCompilePlanHints {
    pub deterministic_topology_order: Vec<String>,
    pub typed_nodes: Vec<TypedNode>,
    pub canonicalization_contract_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagCompileResult {
    pub normalized_graph: Graph,
    pub diagnostics: Vec<ValidationDiagnostic>,
    pub graph_fingerprint: String,
    pub plan_hints: DagCompilePlanHints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityDecision {
    AcceptExact,
    RejectVersion,
}

pub fn negotiate_spec_version(version: &str) -> CompatibilityDecision {
    if version == crate::SPEC_VERSION || version == "0.1" || version == "v0.1" {
        CompatibilityDecision::AcceptExact
    } else {
        CompatibilityDecision::RejectVersion
    }
}

pub fn compile_graph(graph: &Graph) -> Result<DagCompileResult, GraphError> {
    compile_graph_with_defaults(graph, &GraphDefaults::default())
}

pub fn compile_graph_with_defaults(
    graph: &Graph,
    defaults: &GraphDefaults,
) -> Result<DagCompileResult, GraphError> {
    let expanded_graph = expand_graph(graph).map_err(|_| GraphError::ValidationFailed)?;
    let normalized_graph = normalize_graph_with_defaults(&expanded_graph, defaults).canonicalize();
    let diagnostics = normalized_graph.validate_with_warnings();
    let graph_fingerprint = normalized_graph.graph_fingerprint()?;
    let deterministic_topology_order = normalized_graph.topo_order()?;
    let typed_nodes = normalized_graph
        .nodes
        .iter()
        .cloned()
        .map(|node| TypedNode { interface: derive_interface(&node), node })
        .collect();
    let registry = NodeTypeRegistry::default_registry();
    registry
        .validate_node_kinds(&normalized_graph.nodes)
        .map_err(|_| GraphError::ValidationFailed)?;

    Ok(DagCompileResult {
        normalized_graph,
        diagnostics,
        graph_fingerprint,
        plan_hints: DagCompilePlanHints {
            deterministic_topology_order,
            typed_nodes,
            canonicalization_contract_version: CANONICALIZATION_CONTRACT_VERSION.to_string(),
        },
    })
}

pub fn compile_graph_contract(contract: &GraphContract) -> Result<DagCompileResult, GraphError> {
    compile_graph_with_defaults(&contract.graph, &contract.defaults)
}

pub fn compile_graph_strict(input: &str) -> Result<DagCompileResult, GraphError> {
    let graph = parse_graph_strict(input)?;
    compile_graph(&graph)
}
