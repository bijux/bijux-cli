use crate::compile::{compile_graph, DagCompileResult};
use crate::{
    parse_graph_strict, BranchSpec, Edge, EdgeKind, Effect, FileOutput, Graph, GraphInputSpec,
    GraphMeta, Node, NodeKind, ParamValue, PortRef, Resources, RetryPolicy, SemanticNodeKind,
    SubgraphDefinition, SubgraphInstance, TriggerRule,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DagLintFinding {
    pub code: String,
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DagDryRunPreview {
    pub node_count: usize,
    pub edge_count: usize,
    pub estimated_parallelism: usize,
    pub compile_diagnostics: Vec<String>,
}

#[derive(Default)]
pub struct DagBuilder {
    spec: String,
    meta: Option<GraphMeta>,
    inputs: BTreeMap<String, GraphInputSpec>,
    nondeterminism_allowed: bool,
    subgraphs: BTreeMap<String, SubgraphDefinition>,
    subgraph_instances: Vec<SubgraphInstance>,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

impl DagBuilder {
    pub fn new() -> Self {
        Self { spec: crate::SPEC_VERSION.to_string(), ..Self::default() }
    }

    pub fn graph_meta(mut self, meta: GraphMeta) -> Self {
        self.meta = Some(meta);
        self
    }

    pub fn graph_input(mut self, key: &str, value: Value) -> Self {
        let spec = GraphInputSpec::from_default_value(value)
            .unwrap_or_else(|error| panic!("invalid graph input shorthand for {key}: {error}"));
        self.inputs.insert(key.to_string(), spec);
        self
    }

    pub fn nondeterminism_allowed(mut self, allowed: bool) -> Self {
        self.nondeterminism_allowed = allowed;
        self
    }

    pub fn subgraph_definition(mut self, name: &str, definition: SubgraphDefinition) -> Self {
        self.subgraphs.insert(name.to_string(), definition);
        self
    }

    pub fn subgraph_instance(mut self, instance: SubgraphInstance) -> Self {
        self.subgraph_instances.push(instance);
        self
    }

    pub fn node(mut self, node: Node) -> Self {
        self.nodes.push(node);
        self
    }

    pub fn edge(mut self, from_node: &str, from_port: &str, to_node: &str, to_port: &str) -> Self {
        self.edges.push(Edge {
            id: None,
            kind: EdgeKind::Data,
            decision: None,
            from: PortRef { node_id: from_node.to_string(), port: from_port.to_string() },
            to: PortRef { node_id: to_node.to_string(), port: to_port.to_string() },
        });
        self
    }

    pub fn build(self) -> Graph {
        Graph {
            spec: self.spec,
            meta: self.meta,
            inputs: self.inputs,
            nondeterminism_allowed: self.nondeterminism_allowed,
            subgraphs: self.subgraphs,
            subgraph_instances: self.subgraph_instances,
            nodes: self.nodes,
            edges: self.edges,
        }
    }

    pub fn compile(self) -> Result<DagCompileResult, crate::GraphError> {
        let graph = self.build();
        compile_graph(&graph)
    }
}

pub struct NodeBuilder {
    id: String,
    kind: NodeKind,
    semantic_kind: SemanticNodeKind,
    inputs: Vec<String>,
    outputs: Vec<FileOutput>,
    params: ParamValue,
    timeout_ms: Option<u64>,
    resources: Option<Resources>,
    tags: Vec<String>,
    retry: RetryPolicy,
    effects: Vec<Effect>,
    env_allowlist: Vec<String>,
    group: Option<String>,
    trigger_rule: TriggerRule,
    branch: Option<BranchSpec>,
}

impl Default for NodeBuilder {
    fn default() -> Self {
        Self {
            id: String::new(),
            kind: NodeKind::Const,
            semantic_kind: SemanticNodeKind::Task,
            inputs: Vec::new(),
            outputs: Vec::new(),
            params: ParamValue::Literal(Value::Null),
            timeout_ms: None,
            resources: None,
            tags: Vec::new(),
            retry: RetryPolicy::default(),
            effects: Vec::new(),
            env_allowlist: Vec::new(),
            group: None,
            trigger_rule: TriggerRule::AllSuccess,
            branch: None,
        }
    }
}

impl NodeBuilder {
    pub fn new(id: &str, kind: NodeKind) -> Self {
        Self { id: id.to_string(), kind, ..Self::default() }
    }

    pub fn semantic_kind(mut self, value: SemanticNodeKind) -> Self {
        self.semantic_kind = value;
        self
    }

    pub fn input(mut self, name: &str) -> Self {
        self.inputs.push(name.to_string());
        self
    }

    pub fn output(mut self, name: &str, path: &str) -> Self {
        self.outputs.push(FileOutput::new(name.to_string(), path.to_string()));
        self
    }

    pub fn tag(mut self, value: &str) -> Self {
        self.tags.push(value.to_string());
        self
    }

    pub fn effect(mut self, value: Effect) -> Self {
        self.effects.push(value);
        self
    }

    pub fn group(mut self, value: &str) -> Self {
        self.group = Some(value.to_string());
        self
    }

    pub fn param_literal(mut self, value: Value) -> Self {
        self.params = ParamValue::Literal(value);
        self
    }

    pub fn trigger_rule(mut self, value: TriggerRule) -> Self {
        self.trigger_rule = value;
        self
    }

    pub fn branch(mut self, value: BranchSpec) -> Self {
        self.branch = Some(value);
        self
    }

    pub fn build(self) -> Node {
        Node {
            id: self.id,
            kind: self.kind,
            semantic_kind: self.semantic_kind,
            inputs: self.inputs,
            outputs: self.outputs,
            params: self.params,
            container: None,
            timeout_ms: self.timeout_ms,
            resources: self.resources,
            tags: self.tags,
            retry: self.retry,
            cache: Default::default(),
            effects: self.effects,
            env_allowlist: self.env_allowlist,
            group: self.group,
            trigger_rule: self.trigger_rule,
            branch: self.branch,
            dynamic: None,
        }
    }
}

pub fn lint_graph(graph: &Graph) -> Vec<DagLintFinding> {
    let mut findings = Vec::new();
    for node in &graph.nodes {
        if node.id.len() < 3 {
            findings.push(DagLintFinding {
                code: "LINT_NODE_ID_LENGTH".to_string(),
                message: format!("node '{}' id is too short for maintainability", node.id),
                severity: "warning".to_string(),
            });
        }
        if node.outputs.is_empty() {
            findings.push(DagLintFinding {
                code: "LINT_OUTPUT_MISSING".to_string(),
                message: format!("node '{}' has no declared outputs", node.id),
                severity: "error".to_string(),
            });
        }
        if node.retry.max_attempts > 0 && node.effects.contains(&Effect::Network) {
            findings.push(DagLintFinding {
                code: "LINT_RETRY_NETWORK".to_string(),
                message: format!("node '{}' retries with network side effects", node.id),
                severity: "warning".to_string(),
            });
        }
    }
    findings
}

pub fn simulate_graph(graph: &Graph) -> Vec<String> {
    let mut indegree = BTreeMap::<String, usize>::new();
    let mut adj = BTreeMap::<String, Vec<String>>::new();
    for node in &graph.nodes {
        indegree.insert(node.id.clone(), 0);
        adj.insert(node.id.clone(), Vec::new());
    }
    for edge in &graph.edges {
        *indegree.entry(edge.to.node_id.clone()).or_insert(0) += 1;
        adj.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
    let mut ready: BTreeSet<String> = indegree
        .iter()
        .filter_map(|(id, &d)| if d == 0 { Some(id.clone()) } else { None })
        .collect();
    let mut order = Vec::new();
    while let Some(id) = ready.iter().next().cloned() {
        ready.remove(&id);
        order.push(id.clone());
        for next in adj.get(&id).cloned().unwrap_or_default() {
            let d = indegree.entry(next.clone()).or_insert(0);
            *d = d.saturating_sub(1);
            if *d == 0 {
                ready.insert(next);
            }
        }
    }
    order
}

pub fn dry_run_preview(graph: &Graph) -> DagDryRunPreview {
    let compile_result = compile_graph(graph);
    let diagnostics = compile_result
        .as_ref()
        .map(|r| {
            r.diagnostics.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>()
        })
        .unwrap_or_else(|err: &crate::GraphError| vec![err.to_string()]);
    DagDryRunPreview {
        node_count: graph.nodes.len(),
        edge_count: graph.edges.len(),
        estimated_parallelism: simulate_graph(graph).len().max(1),
        compile_diagnostics: diagnostics,
    }
}

pub struct DagUnitHarness;

impl DagUnitHarness {
    pub fn parse(input: &str) -> Result<Graph, crate::GraphError> {
        parse_graph_strict(input)
    }

    pub fn run_lints(input: &str) -> Result<Vec<DagLintFinding>, crate::GraphError> {
        let graph = parse_graph_strict(input)?;
        Ok(lint_graph(&graph))
    }

    pub fn dry_run(input: &str) -> Result<DagDryRunPreview, crate::GraphError> {
        let graph = parse_graph_strict(input)?;
        Ok(dry_run_preview(&graph))
    }
}
