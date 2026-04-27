use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    #[serde(default = "default_manifest_version")]
    pub manifest_version: String,
    pub run_id: String,
    pub created_unix_ms: u128,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub graph_snapshot: String,
    pub status: String,
    pub spec: String,
    pub graph_fingerprint: String,
    #[serde(default = "default_planner_contract_version")]
    pub planner_contract_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_fingerprint: Option<String>,
    pub tool_version: String,
    pub jobs: usize,
    pub adapters: Vec<AdapterInfo>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub outputs: Vec<OutputSummary>,
    pub node_counts: NodeCounts,
    pub policy: PolicyInfo,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_timeout_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_metadata: Option<RunMetadata>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_summary: Option<RunSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeTrace {
    pub node_id: String,
    pub status: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub attempt: u32,
    pub fingerprint: String,
    pub adapter_id: String,
    pub adapter_version: String,
    pub adapter_outputs_schema_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_binary_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<Resources>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inputs_index: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_params: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_proof: Option<CacheProof>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<SkipReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<FailureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_provenance: Option<ReplayProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunMetadata {
    pub submission_source: String,
    pub trigger_source: String,
    pub operator: String,
    #[serde(default)]
    pub labels: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    pub total_nodes: u32,
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayProvenance {
    pub node_action: String,
    pub source_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfo {
    pub kind: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheProof {
    pub hit: bool,
    pub key: String,
    pub source: String,
    pub verified: bool,
    pub reason: String,
    #[serde(default)]
    pub corrupt_detected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipReason {
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeCounts {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyInfo {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdapterInfo {
    pub adapter_id: String,
    pub adapter_version: String,
    pub effects: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resources {
    pub cpu: u32,
    pub mem_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputSummary {
    pub node_id: String,
    pub node_fingerprint: String,
    pub file: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactSchemaInfo {
    pub name: String,
    pub version: String,
    pub media_type: String,
    pub encoding: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunOutputsIndex {
    pub files: Vec<RunOutputFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunOutputFile {
    pub node_id: String,
    pub node_fingerprint: String,
    pub sha256: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Provenance {
    pub os: String,
    pub arch: String,
    pub rustc: String,
    pub tool_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_contract_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_fingerprint: Option<String>,
    pub adapters: Vec<AdapterInfo>,
    pub policy: PolicyInfo,
    pub time_source: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputsIndex {
    pub files: Vec<OutputFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputFile {
    pub path: String,
    pub sha256: String,
    pub node_id: String,
    pub node_fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerTrace {
    pub image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_digest: Option<String>,
    pub engine: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputsIndex {
    pub files: Vec<InputFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputFile {
    pub path: String,
    pub sha256: String,
    pub from_node: String,
    pub from_node_fingerprint: String,
    pub from_output: String,
}

fn default_manifest_version() -> String {
    "run-manifest/v0.1".to_string()
}

fn default_planner_contract_version() -> String {
    "bijux-dag-planner/v1".to_string()
}
