use crate::graph::selection::SelectionSummary;
use bijux_dag_core::resources::{node_accelerator, node_gpu_devices, node_named_resources};
use bijux_dag_core::{
    node_io_contract, EdgeKind, Graph, Node, NodeOutputContract, SemanticNodeKind, TriggerRule,
};
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphInspectionSource {
    pub(crate) kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphInspectionNode {
    pub(crate) node_id: String,
    pub(crate) kind: String,
    pub(crate) semantic_kind: SemanticNodeKind,
    pub(crate) trigger_rule: TriggerRule,
    pub(crate) selected: bool,
    pub(crate) upstream_nodes: Vec<String>,
    pub(crate) downstream_nodes: Vec<String>,
    pub(crate) resources: GraphNodeResourceSummary,
    pub(crate) output_contracts: Vec<NodeOutputContract>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphInspectionEdge {
    pub(crate) from: String,
    pub(crate) from_port: String,
    pub(crate) to: String,
    pub(crate) to_port: String,
    pub(crate) kind: EdgeKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) decision: Option<String>,
    pub(crate) selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphNodeResourceSummary {
    pub(crate) cpu: u32,
    pub(crate) mem_mb: u32,
    pub(crate) gpu_devices: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) accelerator: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) named_resources: BTreeMap<String, u32>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphResourceClaims {
    pub(crate) total_cpu: u32,
    pub(crate) total_mem_mb: u32,
    pub(crate) total_gpu_devices: u32,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) total_named_resources: BTreeMap<String, u32>,
    pub(crate) nodes: Vec<GraphResourceNodeClaim>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphResourceNodeClaim {
    pub(crate) node_id: String,
    pub(crate) selected: bool,
    pub(crate) resources: GraphNodeResourceSummary,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphOutputContractSummary {
    pub(crate) node_id: String,
    pub(crate) selected: bool,
    pub(crate) outputs: Vec<NodeOutputContract>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphBranchSummary {
    pub(crate) node_id: String,
    pub(crate) selected: bool,
    pub(crate) decisions: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) default_decision: Option<String>,
    pub(crate) decision_output: String,
    pub(crate) paths: Vec<GraphBranchPathSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphBranchPathSummary {
    pub(crate) decision: String,
    pub(crate) direct_targets: Vec<String>,
    pub(crate) reachable_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphJoinSummary {
    pub(crate) node_id: String,
    pub(crate) selected: bool,
    pub(crate) trigger_rule: TriggerRule,
    pub(crate) upstream_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphTopologySummary {
    pub(crate) roots: Vec<String>,
    pub(crate) leaves: Vec<String>,
    pub(crate) selected_roots: Vec<String>,
    pub(crate) selected_leaves: Vec<String>,
    pub(crate) branches: Vec<GraphBranchSummary>,
    pub(crate) joins: Vec<GraphJoinSummary>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct GraphInspectionPayload {
    pub(crate) source: GraphInspectionSource,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) graph_fingerprint: Option<String>,
    pub(crate) graph: Graph,
    pub(crate) nodes: Vec<GraphInspectionNode>,
    pub(crate) edges: Vec<GraphInspectionEdge>,
    pub(crate) topology: GraphTopologySummary,
    pub(crate) resources: GraphResourceClaims,
    pub(crate) output_contracts: Vec<GraphOutputContractSummary>,
    pub(crate) selection: SelectionSummary,
}

pub(crate) fn build_graph_inspection_payload(
    graph: &Graph,
    graph_fingerprint: Option<String>,
    source: GraphInspectionSource,
    selection: SelectionSummary,
) -> GraphInspectionPayload {
    let canonical = graph.canonicalize();
    let selected = selection.selected_nodes.iter().cloned().collect::<BTreeSet<_>>();
    let inbound = inbound_neighbors(&canonical);
    let outbound = outbound_neighbors(&canonical);

    let nodes = canonical
        .nodes
        .iter()
        .map(|node| GraphInspectionNode {
            node_id: node.id.clone(),
            kind: node.kind.as_str().to_string(),
            semantic_kind: node.semantic_kind.clone(),
            trigger_rule: node.trigger_rule.clone(),
            selected: selected.contains(&node.id),
            upstream_nodes: inbound.get(&node.id).cloned().unwrap_or_default(),
            downstream_nodes: outbound.get(&node.id).cloned().unwrap_or_default(),
            resources: node_resource_summary(node),
            output_contracts: node_io_contract(&canonical, &node.id)
                .map(|contract| contract.outputs)
                .unwrap_or_default(),
        })
        .collect::<Vec<_>>();

    let edges = canonical
        .edges
        .iter()
        .map(|edge| GraphInspectionEdge {
            from: edge.from.node_id.clone(),
            from_port: edge.from.port.clone(),
            to: edge.to.node_id.clone(),
            to_port: edge.to.port.clone(),
            kind: edge.kind.clone(),
            decision: edge.decision.clone(),
            selected: selected.contains(&edge.from.node_id) && selected.contains(&edge.to.node_id),
        })
        .collect::<Vec<_>>();

    let output_contracts = nodes
        .iter()
        .map(|node| GraphOutputContractSummary {
            node_id: node.node_id.clone(),
            selected: node.selected,
            outputs: node.output_contracts.clone(),
        })
        .collect::<Vec<_>>();

    GraphInspectionPayload {
        source,
        graph_fingerprint,
        graph: canonical.clone(),
        nodes,
        edges,
        topology: GraphTopologySummary {
            roots: roots(&canonical, None),
            leaves: leaves(&canonical, None),
            selected_roots: roots(&canonical, Some(&selected)),
            selected_leaves: leaves(&canonical, Some(&selected)),
            branches: branch_summaries(&canonical, &selected),
            joins: join_summaries(&canonical, &selected, &inbound),
        },
        resources: resource_claims(&canonical, &selected),
        output_contracts,
        selection,
    }
}

fn node_resource_summary(node: &Node) -> GraphNodeResourceSummary {
    GraphNodeResourceSummary {
        cpu: node.resources.as_ref().map(|resources| resources.cpu).unwrap_or_default(),
        mem_mb: node.resources.as_ref().map(|resources| resources.mem_mb).unwrap_or_default(),
        gpu_devices: node_gpu_devices(node),
        accelerator: node_accelerator(node),
        named_resources: node_named_resources(node),
    }
}

fn inbound_neighbors(graph: &Graph) -> BTreeMap<String, Vec<String>> {
    let mut inbound = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), Vec::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        inbound.entry(edge.to.node_id.clone()).or_default().push(edge.from.node_id.clone());
    }
    for neighbors in inbound.values_mut() {
        neighbors.sort();
        neighbors.dedup();
    }
    inbound
}

fn outbound_neighbors(graph: &Graph) -> BTreeMap<String, Vec<String>> {
    let mut outbound = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), Vec::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for edge in &graph.edges {
        outbound.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
    for neighbors in outbound.values_mut() {
        neighbors.sort();
        neighbors.dedup();
    }
    outbound
}

fn roots(graph: &Graph, selected: Option<&BTreeSet<String>>) -> Vec<String> {
    let inbound = inbound_neighbors(graph);
    let mut roots = graph
        .nodes
        .iter()
        .filter(|node| selected.is_none_or(|selected| selected.contains(&node.id)))
        .filter(|node| {
            inbound.get(&node.id).is_none_or(|upstream| {
                upstream
                    .iter()
                    .all(|node_id| selected.is_some_and(|selected| !selected.contains(node_id)))
            })
        })
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    roots.sort();
    roots
}

fn leaves(graph: &Graph, selected: Option<&BTreeSet<String>>) -> Vec<String> {
    let outbound = outbound_neighbors(graph);
    let mut leaves = graph
        .nodes
        .iter()
        .filter(|node| selected.is_none_or(|selected| selected.contains(&node.id)))
        .filter(|node| {
            outbound.get(&node.id).is_none_or(|downstream| {
                downstream
                    .iter()
                    .all(|node_id| selected.is_some_and(|selected| !selected.contains(node_id)))
            })
        })
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    leaves.sort();
    leaves
}

fn branch_summaries(graph: &Graph, selected: &BTreeSet<String>) -> Vec<GraphBranchSummary> {
    let adjacency = graph.edges.iter().fold(
        BTreeMap::<String, Vec<&bijux_dag_core::Edge>>::new(),
        |mut map, edge| {
            map.entry(edge.from.node_id.clone()).or_default().push(edge);
            map
        },
    );

    graph
        .nodes
        .iter()
        .filter(|node| node.semantic_kind == SemanticNodeKind::Branch)
        .filter_map(|node| {
            let branch = node.branch.as_ref()?;
            let paths = branch
                .decisions
                .iter()
                .map(|decision| {
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

                    GraphBranchPathSummary {
                        decision: decision.clone(),
                        direct_targets: direct_targets.into_iter().collect(),
                        reachable_nodes: reachable.into_iter().collect(),
                    }
                })
                .collect::<Vec<_>>();

            Some(GraphBranchSummary {
                node_id: node.id.clone(),
                selected: selected.contains(&node.id),
                decisions: branch.decisions.clone(),
                default_decision: branch.default_decision.clone(),
                decision_output: branch.decision_output.clone(),
                paths,
            })
        })
        .collect()
}

fn join_summaries(
    graph: &Graph,
    selected: &BTreeSet<String>,
    inbound: &BTreeMap<String, Vec<String>>,
) -> Vec<GraphJoinSummary> {
    graph
        .nodes
        .iter()
        .filter_map(|node| {
            let upstream_nodes = inbound.get(&node.id).cloned().unwrap_or_default();
            (upstream_nodes.len() > 1).then(|| GraphJoinSummary {
                node_id: node.id.clone(),
                selected: selected.contains(&node.id),
                trigger_rule: node.trigger_rule.clone(),
                upstream_nodes,
            })
        })
        .collect()
}

fn resource_claims(graph: &Graph, selected: &BTreeSet<String>) -> GraphResourceClaims {
    let mut total_cpu = 0u32;
    let mut total_mem_mb = 0u32;
    let mut total_gpu_devices = 0u32;
    let mut total_named_resources = BTreeMap::new();
    let nodes = graph
        .nodes
        .iter()
        .map(|node| {
            let resources = node_resource_summary(node);
            total_cpu += resources.cpu;
            total_mem_mb += resources.mem_mb;
            total_gpu_devices += resources.gpu_devices;
            for (name, amount) in &resources.named_resources {
                *total_named_resources.entry(name.clone()).or_insert(0) += amount;
            }
            GraphResourceNodeClaim {
                node_id: node.id.clone(),
                selected: selected.contains(&node.id),
                resources,
            }
        })
        .collect::<Vec<_>>();

    GraphResourceClaims { total_cpu, total_mem_mb, total_gpu_devices, total_named_resources, nodes }
}

#[cfg(test)]
mod tests {
    use super::{build_graph_inspection_payload, GraphInspectionSource};
    use crate::graph::selection::SelectionSummary;

    #[test]
    fn graph_inspection_payload_surfaces_topology_resources_and_output_contracts() {
        let graph = crate::parse_graph(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"source","kind":"const","outputs":[{"name":"out","path":"source/out","kind":"value","media_type":"application/json"}],"params":{"value":1},"resources":{"cpu":1,"mem_mb":32}},
                {"id":"branch","kind":"const","semantic_kind":"branch","outputs":[{"name":"decision","path":"branch/decision","kind":"value"}],"params":{"value":"left"},"branch":{"decisions":["left","right"],"default_decision":"right","decision_output":"decision"}},
                {"id":"left","kind":"const","inputs":["in"],"outputs":[{"name":"left_out","path":"left/out"}],"params":{"value":2},"resources":{"cpu":2,"mem_mb":64,"gpu_devices":1,"named_resources":{"license.render":1}}},
                {"id":"right","kind":"const","inputs":["in"],"outputs":[{"name":"right_out","path":"right/out"}],"params":{"value":3}},
                {"id":"join","kind":"const","inputs":["left_in","right_in"],"outputs":[{"name":"done","path":"join/done","promotable":true}],"params":{"value":4},"trigger_rule":"any_success"}
              ],
              "edges":[
                {"from":{"node_id":"source","port":"out"},"to":{"node_id":"branch","port":"in"}},
                {"kind":"conditional","decision":"left","from":{"node_id":"branch","port":"decision"},"to":{"node_id":"left","port":"in"}},
                {"kind":"conditional","decision":"right","from":{"node_id":"branch","port":"decision"},"to":{"node_id":"right","port":"in"}},
                {"from":{"node_id":"left","port":"left_out"},"to":{"node_id":"join","port":"left_in"}},
                {"from":{"node_id":"right","port":"right_out"},"to":{"node_id":"join","port":"right_in"}}
              ]
            }"#,
        )
        .expect("graph");

        let payload = build_graph_inspection_payload(
            &graph,
            Some("graph-fp-1".to_string()),
            GraphInspectionSource { kind: "dag".to_string(), run_dir: None, run_id: None },
            SelectionSummary {
                requested_selectors: vec!["include:id:left".to_string()],
                upstream_targets: Vec::new(),
                downstream_roots: Vec::new(),
                dependency_closure_enabled: true,
                selected_nodes: vec![
                    "branch".to_string(),
                    "left".to_string(),
                    "source".to_string(),
                ],
                omitted_nodes: vec![
                    crate::graph::selection::OmittedNodeSummary {
                        node_id: "join".to_string(),
                        reason: "not_selected_by_include_selector".to_string(),
                    },
                    crate::graph::selection::OmittedNodeSummary {
                        node_id: "right".to_string(),
                        reason: "not_selected_by_include_selector".to_string(),
                    },
                ],
            },
        );

        assert_eq!(payload.topology.roots, vec!["source".to_string()]);
        assert_eq!(payload.topology.leaves, vec!["join".to_string()]);
        assert_eq!(payload.topology.selected_roots, vec!["source".to_string()]);
        assert_eq!(payload.topology.selected_leaves, vec!["left".to_string()]);
        assert_eq!(payload.topology.branches.len(), 1);
        assert_eq!(payload.topology.joins[0].node_id, "join");
        assert_eq!(payload.resources.total_cpu, 3);
        assert_eq!(payload.resources.total_mem_mb, 96);
        assert_eq!(payload.resources.total_gpu_devices, 1);
        assert_eq!(payload.resources.total_named_resources["license.render"], 1);
        assert!(payload
            .output_contracts
            .iter()
            .find(|summary| summary.node_id == "join")
            .is_some_and(|summary| summary.outputs[0].promotable));
        assert!(payload
            .edges
            .iter()
            .find(|edge| edge.to == "right")
            .is_some_and(|edge| !edge.selected));
    }
}
