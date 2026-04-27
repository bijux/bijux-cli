use crate::execution_plan::{ExecutionPlan, PlannedDependency, PlannedNode};
use crate::{RuntimeConfig, Selector, SelectorSet};
use bijux_dag_core::{node_io_contract, Graph, Node, NodeIoContract, NodeKind, PlanOptions, PlannerSeverity};
use std::collections::{BTreeSet, HashMap};

pub fn build_plan(graph: &Graph, options: &RuntimeConfig) -> ExecutionPlan {
    let canonical = graph.canonicalize();
    let requested_selectors = options
        .selectors
        .include
        .iter()
        .map(|selector| crate::requested_selector_label("include", selector))
        .chain(
            options
                .selectors
                .exclude
                .iter()
                .map(|selector| crate::requested_selector_label("exclude", selector)),
        )
        .collect::<Vec<_>>();
    let dep_map = build_dep_map(graph);
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
        for node_id in &keep {
            filter_reasons.remove(node_id);
        }
        for node in &graph.nodes {
            if !keep.contains(&node.id) {
                filter_reasons.entry(node.id.clone()).or_insert_with(|| {
                    "not_selected_by_dependency_closure".to_string()
                });
            }
        }
    }
    let selected_nodes = graph
        .nodes
        .iter()
        .filter_map(|node| (!filter_reasons.contains_key(&node.id)).then_some(node.id.clone()))
        .collect::<BTreeSet<_>>();
    let lowered = bijux_dag_core::lower_graph_to_execution_plan(
        graph,
        PlanOptions {
            selected_nodes,
            supported_kinds: canonical
                .nodes
                .iter()
                .map(|node| node.kind.as_str().to_string())
                .collect(),
        },
    );

    let (
        planner_contract_version,
        graph_fingerprint,
        planner_fingerprint,
        execution_fingerprint,
        evidence_fingerprint,
        planned_nodes,
        planned_dependencies,
        order,
        mut diagnostics,
    ) = match lowered {
        Ok(plan) => {
            let mut diagnostics = plan
                .diagnostics
                .iter()
                .map(|diag| {
                    let severity = match diag.severity {
                        PlannerSeverity::Error => "error",
                        PlannerSeverity::Warning => "warning",
                    };
                    format!("{}:{}:{}", diag.id, severity, diag.message)
                })
                .collect::<Vec<_>>();
            (
                plan.planner_contract_version,
                plan.graph_fingerprint,
                plan.planner_fingerprint,
                plan.execution_fingerprint,
                plan.evidence_fingerprint,
                plan.nodes
                    .iter()
                    .map(|node| PlannedNode {
                        id: node.id.clone(),
                        kind: node.kind.clone(),
                        deps: node.deps.clone(),
                        io_contract: node.io_contract.clone(),
                        outputs: node.outputs.clone(),
                        retry: node.retry.clone(),
                        timeout_ms: node.timeout_ms,
                    })
                    .collect::<Vec<_>>(),
                plan.edges
                    .iter()
                    .map(|edge| PlannedDependency {
                        from: edge.from.clone(),
                        from_port: edge.from_port.clone(),
                        to: edge.to.clone(),
                        to_port: edge.to_port.clone(),
                    })
                    .collect::<Vec<_>>(),
                plan.ordering,
                {
                    diagnostics.sort();
                    diagnostics
                },
            )
        }
        Err(error) => {
            let mut diagnostics = vec![format!("P4000:error:planner-lowering-failure:{}", error)];
            diagnostics.sort();
            (
                "bijux-dag-runtime-planner/v1".to_string(),
                canonical
                    .graph_fingerprint()
                    .unwrap_or_else(|_| "graph-fingerprint-unavailable".to_string()),
                "planner-fingerprint-unavailable".to_string(),
                "execution-fingerprint-unavailable".to_string(),
                "evidence-fingerprint-unavailable".to_string(),
                canonical
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
                        io_contract: node_io_contract(graph, &node.id).unwrap_or_else(|| NodeIoContract {
                            inputs: Vec::new(),
                            param_bindings: Vec::new(),
                            env_bindings: Vec::new(),
                            outputs: Vec::new(),
                        }),
                        outputs: node.outputs.clone(),
                        retry: node.retry.clone(),
                        timeout_ms: node.timeout_ms,
                    })
                    .collect::<Vec<_>>(),
                canonical
                    .edges
                    .iter()
                    .map(|edge| PlannedDependency {
                        from: edge.from.node_id.clone(),
                        from_port: edge.from.port.clone(),
                        to: edge.to.node_id.clone(),
                        to_port: edge.to.port.clone(),
                    })
                    .collect::<Vec<_>>(),
                canonical.nodes.iter().map(|node| node.id.clone()).collect::<Vec<_>>(),
                diagnostics,
            )
        }
    };

    for node in &canonical.nodes {
        if !node_kind_supported(node.kind.as_str()) {
            diagnostics.push(format!("P4013:{}:unsupported-node-kind", node.id));
        }
        if node.resources.is_some() && !runtime_resource_capability_supported(&node.kind) {
            diagnostics.push(format!("P4021:{}:unsupported-runtime-capability", node.id));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    let (indegree, adj) = build_graph_index_from_plan(&planned_nodes, &planned_dependencies);
    let plan_dep_map = build_dep_map_from_plan(&planned_dependencies);

    ExecutionPlan {
        planner_contract_version,
        graph_fingerprint,
        planner_fingerprint,
        execution_fingerprint,
        evidence_fingerprint,
        requested_selectors,
        dependency_closure_enabled: options.partial_rerun_dependency_closure,
        planned_nodes,
        planned_dependencies,
        diagnostics,
        nodes: graph.nodes.clone(),
        order,
        dep_map: plan_dep_map,
        indegree,
        adj,
        filter_reasons,
    }
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
        map.entry(edge.to.node_id.clone()).or_default().insert(edge.from.node_id.clone());
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

fn build_dep_map_from_plan(plan_deps: &[PlannedDependency]) -> HashMap<String, BTreeSet<String>> {
    let mut map: HashMap<String, BTreeSet<String>> = HashMap::new();
    for edge in plan_deps {
        map.entry(edge.to.clone()).or_default().insert(edge.from.clone());
    }
    map
}

fn build_graph_index_from_plan(
    nodes: &[PlannedNode],
    deps: &[PlannedDependency],
) -> (HashMap<String, usize>, HashMap<String, Vec<String>>) {
    let mut indegree: HashMap<String, usize> = HashMap::new();
    let mut adj: HashMap<String, Vec<String>> = HashMap::new();
    for node in nodes {
        indegree.insert(node.id.clone(), 0);
        adj.insert(node.id.clone(), Vec::new());
    }
    for edge in deps {
        if let Some(v) = adj.get_mut(&edge.from) {
            v.push(edge.to.clone());
        }
        if let Some(d) = indegree.get_mut(&edge.to) {
            *d += 1;
        }
    }
    (indegree, adj)
}

fn filter_reason(node: &Node, selectors: &SelectorSet) -> Option<String> {
    if !selectors.include.is_empty()
        && !selectors.include.iter().any(|sel| selector_matches(node, sel))
    {
        return Some("not_selected_by_include_selector".to_string());
    }
    if selectors.exclude.iter().any(|sel| selector_matches(node, sel)) {
        return Some("excluded_by_selector".to_string());
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
                    outputs: vec![FileOutput { name: "out".to_string(), path: "out".to_string() }],
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
                    outputs: vec![FileOutput { name: "out".to_string(), path: "out".to_string() }],
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
        assert_eq!(
            plan.filter_reasons.get("b").map(String::as_str),
            Some("not_selected_by_include_selector")
        );
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
        assert_eq!(plan.filter_reasons.get("a").map(String::as_str), Some("excluded_by_selector"));
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
        assert_eq!(plan.filter_reasons.get("a").map(String::as_str), Some("excluded_by_selector"));
        assert_eq!(
            plan.filter_reasons.get("b").map(String::as_str),
            Some("not_selected_by_include_selector")
        );
    }
}
