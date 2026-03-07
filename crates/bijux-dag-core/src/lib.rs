#![allow(
    clippy::cast_possible_truncation,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::pedantic,
    clippy::return_self_not_must_use,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unreadable_literal,
    clippy::unwrap_used
)]

#[cfg(test)]
use criterion as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};
#[cfg(test)]
use tempfile as _;

#[path = "build/builder.rs"]
pub mod builder;
#[path = "graph/canonical.rs"]
pub mod canonical;
#[path = "build/compile.rs"]
pub mod compile;
#[path = "graph/edge.rs"]
pub mod edge;
#[path = "analysis/effects.rs"]
pub mod effects;
#[path = "contracts/error.rs"]
pub mod error;
#[path = "analysis/fingerprint.rs"]
pub mod fingerprint;
#[path = "graph/graph.rs"]
pub mod graph;
#[path = "graph/meta.rs"]
pub mod meta;
#[path = "graph/model.rs"]
pub mod model;
#[path = "graph/node.rs"]
pub mod node;
#[path = "pipeline/parse.rs"]
pub mod parse;
#[path = "planner/planner.rs"]
pub mod planner;
#[path = "pipeline/resolve.rs"]
pub mod resolve;
#[path = "graph/resources.rs"]
pub mod resources;
#[path = "analysis/semantics.rs"]
pub mod semantics;
#[path = "graph/topology.rs"]
pub mod topology;
#[path = "pipeline/validate.rs"]
pub mod validate;
pub use builder::{
    dry_run_preview, lint_graph, simulate_graph, DagBuilder, DagDryRunPreview, DagLintFinding,
    DagUnitHarness, NodeBuilder,
};
pub use error::GraphError;
pub use parse::parse_graph_strict;
pub use planner::{
    can_runtime_execute_plan_without_raw_graph, graph_lowering_boundary_note,
    lower_graph_to_execution_plan, map_planner_error_to_graph_error, node_kind_supported,
    planner_alignment_required_doc, planner_alignment_required_schema,
    planner_alignment_required_test, planner_diagnostics_from_error, planner_identity_for_graph,
    ExecutionPlan, PlanOptions, PlannedEdge, PlannedNode, PlannerDiagnostic, PlannerError,
    PlannerSeverity, PLANNER_CONTRACT_VERSION,
};
pub use semantics::{
    classify_compatibility, complexity_score, enforce_late_binding_immutability, explain_graph,
    migration_patch, normalize_semantic_graph, semantic_diff, static_analysis, BranchDecisionNode,
    CompatibilityClassification, ConditionalExecution, DynamicEdgeExpansionRule,
    GraphComplexityScore, GraphCompositionContract, GraphExplainabilityModel, GraphMigrationPatch,
    GraphTemplate, JoinSemantics, LateBindingRule, MapFanOutSemantics, NormalizedSemanticGraph,
    ParameterBindingSemantics, PartitionSemantics, ReduceFanInSemantics, SemanticDiffClass,
    SemanticDiffReport, StaticAnalysisReport, SubgraphEmbedding, WindowingSemantics,
};

pub const SPEC_VERSION: &str = "bijux-dag/v0.1";
pub const CANONICALIZATION_CONTRACT_VERSION: &str = "bijux-dag-canonical/v1";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(transparent)]
pub struct GraphId(pub String);

impl GraphId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct GraphFingerprintExplain {
    pub graph_id: GraphId,
    pub canonical_json: String,
    pub canonical_json_bytes_len: usize,
    pub hash_algorithm: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub spec: String,
    #[serde(default)]
    pub meta: Option<GraphMeta>,
    #[serde(default)]
    pub inputs: serde_json::Map<String, Value>,
    #[serde(default)]
    pub nondeterminism_allowed: bool,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    pub id: String,
    pub kind: NodeKind,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<FileOutput>,
    #[serde(default)]
    pub params: ParamValue,
    #[serde(default)]
    pub container: Option<ContainerSpec>,
    #[serde(default)]
    pub timeout_ms: Option<u64>,
    #[serde(default)]
    pub resources: Option<Resources>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub retry: RetryPolicy,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub env_allowlist: Vec<String>,
    #[serde(default)]
    pub group: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GraphMeta {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub owners: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FileOutput {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    Ref(RefSpec),
    Array(Vec<ParamValue>),
    Object(BTreeMap<String, ParamValue>),
    Literal(Value),
}

impl Default for ParamValue {
    fn default() -> Self {
        ParamValue::Literal(Value::Null)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefSpec {
    #[serde(default)]
    pub graph_input: Option<String>,
    #[serde(default)]
    pub node_output: Option<NodeOutputRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeOutputRef {
    pub node_id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedGraph {
    pub graph: Graph,
    pub resolved_params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NodeKind {
    Const,
    Shell,
    Container,
    External(String),
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            NodeKind::Const => "const",
            NodeKind::Shell => "shell",
            NodeKind::Container => "container",
            NodeKind::External(s) => s.as_str(),
        }
    }
}

impl Serialize for NodeKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for NodeKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let kind = match s.as_str() {
            "const" => NodeKind::Const,
            "shell" => NodeKind::Shell,
            "container" => NodeKind::Container,
            _ => NodeKind::External(s),
        };
        Ok(kind)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Effect {
    Filesystem,
    Network,
    Env,
    Clock,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    pub from: PortRef,
    pub to: PortRef,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PortRef {
    pub node_id: String,
    pub port: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct Resources {
    pub cpu: u32,
    pub mem_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerSpec {
    pub image: String,
    pub argv: Vec<String>,
    #[serde(default)]
    pub env_allowlist: Vec<String>,
    #[serde(default)]
    pub workdir: Option<String>,
    pub engine: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub backoff_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationDiagnostic {
    pub code: String,
    pub message: String,
    pub path: String,
    pub hint: Option<String>,
    pub severity: Severity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Error,
    Warning,
}

impl Graph {
    pub fn validate_with_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diags = Vec::new();

        let mut ids = BTreeSet::new();
        let mut node_map: HashMap<&str, &Node> = HashMap::new();
        for node in &self.nodes {
            if !ids.insert(node.id.as_str()) {
                diags.push(error(
                    "E1001",
                    format!("duplicate node id: {}", node.id),
                    format!("/nodes/{}", node.id),
                    Some("Use unique node ids".to_string()),
                ));
            }
            if !is_valid_node_id(&node.id) {
                diags.push(error(
                    "E1007",
                    format!("illegal node id: {}", node.id),
                    format!("/nodes/{}", node.id),
                    Some("Use [a-zA-Z0-9_-] only".to_string()),
                ));
            }
            for tag in &node.tags {
                if !is_valid_canonical_name(tag) {
                    diags.push(error(
                        "E1026",
                        format!("illegal node tag: {}", tag),
                        format!("/nodes/{}/tags", node.id),
                        Some("Use [a-zA-Z0-9_-] only".to_string()),
                    ));
                }
            }
            if node.kind == NodeKind::Shell && node.effects.is_empty() {
                diags.push(error(
                    "E1009",
                    format!("missing effects for shell node: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Declare effects for shell nodes".to_string()),
                ));
            }
            if node.kind == NodeKind::Shell && !node.effects.contains(&Effect::Filesystem) {
                diags.push(error(
                    "E1009",
                    format!("shell node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for shell nodes".to_string()),
                ));
            }
            if node.kind == NodeKind::Container && node.container.is_none() {
                diags.push(error(
                    "E1023",
                    format!("missing container spec for node: {}", node.id),
                    format!("/nodes/{}/container", node.id),
                    Some("Provide container spec for container nodes".to_string()),
                ));
            }
            if node.kind == NodeKind::Container && !node.effects.contains(&Effect::Filesystem) {
                diags.push(error(
                    "E1009",
                    format!("container node missing filesystem effect: {}", node.id),
                    format!("/nodes/{}/effects", node.id),
                    Some("Include filesystem effect for container nodes".to_string()),
                ));
            }
            if node.retry.max_attempts > 0
                && (node.effects.contains(&Effect::Clock)
                    || node.effects.contains(&Effect::Network))
            {
                let has_seed = self.inputs.contains_key("random_seed");
                if !has_seed && !self.nondeterminism_allowed {
                    diags.push(error(
                        "E1011",
                        format!("retry not allowed for nondeterministic node: {}", node.id),
                        format!("/nodes/{}/retry", node.id),
                        Some(
                            "Provide inputs.random_seed or set nondeterminism_allowed=true"
                                .to_string(),
                        ),
                    ));
                }
            }
            if !node.env_allowlist.is_empty() && !node.effects.contains(&Effect::Env) {
                diags.push(error(
                    "E1010",
                    format!("env_allowlist without env effect: {}", node.id),
                    format!("/nodes/{}/env_allowlist", node.id),
                    Some("Add env effect when using env_allowlist".to_string()),
                ));
            }
            if node.kind == NodeKind::Container {
                if let Some(ref spec) = node.container {
                    if !spec.env_allowlist.is_empty() && !node.effects.contains(&Effect::Env) {
                        diags.push(error(
                            "E1010",
                            format!("container env_allowlist without env effect: {}", node.id),
                            format!("/nodes/{}/container/env_allowlist", node.id),
                            Some("Add env effect when using env_allowlist".to_string()),
                        ));
                    }
                    if spec.engine != "docker" && spec.engine != "podman" {
                        diags.push(error(
                            "E1024",
                            format!("invalid container engine: {}", spec.engine),
                            format!("/nodes/{}/container/engine", node.id),
                            Some("Use engine \"docker\" or \"podman\"".to_string()),
                        ));
                    }
                    if spec.argv.is_empty() {
                        diags.push(error(
                            "E1024",
                            format!("container argv must not be empty: {}", node.id),
                            format!("/nodes/{}/container/argv", node.id),
                            Some("Provide argv for container nodes".to_string()),
                        ));
                    }
                }
            }
            node_map.insert(node.id.as_str(), node);
        }

        let mut edge_pairs = BTreeSet::new();
        for edge in &self.edges {
            let from_node = node_map.get(edge.from.node_id.as_str());
            let to_node = node_map.get(edge.to.node_id.as_str());
            if from_node.is_none() {
                diags.push(error(
                    "E1002",
                    format!("dangling node reference: {}", edge.from.node_id),
                    format!("/edges/from/{}", edge.from.node_id),
                    None,
                ));
                continue;
            }
            if to_node.is_none() {
                diags.push(error(
                    "E1002",
                    format!("dangling node reference: {}", edge.to.node_id),
                    format!("/edges/to/{}", edge.to.node_id),
                    None,
                ));
                continue;
            }
            let from_node = from_node.unwrap();
            let to_node = to_node.unwrap();

            if !from_node.outputs.iter().any(|p| p.name == edge.from.port) {
                diags.push(error(
                    "E1003",
                    format!(
                        "dangling port reference: {}.{}",
                        edge.from.node_id, edge.from.port
                    ),
                    format!("/edges/from/{}/{}", edge.from.node_id, edge.from.port),
                    None,
                ));
            }
            if !to_node.inputs.iter().any(|p| p == &edge.to.port) {
                diags.push(error(
                    "E1003",
                    format!(
                        "dangling port reference: {}.{}",
                        edge.to.node_id, edge.to.port
                    ),
                    format!("/edges/to/{}/{}", edge.to.node_id, edge.to.port),
                    None,
                ));
            }

            let pair_key = format!(
                "{}:{}->{}:{}",
                edge.from.node_id, edge.from.port, edge.to.node_id, edge.to.port
            );
            if !edge_pairs.insert(pair_key) {
                diags.push(error(
                    "E1008",
                    "output collision".to_string(),
                    "/edges".to_string(),
                    Some("Avoid duplicate edge targets".to_string()),
                ));
            }
        }

        for node in &self.nodes {
            let mut seen = BTreeSet::new();
            for out in &node.outputs {
                if !is_valid_output_path(&out.path) {
                    diags.push(error(
                        "E1025",
                        format!("invalid output path: {}", out.path),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Use relative paths without '..'".to_string()),
                    ));
                }
                if !seen.insert(out.name.as_str()) {
                    diags.push(error(
                        "E1008",
                        format!("duplicate output name: {}", out.name),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Make output names unique per node".to_string()),
                    ));
                }
            }
        }

        let mut output_paths = BTreeSet::new();
        for node in &self.nodes {
            for out in &node.outputs {
                if !output_paths.insert(out.path.as_str()) {
                    diags.push(error(
                        "E1008",
                        format!("output collision: {}", out.path),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Avoid duplicate output paths across nodes".to_string()),
                    ));
                }
            }
        }

        if self.has_cycle() {
            diags.push(error(
                "E1004",
                "cycle detected".to_string(),
                "/edges".to_string(),
                None,
            ));
        }

        diags.extend(self.validate_param_refs());
        diags.extend(self.unreachable_warnings());
        diags.extend(self.orphan_warnings());
        diags.extend(self.validate_graph_meta_names());

        diags
    }

    pub fn validate_strict(&self) -> Result<Vec<ValidationDiagnostic>, GraphError> {
        let diags = self.validate_with_warnings();
        if diags.iter().any(|d| d.severity == Severity::Error) {
            return Err(GraphError::ValidationFailed);
        }
        Ok(diags)
    }

    pub fn canonicalize(&self) -> Graph {
        let mut nodes = self.nodes.clone();
        let mut edges = self.edges.clone();

        nodes.sort_by(|a, b| a.id.cmp(&b.id));
        for node in &mut nodes {
            sort_param_value(&mut node.params);
            node.inputs.sort();
            for out in &mut node.outputs {
                out.path = normalize_rel_path(&out.path);
            }
            node.outputs.sort_by(|a, b| a.name.cmp(&b.name));
            node.effects.sort_by_key(effect_order);
            node.env_allowlist.sort();
            node.tags.sort();
        }

        edges.sort_by(|a, b| {
            (&a.from.node_id, &a.from.port, &a.to.node_id, &a.to.port).cmp(&(
                &b.from.node_id,
                &b.from.port,
                &b.to.node_id,
                &b.to.port,
            ))
        });

        let mut inputs = self.inputs.clone();
        let mut inputs_value = Value::Object(inputs.clone());
        sort_value_maps(&mut inputs_value);
        if let Value::Object(map) = inputs_value {
            inputs = map;
        }
        Graph {
            spec: self.spec.clone(),
            meta: self.meta.clone(),
            inputs,
            nondeterminism_allowed: self.nondeterminism_allowed,
            nodes,
            edges,
        }
    }

    pub fn to_canonical_json(&self) -> Result<String, GraphError> {
        let canonical = self.canonicalize();
        Ok(serde_json::to_string_pretty(&canonical)?)
    }

    pub fn topo_order(&self) -> Result<Vec<String>, GraphError> {
        let mut indegree: HashMap<String, usize> = HashMap::new();
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();
        for node in &self.nodes {
            indegree.insert(node.id.clone(), 0);
            adj.insert(node.id.clone(), Vec::new());
        }
        for edge in &self.edges {
            let from = edge.from.node_id.clone();
            let to = edge.to.node_id.clone();
            if let Some(v) = adj.get_mut(&from) {
                v.push(to.clone());
            }
            if let Some(d) = indegree.get_mut(&to) {
                *d += 1;
            }
        }

        let mut ready: BTreeSet<String> = indegree
            .iter()
            .filter_map(|(id, &deg)| if deg == 0 { Some(id.clone()) } else { None })
            .collect();
        let mut order = Vec::new();

        while let Some(id) = ready.iter().next().cloned() {
            ready.remove(&id);
            order.push(id.clone());
            if let Some(neighbors) = adj.get(&id) {
                for n in neighbors {
                    if let Some(deg) = indegree.get_mut(n) {
                        *deg -= 1;
                        if *deg == 0 {
                            ready.insert(n.clone());
                        }
                    }
                }
            }
        }

        if order.len() != indegree.len() {
            return Err(GraphError::ValidationFailed);
        }

        Ok(order)
    }

    pub fn graph_fingerprint(&self) -> Result<String, GraphError> {
        let json = self.to_canonical_json()?;
        Ok(hash_bytes(json.as_bytes()))
    }

    pub fn graph_id(&self) -> Result<GraphId, GraphError> {
        Ok(GraphId(self.graph_fingerprint()?))
    }

    pub fn graph_fingerprint_explain(&self) -> Result<GraphFingerprintExplain, GraphError> {
        let canonical_json = self.to_canonical_json()?;
        let graph_id = GraphId(hash_bytes(canonical_json.as_bytes()));
        Ok(GraphFingerprintExplain {
            graph_id,
            canonical_json_bytes_len: canonical_json.len(),
            canonical_json,
            hash_algorithm: "sha256".to_string(),
        })
    }

    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, GraphError> {
        Ok(self.to_canonical_json()?.into_bytes())
    }

    pub fn canonical_graph_pretty(&self) -> Result<String, GraphError> {
        self.to_canonical_json()
    }

    pub fn node_fingerprint(&self, node: &Node) -> Result<String, GraphError> {
        let resolved = resolve_param_value(&node.params, self)?;
        self.node_fingerprint_with_params(node, &resolved)
    }

    pub fn node_fingerprint_with_params(
        &self,
        node: &Node,
        resolved_params: &Value,
    ) -> Result<String, GraphError> {
        let mut node = node.clone();
        let mut params = resolved_params.clone();
        sort_value_maps(&mut params);
        node.params = ParamValue::Literal(params);
        node.inputs.sort();
        for out in &mut node.outputs {
            out.path = normalize_rel_path(&out.path);
        }
        node.outputs.sort_by(|a, b| a.name.cmp(&b.name));
        node.effects.sort_by_key(effect_order);
        node.env_allowlist.sort();
        node.group = None;
        let json = serde_json::to_string_pretty(&node)?;
        Ok(hash_bytes(json.as_bytes()))
    }

    pub fn resolve_graph(&self) -> Result<ResolvedGraph, GraphError> {
        let mut resolved = BTreeMap::new();
        for node in &self.nodes {
            let mut val = resolve_param_value(&node.params, self)?;
            sort_value_maps(&mut val);
            resolved.insert(node.id.clone(), val);
        }
        Ok(ResolvedGraph {
            graph: self.clone(),
            resolved_params: resolved,
        })
    }

    fn has_cycle(&self) -> bool {
        self.topo_order().is_err()
    }

    fn unreachable_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diags = Vec::new();
        let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
        for edge in &self.edges {
            adj.entry(edge.from.node_id.as_str())
                .or_default()
                .push(edge.to.node_id.as_str());
        }
        let mut visited: BTreeSet<&str> = BTreeSet::new();
        let mut stack: Vec<&str> = self
            .nodes
            .iter()
            .filter_map(|n| {
                if n.inputs.is_empty() {
                    Some(n.id.as_str())
                } else {
                    None
                }
            })
            .collect();
        while let Some(id) = stack.pop() {
            if !visited.insert(id) {
                continue;
            }
            if let Some(next) = adj.get(id) {
                for n in next {
                    stack.push(n);
                }
            }
        }
        for node in &self.nodes {
            if !visited.contains(node.id.as_str()) {
                diags.push(warn(
                    "W2001",
                    format!("unreachable node: {}", node.id),
                    format!("/nodes/{}", node.id),
                    None,
                ));
            }
        }
        diags
    }

    fn orphan_warnings(&self) -> Vec<ValidationDiagnostic> {
        let mut diags = Vec::new();
        let mut involved: BTreeSet<&str> = BTreeSet::new();
        for edge in &self.edges {
            involved.insert(edge.from.node_id.as_str());
            involved.insert(edge.to.node_id.as_str());
        }
        for node in &self.nodes {
            if !involved.contains(node.id.as_str()) {
                diags.push(warn(
                    "W2002",
                    format!("orphan node: {}", node.id),
                    format!("/nodes/{}", node.id),
                    None,
                ));
            }
        }
        diags
    }

    fn validate_param_refs(&self) -> Vec<ValidationDiagnostic> {
        let mut diags = Vec::new();
        let order = self.topo_order().ok();
        let mut index: HashMap<&str, usize> = HashMap::new();
        if let Some(order) = order.as_ref() {
            for (i, id) in order.iter().enumerate() {
                index.insert(id.as_str(), i);
            }
        }
        for node in &self.nodes {
            validate_param_value(
                &node.params,
                self,
                node,
                &index,
                &mut diags,
                &format!("/nodes/{}/params", node.id),
            );
        }
        diags
    }

    fn validate_graph_meta_names(&self) -> Vec<ValidationDiagnostic> {
        let mut diags = Vec::new();
        if let Some(meta) = &self.meta {
            if !is_valid_canonical_name(&meta.name) {
                diags.push(error(
                    "E1027",
                    format!("illegal graph name: {}", meta.name),
                    "/meta/name".to_string(),
                    Some("Use [a-zA-Z0-9_-] only".to_string()),
                ));
            }
            for tag in &meta.tags {
                if !is_valid_canonical_name(tag) {
                    diags.push(error(
                        "E1026",
                        format!("illegal graph tag: {}", tag),
                        "/meta/tags".to_string(),
                        Some("Use [a-zA-Z0-9_-] only".to_string()),
                    ));
                }
            }
        }
        diags
    }
}

fn resolve_param_value(value: &ParamValue, graph: &Graph) -> Result<Value, GraphError> {
    match value {
        ParamValue::Literal(v) => Ok(v.clone()),
        ParamValue::Array(arr) => {
            let mut out = Vec::new();
            for v in arr {
                out.push(resolve_param_value(v, graph)?);
            }
            Ok(Value::Array(out))
        }
        ParamValue::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), resolve_param_value(v, graph)?);
            }
            Ok(Value::Object(out))
        }
        ParamValue::Ref(spec) => resolve_ref(spec, graph),
    }
}

fn resolve_ref(spec: &RefSpec, graph: &Graph) -> Result<Value, GraphError> {
    if let Some(input) = &spec.graph_input {
        if let Some(v) = graph.inputs.get(input) {
            return Ok(v.clone());
        }
        return Err(GraphError::ValidationFailed);
    }
    if let Some(node_out) = &spec.node_output {
        let target = graph.nodes.iter().find(|n| n.id == node_out.node_id);
        if let Some(node) = target {
            if let Some(out) = node.outputs.iter().find(|o| o.name == node_out.path) {
                return Ok(Value::String(out.path.clone()));
            }
        }
        return Err(GraphError::ValidationFailed);
    }
    Err(GraphError::ValidationFailed)
}

fn validate_param_value(
    value: &ParamValue,
    graph: &Graph,
    node: &Node,
    order: &HashMap<&str, usize>,
    diags: &mut Vec<ValidationDiagnostic>,
    path: &str,
) {
    match value {
        ParamValue::Ref(spec) => {
            if let Some(input) = &spec.graph_input {
                if !graph.inputs.contains_key(input) {
                    diags.push(error(
                        "E1020",
                        format!("unknown graph input ref: {}", input),
                        path.to_string(),
                        None,
                    ));
                }
            }
            if let Some(node_out) = &spec.node_output {
                let target = graph.nodes.iter().find(|n| n.id == node_out.node_id);
                if target.is_none()
                    || !target
                        .unwrap()
                        .outputs
                        .iter()
                        .any(|p| p.name == node_out.path)
                {
                    diags.push(error(
                        "E1021",
                        format!(
                            "unknown node output ref: {}.{}",
                            node_out.node_id, node_out.path
                        ),
                        path.to_string(),
                        None,
                    ));
                }
                if let (Some(&src), Some(&cur)) = (
                    order.get(node_out.node_id.as_str()),
                    order.get(node.id.as_str()),
                ) {
                    if src >= cur {
                        diags.push(error(
                            "E1022",
                            format!(
                                "forward node output ref: {}.{}",
                                node_out.node_id, node_out.path
                            ),
                            path.to_string(),
                            None,
                        ));
                    }
                }
            }
        }
        ParamValue::Array(arr) => {
            for (i, v) in arr.iter().enumerate() {
                validate_param_value(v, graph, node, order, diags, &format!("{}/{}", path, i));
            }
        }
        ParamValue::Object(map) => {
            for (k, v) in map {
                validate_param_value(v, graph, node, order, diags, &format!("{}/{}", path, k));
            }
        }
        ParamValue::Literal(_) => {}
    }
}

fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let result = hasher.finalize();
    hex::encode(result)
}

fn sort_value_maps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut sorted: BTreeMap<String, Value> = BTreeMap::new();
            let entries = std::mem::take(map);
            for (k, mut v) in entries {
                sort_value_maps(&mut v);
                sorted.insert(k, v);
            }
            let mut new_map = serde_json::Map::new();
            for (k, v) in sorted {
                new_map.insert(k, v);
            }
            *map = new_map;
        }
        Value::Array(arr) => {
            for v in arr.iter_mut() {
                sort_value_maps(v);
            }
        }
        _ => {}
    }
}

fn sort_param_value(value: &mut ParamValue) {
    match value {
        ParamValue::Array(arr) => {
            for v in arr.iter_mut() {
                sort_param_value(v);
            }
        }
        ParamValue::Object(map) => {
            let mut sorted: BTreeMap<String, ParamValue> = BTreeMap::new();
            let entries = std::mem::take(map);
            for (k, mut v) in entries {
                sort_param_value(&mut v);
                sorted.insert(k, v);
            }
            *map = sorted;
        }
        ParamValue::Ref(_) | ParamValue::Literal(_) => {}
    }
}

fn is_valid_node_id(id: &str) -> bool {
    is_valid_canonical_name(id)
}

fn is_valid_canonical_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn error(code: &str, message: String, path: String, hint: Option<String>) -> ValidationDiagnostic {
    ValidationDiagnostic {
        code: code.to_string(),
        message,
        path,
        hint,
        severity: Severity::Error,
    }
}

fn warn(code: &str, message: String, path: String, hint: Option<String>) -> ValidationDiagnostic {
    ValidationDiagnostic {
        code: code.to_string(),
        message,
        path,
        hint,
        severity: Severity::Warning,
    }
}

fn effect_order(effect: &Effect) -> u8 {
    match effect {
        Effect::Filesystem => 0,
        Effect::Network => 1,
        Effect::Env => 2,
        Effect::Clock => 3,
    }
}

fn is_valid_output_path(path: &str) -> bool {
    if path.contains("..") {
        return false;
    }
    let normalized = normalize_rel_path(path);
    if normalized.starts_with('/') {
        return false;
    }
    if normalized.len() > 2 {
        let bytes = normalized.as_bytes();
        if bytes[1] == b':' && bytes[2] == b'/' {
            return false;
        }
    }
    true
}

fn normalize_rel_path(path: &str) -> String {
    path.replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_graph() -> Graph {
        Graph {
            spec: SPEC_VERSION.to_string(),
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
                    params: ParamValue::default(),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput {
                        name: "out".to_string(),
                        path: "out".to_string(),
                    }],
                    params: ParamValue::default(),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                    group: None,
                },
            ],
            edges: vec![Edge {
                from: PortRef {
                    node_id: "a".to_string(),
                    port: "out".to_string(),
                },
                to: PortRef {
                    node_id: "b".to_string(),
                    port: "in".to_string(),
                },
            }],
        }
    }

    #[test]
    fn fingerprint_stable_under_reorder() {
        let graph = base_graph();
        let mut graph2 = base_graph();
        graph2.nodes.reverse();
        let a = graph.graph_fingerprint().unwrap();
        let b = graph2.graph_fingerprint().unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn canonicalization_stable_over_many_reorders() {
        let graph = base_graph();
        let canonical = graph.to_canonical_json().unwrap();
        let fp = graph.graph_fingerprint().unwrap();
        for seed in 0..50u64 {
            let mut g = base_graph();
            shuffle_with_seed(&mut g.nodes[..], seed + 1);
            shuffle_with_seed(&mut g.edges[..], seed + 101);
            let c = g.to_canonical_json().unwrap();
            let f = g.graph_fingerprint().unwrap();
            assert_eq!(c, canonical);
            assert_eq!(f, fp);
        }
    }

    fn shuffle_with_seed<T>(items: &mut [T], mut seed: u64) {
        if items.len() <= 1 {
            return;
        }
        for i in (1..items.len()).rev() {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let j = (seed % (i as u64 + 1)) as usize;
            items.swap(i, j);
        }
    }

    #[test]
    fn fingerprint_changes_on_param() {
        let mut graph = base_graph();
        graph.nodes[0].params = ParamValue::Object(
            [("x".to_string(), ParamValue::Literal(Value::from(1)))]
                .into_iter()
                .collect(),
        );
        let a = graph.graph_fingerprint().unwrap();
        graph.nodes[0].params = ParamValue::Object(
            [("x".to_string(), ParamValue::Literal(Value::from(2)))]
                .into_iter()
                .collect(),
        );
        let b = graph.graph_fingerprint().unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn canonicalize_stable_bytes() {
        let graph = base_graph();
        let mut last = graph.to_canonical_json().unwrap();
        for _ in 0..50 {
            let cur = graph.to_canonical_json().unwrap();
            assert_eq!(last, cur);
            last = cur;
        }
    }

    #[test]
    fn canonicalization_stable_under_random_ordering() {
        let graph = base_graph();
        let canonical = graph.to_canonical_json().unwrap();
        let fp = graph.graph_fingerprint().unwrap();
        for seed in 1..25u64 {
            let mut g = base_graph();
            shuffle(&mut g.nodes, seed);
            shuffle(&mut g.edges, seed.wrapping_mul(7));
            let cur = g.to_canonical_json().unwrap();
            let cur_fp = g.graph_fingerprint().unwrap();
            assert_eq!(canonical, cur);
            assert_eq!(fp, cur_fp);
        }
    }

    #[test]
    fn resolver_determinism() {
        let mut graph = base_graph();
        graph.inputs.insert("x".to_string(), serde_json::json!(1));
        graph.nodes[0].params = ParamValue::Ref(RefSpec {
            graph_input: Some("x".to_string()),
            node_output: None,
        });
        let a = serde_json::to_string(&graph.resolve_graph().unwrap().resolved_params).unwrap();
        let b = serde_json::to_string(&graph.resolve_graph().unwrap().resolved_params).unwrap();
        assert_eq!(a, b);
    }

    fn shuffle<T>(items: &mut [T], mut seed: u64) {
        let len = items.len();
        if len < 2 {
            return;
        }
        for i in (1..len).rev() {
            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let j = (seed as usize) % (i + 1);
            items.swap(i, j);
        }
    }
}
