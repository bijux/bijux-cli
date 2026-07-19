//! Core DAG model surface.

use crate::dynamic::DynamicSpec;
use crate::input::{materialize_graph_input_value, GraphInputSpec, GraphInputViolation};
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

pub const PATH_VARIABLE_NAMES: &[&str] =
    &["run_dir", "work_dir", "inputs_dir", "outputs_dir", "cache_dir"];

pub fn is_known_path_variable(name: &str) -> bool {
    PATH_VARIABLE_NAMES.iter().any(|candidate| candidate == &name)
}

pub fn env_allowlist_pattern_is_exact(pattern: &str) -> bool {
    !pattern.contains('*')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Graph {
    pub spec: String,
    #[serde(default)]
    pub meta: Option<GraphMeta>,
    #[serde(default)]
    pub inputs: BTreeMap<String, GraphInputSpec>,
    #[serde(default)]
    pub nondeterminism_allowed: bool,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub subgraphs: BTreeMap<String, SubgraphDefinition>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub subgraph_instances: Vec<SubgraphInstance>,
    pub nodes: Vec<Node>,
    pub edges: Vec<Edge>,
}

impl Graph {
    pub fn effective_inputs(&self) -> Result<BTreeMap<String, Value>, GraphInputViolation> {
        let mut effective = BTreeMap::new();
        for (input_name, spec) in &self.inputs {
            if let Some(value) = spec.effective_value() {
                effective.insert(
                    input_name.clone(),
                    materialize_graph_input_value(spec, value, &format!("/inputs/{input_name}"))?,
                );
            }
        }
        Ok(effective)
    }

    pub fn input_schema(&self) -> BTreeMap<String, Value> {
        self.inputs
            .iter()
            .map(|(input_name, spec)| (input_name.clone(), spec.schema_json()))
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubgraphDefinition {
    pub graph: Graph,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub outputs: BTreeMap<String, NodeOutputRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubgraphInstance {
    pub id: String,
    pub subgraph: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub input_bindings: BTreeMap<String, ParamValue>,
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
    pub outputs: Vec<OutputSpec>,
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dynamic: Option<DynamicSpec>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum OutputKind {
    #[default]
    File,
    Directory,
    Value,
    Table,
    Log,
    Binary,
    Bundle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct OutputSpec {
    pub name: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "output_kind_is_default")]
    pub kind: OutputKind,
    #[serde(
        default = "output_required_default",
        skip_serializing_if = "output_required_is_default"
    )]
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
}

impl OutputSpec {
    pub fn new(name: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            path: path.into(),
            kind: OutputKind::File,
            required: true,
            media_type: None,
            promotable: false,
        }
    }

    pub fn effective_media_type(&self) -> String {
        self.media_type
            .clone()
            .unwrap_or_else(|| default_media_type_for_kind(&self.kind).to_string())
    }

    pub fn expects_directory(&self) -> bool {
        matches!(self.kind, OutputKind::Directory)
    }

    pub fn expects_file(&self) -> bool {
        !self.expects_directory()
    }
}

pub type FileOutput = OutputSpec;

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
    #[serde(default)]
    pub path_var: Option<PathVarRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeOutputRef {
    pub node_id: String,
    #[serde(alias = "path")]
    pub output_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum PathVarRef {
    Name(String),
    Binding(PathVarBinding),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PathVarBinding {
    pub name: String,
    #[serde(default)]
    pub relative_path: Option<String>,
}

impl PathVarRef {
    pub fn name(&self) -> &str {
        match self {
            Self::Name(name) => name,
            Self::Binding(binding) => &binding.name,
        }
    }

    pub fn relative_path(&self) -> Option<&str> {
        match self {
            Self::Name(_) => None,
            Self::Binding(binding) => binding.relative_path.as_deref(),
        }
    }

    pub fn display_path(&self) -> String {
        match self.relative_path() {
            Some(relative_path) => format!("{{{}}}/{}", self.name(), relative_path),
            None => format!("{{{}}}", self.name()),
        }
    }
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
    Python,
    Http,
    FileTransform,
    Container,
    External(String),
}

impl NodeKind {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Const => "const",
            Self::Shell => "shell",
            Self::Python => "python",
            Self::Http => "http",
            Self::FileTransform => "file_transform",
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
            "python" => Self::Python,
            "http" => Self::Http,
            "file_transform" => Self::FileTransform,
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
    Dynamic,
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
    #[serde(default)]
    pub gpu_devices: u32,
    #[serde(default, skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub named_resources: std::collections::BTreeMap<String, u32>,
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

pub fn output_kind_is_default(kind: &OutputKind) -> bool {
    matches!(kind, OutputKind::File)
}

pub fn output_required_default() -> bool {
    true
}

pub fn output_required_is_default(required: &bool) -> bool {
    *required
}

fn is_false(value: &bool) -> bool {
    !value
}

pub fn default_media_type_for_kind(kind: &OutputKind) -> &'static str {
    match kind {
        OutputKind::File => "application/octet-stream",
        OutputKind::Directory => "application/vnd.bijux.directory",
        OutputKind::Value => "application/json",
        OutputKind::Table => "application/vnd.bijux.table",
        OutputKind::Log => "text/plain",
        OutputKind::Binary => "application/octet-stream",
        OutputKind::Bundle => "application/vnd.bijux.bundle",
    }
}

#[cfg(test)]
mod tests {
    use super::{OutputKind, OutputSpec};
    use serde_json::json;

    #[test]
    fn output_spec_defaults_to_non_promotable() {
        let output = OutputSpec::new("report", "report.json");
        assert!(!output.promotable);
    }

    #[test]
    fn output_spec_deserializes_promotable_outputs() {
        let output: OutputSpec = serde_json::from_value(json!({
            "name": "deliverable",
            "path": "publish/report.json",
            "kind": "file",
            "required": true,
            "promotable": true
        }))
        .expect("output spec");
        assert!(output.promotable);
        assert_eq!(output.kind, OutputKind::File);
    }
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
                && map
                    .keys()
                    .all(|key| matches!(key.as_str(), "graph_input" | "node_output" | "path_var"));
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
