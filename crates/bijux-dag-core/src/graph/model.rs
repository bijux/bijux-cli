//! Core DAG model surface.

use serde::{Deserialize, Serialize};
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
        Self::Literal(Value::Null)
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
    pub expression: String,
    pub true_port: String,
    pub false_port: String,
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

pub fn edge_kind_is_default(kind: &EdgeKind) -> bool {
    matches!(kind, EdgeKind::Data)
}
