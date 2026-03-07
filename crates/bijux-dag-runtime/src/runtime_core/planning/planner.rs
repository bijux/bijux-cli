use crate::execution_plan::{ExecutionPlan, PlannedDependency, PlannedNode};
use crate::{RuntimeConfig, Selector, SelectorSet};
use bijux_dag_core::{Graph, Node, NodeKind};
use sha2::{Digest, Sha256};
use std::collections::{BTreeSet, HashMap};

pub fn build_plan(graph: &Graph, options: &RuntimeConfig) -> ExecutionPlan {
    let canonical = graph.canonicalize();
    let order = canonical
        .nodes
        .iter()
        .map(|n| n.id.clone())
        .collect::<Vec<String>>();
    let dep_map = build_dep_map(graph);
    let (indegree, adj) = build_graph_index(graph);
    let mut filter_reasons = HashMap::new();
    for node in &graph.nodes {
        if let Some(reason) = filter_reason(node, &options.selectors) {
            filter_reasons.insert(node.id.clone(), reason);
        }
    }
    if options.partial_rerun_dependency_closure {
        let mut keep = BTreeSet::new();
        for node in &graph.nodes {
            if !filter_reasons.contains_key(&node.id) {
                expand_dependencies(&node.id, &dep_map, &mut keep);
            }
        }
        for node in &graph.nodes {
            if !keep.contains(&node.id) {
                filter_reasons.insert(node.id.clone(), "filtered".to_string());
            }
        }
    }
    let planned_dependencies = canonical
        .edges
        .iter()
        .map(|edge| PlannedDependency {
            from: edge.from.node_id.clone(),
            to: edge.to.node_id.clone(),
        })
        .collect::<Vec<_>>();
    let planned_nodes = canonical
        .nodes
        .iter()
        .map(|node| PlannedNode {
            id: node.id.clone(),
            kind: node.kind.as_str().to_string(),
            deps: dep_map
                .get(&node.id)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .collect(),
            outputs: node.outputs.clone(),
            retry: node.retry.clone(),
            timeout_ms: node.timeout_ms,
        })
        .collect::<Vec<_>>();
    let graph_fingerprint = canonical
        .graph_fingerprint()
        .unwrap_or_else(|_| "graph-fingerprint-unavailable".to_string());
    let planner_fingerprint = planner_fingerprint(&planned_nodes, &planned_dependencies, &order)
        .unwrap_or_else(|_| "planner-fingerprint-unavailable".to_string());

    let mut diagnostics = Vec::new();
    for node in &canonical.nodes {
        if !node_kind_supported(node.kind.as_str()) {
            diagnostics.push(format!("P4013:{}:unsupported-node-kind", node.id));
        }
        if node.resources.is_some() && !runtime_resource_capability_supported(&node.kind) {
            diagnostics.push(format!("P4021:{}:unsupported-runtime-capability", node.id));
        }
    }
    ExecutionPlan {
        planner_contract_version: "bijux-dag-runtime-planner/v1".to_string(),
        graph_fingerprint,
        planner_fingerprint,
        planned_nodes,
        planned_dependencies,
        diagnostics,
        nodes: graph.nodes.clone(),
        order,
        dep_map,
        indegree,
        adj,
        filter_reasons,
    }
}

fn planner_fingerprint(
    nodes: &[PlannedNode],
    deps: &[PlannedDependency],
    order: &[String],
) -> Result<String, String> {
    let payload = serde_json::to_vec(&(nodes, deps, order)).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

fn node_kind_supported(kind: &str) -> bool {
    matches!(kind, "const" | "shell" | "container")
}

fn runtime_resource_capability_supported(kind: &NodeKind) -> bool {
    matches!(kind, NodeKind::Shell | NodeKind::Container)
}

fn build_dep_map(graph: &Graph) -> HashMap<String, BTreeSet<String>> {
    let mut map: HashMap<String, BTreeSet<String>> = HashMap::new();
    for edge in &graph.edges {
        map.entry(edge.to.node_id.clone())
            .or_default()
            .insert(edge.from.node_id.clone());
    }
    map
}

fn build_graph_index(graph: &Graph) -> (HashMap<String, usize>, HashMap<String, Vec<String>>) {
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for node in &graph.nodes {
        indegree.insert(node.id.clone(), 0);
        adj.insert(node.id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        let from = edge.from.node_id.clone();
        let to = edge.to.node_id.clone();
        if let Some(v) = adj.get_mut(&from) {
            v.push(to.clone());
        }
        if let Some(d) = indegree.get_mut(&to) {
            *d += 1;
        }
    }
    (indegree, adj)
}

fn filter_reason(node: &Node, selectors: &SelectorSet) -> Option<String> {
    if !selectors.include.is_empty()
        && !selectors
            .include
            .iter()
            .any(|sel| selector_matches(node, sel))
    {
        return Some("filtered".to_string());
    }
    if selectors
        .exclude
        .iter()
        .any(|sel| selector_matches(node, sel))
    {
        return Some("filtered".to_string());
    }
    None
}

fn expand_dependencies(
    node_id: &str,
    dep_map: &HashMap<String, BTreeSet<String>>,
    keep: &mut BTreeSet<String>,
) {
    if !keep.insert(node_id.to_string()) {
        return;
    }
    if let Some(deps) = dep_map.get(node_id) {
        for dep in deps {
            expand_dependencies(dep, dep_map, keep);
        }
    }
}

fn selector_matches(node: &Node, selector: &Selector) -> bool {
    match selector {
        Selector::IdPrefix(prefix) => node.id.starts_with(prefix),
        Selector::Tag(tag) => node.tags.iter().any(|t| t == tag),
        Selector::Kind(kind) => node.kind.as_str() == kind,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dag_core::{FileOutput, Graph, Node, NodeKind, RetryPolicy};

    fn sample_graph() -> Graph {
        Graph {
            spec: "bijux-dag/v0.1".to_string(),
            meta: None,
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out".to_string(),
                    }],
                    params: Default::default(),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["etl".to_string()],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Shell,
                    inputs: vec![],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out".to_string(),
                    }],
                    params: Default::default(),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec!["gpu".to_string()],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![],
        }
    }

    #[test]
    fn planner_filters_by_tag() {
        let graph = sample_graph();
        let options = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![Selector::Tag("etl".to_string())],
                exclude: vec![],
            },
            ..RuntimeConfig::default()
        };
        let plan = build_plan(&graph, &options);
        assert!(plan.filter_reasons.contains_key("b"));
        assert!(!plan.filter_reasons.contains_key("a"));
    }

    #[test]
    fn planner_filters_by_tag_exclude() {
        let graph = sample_graph();
        let options = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![],
                exclude: vec![Selector::Tag("etl".to_string())],
            },
            ..RuntimeConfig::default()
        };
        let plan = build_plan(&graph, &options);
        assert!(plan.filter_reasons.contains_key("a"));
        assert!(!plan.filter_reasons.contains_key("b"));
    }

    #[test]
    fn planner_inclusion_and_exclusion_combined() {
        let graph = sample_graph();
        let options = RuntimeConfig {
            selectors: SelectorSet {
                include: vec![Selector::Tag("etl".to_string())],
                exclude: vec![Selector::IdPrefix("a".to_string())],
            },
            ..RuntimeConfig::default()
        };
        let plan = build_plan(&graph, &options);
        assert!(plan.filter_reasons.contains_key("a"));
        assert!(plan.filter_reasons.contains_key("b"));
    }
}
