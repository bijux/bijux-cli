//! Core DAG model surface.

use serde::{de::Error as DeError, Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
    #[serde(default, skip_serializing_if = "semantic_kind_is_default")]
    pub semantic_kind: SemanticNodeKind,
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
    #[serde(default, skip_serializing_if = "cache_behavior_is_default")]
    pub cache: CacheBehavior,
    #[serde(default)]
    pub effects: Vec<Effect>,
    #[serde(default)]
    pub env_allowlist: Vec<String>,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default, skip_serializing_if = "trigger_rule_is_default")]
    pub trigger_rule: TriggerRule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub branch: Option<BranchSpec>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CacheBehavior {
    #[serde(default = "cache_behavior_enabled")]
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Default for CacheBehavior {
    fn default() -> Self {
        Self { enabled: true, reason: None }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum ParamValue {
    Ref(RefSpec),
    Array(Vec<ParamValue>),
    Object(BTreeMap<String, ParamValue>),
    Literal(Value),
}

impl Default for ParamValue {
    fn default() -> Self {
        Self::Literal(Value::Null)
    }
}

impl<'de> Deserialize<'de> for ParamValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        param_value_from_json(value).map_err(D::Error::custom)
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
    #[serde(alias = "path")]
    pub output_name: String,
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
            Self::Const => "const",
            Self::Shell => "shell",
            Self::Container => "container",
            Self::External(kind) => kind.as_str(),
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
        let value = String::deserialize(deserializer)?;
        Ok(match value.as_str() {
            "const" => Self::Const,
            "shell" => Self::Shell,
            "container" => Self::Container,
            _ => Self::External(value),
        })
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum SemanticNodeKind {
    #[default]
    Task,
    Branch,
    Barrier,
    Map,
    Reduce,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TriggerRule {
    AllSuccess,
    AnySuccess,
    AllDone,
    NoneFailed,
}

impl Default for TriggerRule {
    fn default() -> Self {
        Self::AllSuccess
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct BranchSpec {
    pub decisions: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_decision: Option<String>,
    pub decision_output: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum EdgeKind {
    #[default]
    Data,
    Control,
    Conditional,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Edge {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    #[serde(default, skip_serializing_if = "edge_kind_is_default")]
    pub kind: EdgeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<String>,
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

pub fn trigger_rule_is_default(rule: &TriggerRule) -> bool {
    matches!(rule, TriggerRule::AllSuccess)
}

pub fn semantic_kind_is_default(kind: &SemanticNodeKind) -> bool {
    matches!(kind, SemanticNodeKind::Task)
}

pub fn edge_kind_is_default(kind: &EdgeKind) -> bool {
    matches!(kind, EdgeKind::Data)
}

pub fn cache_behavior_is_default(cache: &CacheBehavior) -> bool {
    cache.enabled && cache.reason.is_none()
}

pub fn cache_behavior_enabled() -> bool {
    true
}

fn param_value_from_json(value: Value) -> Result<ParamValue, String> {
    match value {
        Value::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                out.push(param_value_from_json(item)?);
            }
            Ok(ParamValue::Array(out))
        }
        Value::Object(map) => {
            let ref_like = !map.is_empty()
                && map.keys().all(|key| matches!(key.as_str(), "graph_input" | "node_output"));
            if ref_like {
                let reference = serde_json::from_value(Value::Object(map))
                    .map_err(|error| error.to_string())?;
                Ok(ParamValue::Ref(reference))
            } else {
                let mut out = BTreeMap::new();
                for (key, item) in map {
                    out.insert(key, param_value_from_json(item)?);
                }
                Ok(ParamValue::Object(out))
            }
        }
        literal => Ok(ParamValue::Literal(literal)),
    }
}
