use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub const SPEC_VERSION: &str = "bijux-dag/v0.1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub spec: String,
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
    pub inputs: Vec<String>,
    pub outputs: Vec<String>,
    #[serde(default)]
    pub params: Value,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Const,
    Shell,
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

#[derive(Debug, thiserror::Error)]
pub enum GraphError {
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid spec version: {0}")]
    InvalidSpec(String),
    #[error("validation failed")]
    ValidationFailed,
}

pub fn parse_graph_strict(input: &str) -> Result<Graph, GraphError> {
    let graph: Graph = serde_json::from_str(input)?;
    if graph.spec != SPEC_VERSION {
        return Err(GraphError::InvalidSpec(graph.spec));
    }
    Ok(graph)
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

            if !from_node.outputs.iter().any(|p| p == &edge.from.port) {
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

        let mut output_names: HashMap<&str, &str> = HashMap::new();
        for node in &self.nodes {
            for out in &node.outputs {
                if let Some(prev) = output_names.insert(out.as_str(), node.id.as_str()) {
                    diags.push(error(
                        "E1008",
                        format!(
                            "output collision: {} and {} both declare {}",
                            prev, node.id, out
                        ),
                        format!("/nodes/{}/outputs", node.id),
                        Some("Make output names unique across nodes".to_string()),
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
            sort_value_maps(&mut node.params);
            node.inputs.sort();
            node.outputs.sort();
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

    pub fn node_fingerprint(&self, node: &Node) -> Result<String, GraphError> {
        let mut node = node.clone();
        sort_value_maps(&mut node.params);
        node.inputs.sort();
        node.outputs.sort();
        node.effects.sort_by_key(effect_order);
        node.env_allowlist.sort();
        let json = serde_json::to_string_pretty(&node)?;
        Ok(hash_bytes(json.as_bytes()))
    }

    pub fn resolve_params(&self) -> Result<BTreeMap<String, Value>, GraphError> {
        let mut resolved = BTreeMap::new();
        for node in &self.nodes {
            let val = resolve_value(&node.params, self);
            resolved.insert(node.id.clone(), val);
        }
        Ok(resolved)
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
        for node in &self.nodes {
            let mut stack = vec![&node.params];
            while let Some(v) = stack.pop() {
                match v {
                    Value::String(s) => {
                        if let Some(inner) = parse_ref(s) {
                            if inner.starts_with("graph.inputs.") {
                                let key = inner.trim_start_matches("graph.inputs.");
                                if !self.inputs.contains_key(key) {
                                    diags.push(error(
                                        "E1012",
                                        format!("unknown graph input ref: {}", key),
                                        format!("/nodes/{}/params", node.id),
                                        None,
                                    ));
                                }
                            } else if inner.starts_with("node:") {
                                let rest = inner.trim_start_matches("node:");
                                let parts: Vec<&str> = rest.split(".outputs.").collect();
                                if parts.len() != 2 {
                                    diags.push(error(
                                        "E1012",
                                        format!("invalid node output ref: {}", inner),
                                        format!("/nodes/{}/params", node.id),
                                        None,
                                    ));
                                } else {
                                    let node_id = parts[0];
                                    let port = parts[1];
                                    let target = self.nodes.iter().find(|n| n.id == node_id);
                                    if target.is_none()
                                        || !target.unwrap().outputs.iter().any(|p| p == port)
                                    {
                                        diags.push(error(
                                            "E1012",
                                            format!("unknown node output ref: {}", inner),
                                            format!("/nodes/{}/params", node.id),
                                            None,
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Value::Array(arr) => {
                        for x in arr {
                            stack.push(x);
                        }
                    }
                    Value::Object(map) => {
                        for v in map.values() {
                            stack.push(v);
                        }
                    }
                    _ => {}
                }
            }
        }
        diags
    }
}

fn parse_ref(s: &str) -> Option<&str> {
    if s.starts_with("${") && s.ends_with('}') {
        return Some(&s[2..s.len() - 1]);
    }
    None
}

fn resolve_value(value: &Value, graph: &Graph) -> Value {
    match value {
        Value::String(s) => {
            if let Some(inner) = parse_ref(s) {
                if inner.starts_with("graph.inputs.") {
                    let key = inner.trim_start_matches("graph.inputs.");
                    if let Some(v) = graph.inputs.get(key) {
                        return v.clone();
                    }
                } else if inner.starts_with("node:") {
                    return serde_json::json!({ "ref": inner });
                }
            }
            Value::String(s.clone())
        }
        Value::Array(arr) => {
            let mut out = Vec::new();
            for v in arr {
                out.push(resolve_value(v, graph));
            }
            Value::Array(out)
        }
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (k, v) in map {
                out.insert(k.clone(), resolve_value(v, graph));
            }
            Value::Object(out)
        }
        _ => value.clone(),
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

fn is_valid_node_id(id: &str) -> bool {
    id.chars()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_graph() -> Graph {
        Graph {
            spec: SPEC_VERSION.to_string(),
            inputs: serde_json::Map::new(),
            nondeterminism_allowed: false,
            nodes: vec![
                Node {
                    id: "a".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec![],
                    outputs: vec!["out".to_string()],
                    params: Value::Null,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
                },
                Node {
                    id: "b".to_string(),
                    kind: NodeKind::Const,
                    inputs: vec!["in".to_string()],
                    outputs: vec!["out".to_string()],
                    params: Value::Null,
                    timeout_ms: None,
                    resources: None,
                    tags: vec![],
                    retry: RetryPolicy::default(),
                    effects: vec![],
                    env_allowlist: vec![],
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
    fn fingerprint_changes_on_param() {
        let mut graph = base_graph();
        graph.nodes[0].params = serde_json::json!({"x": 1});
        let a = graph.graph_fingerprint().unwrap();
        graph.nodes[0].params = serde_json::json!({"x": 2});
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
    fn resolver_determinism() {
        let mut graph = base_graph();
        graph.inputs.insert("x".to_string(), serde_json::json!(1));
        graph.nodes[0].params = serde_json::json!({"ref": "${graph.inputs.x}"});
        let a = serde_json::to_string(&graph.resolve_params().unwrap()).unwrap();
        let b = serde_json::to_string(&graph.resolve_params().unwrap()).unwrap();
        assert_eq!(a, b);
    }
}
