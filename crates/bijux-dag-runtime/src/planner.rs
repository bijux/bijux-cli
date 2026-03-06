use crate::{RuntimeConfig, Selector, SelectorSet};
use bijux_dag_core::{Graph, Node};
use std::collections::{BTreeSet, HashMap};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ExecutionPlan {
    pub nodes: Vec<Node>,
    pub order: Vec<String>,
    pub dep_map: HashMap<String, BTreeSet<String>>,
    pub indegree: HashMap<String, usize>,
    pub adj: HashMap<String, Vec<String>>,
    pub filter_reasons: HashMap<String, String>,
}

pub fn build_plan(graph: &Graph, options: &RuntimeConfig) -> ExecutionPlan {
    let canonical = graph.canonicalize();
    let order = canonical.nodes.iter().map(|n| n.id.clone()).collect();
    let dep_map = build_dep_map(graph);
    let (indegree, adj) = build_graph_index(graph);
    let mut filter_reasons = HashMap::new();
    for node in &graph.nodes {
        if let Some(reason) = filter_reason(node, &options.selectors) {
            filter_reasons.insert(node.id.clone(), reason);
        }
    }
    ExecutionPlan {
        nodes: graph.nodes.clone(),
        order,
        dep_map,
        indegree,
        adj,
        filter_reasons,
    }
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
