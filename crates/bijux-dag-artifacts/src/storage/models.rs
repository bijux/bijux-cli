use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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
    pub run_timeout_behavior: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_cancellation_cause: Option<String>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub planner_contract_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_fingerprint: Option<String>,
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
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<NodeLogEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<NodeLogEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<TraceOutputArtifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ContainerTrace>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_proof: Option<CacheProof>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_identity: Option<CacheIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_decision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trigger_evaluation: Option<TriggerEvaluation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skip_reason: Option<SkipReason>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure: Option<FailureInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transition_cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_state: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lifecycle_transitions: Vec<NodeLifecycleTransition>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_provenance: Option<ReplayProvenance>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeLogEvidence {
    pub path: String,
    pub size_bytes: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tail_lines: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeLifecycleTransition {
    pub from_state: String,
    pub to_state: String,
    pub cause: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriggerEvaluation {
    pub trigger_rule: String,
    pub satisfied: bool,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_statuses: Vec<TriggerParentStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TriggerParentStatus {
    pub node_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub graph_inputs: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunStopRequest {
    #[serde(default = "default_run_stop_request_version")]
    pub schema_version: String,
    pub run_id: String,
    pub requested_unix_ms: u128,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunSummary {
    pub total_nodes: u32,
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cancelled: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub promoted_outputs: Vec<PromotedOutputSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotedOutputSummary {
    pub canonical_artifact_id: String,
    pub legacy_artifact_id: String,
    pub node_id: String,
    pub output_name: String,
    pub artifact_sha256: String,
    pub destination_path: String,
    pub target_environment: String,
    pub promoted_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunDirSchemaIndex {
    pub schema_version: String,
    pub manifest_version: String,
    pub manifest_schema: String,
    pub node_trace_schema: String,
    pub inputs_index_schema: String,
    pub outputs_index_schema: String,
    pub lineage_schema_version: String,
    pub timeline_schema_version: String,
    pub event_log_schema_version: String,
    pub required_root_files: Vec<String>,
    pub required_node_files: Vec<String>,
    pub optional_root_files: Vec<String>,
}

impl Default for RunDirSchemaIndex {
    fn default() -> Self {
        Self {
            schema_version: "run-dir-schema/v0.1".to_string(),
            manifest_version: default_manifest_version(),
            manifest_schema: "configs/dag/schema/run_manifest.schema.json".to_string(),
            node_trace_schema: "configs/dag/schema/node_trace.schema.json".to_string(),
            inputs_index_schema: "configs/dag/schema/inputs_index.schema.json".to_string(),
            outputs_index_schema: "configs/dag/schema/outputs_index.schema.json".to_string(),
            lineage_schema_version: "lineage/v0.1".to_string(),
            timeline_schema_version: "v0.1".to_string(),
            event_log_schema_version: "runtime-events/v0.1".to_string(),
            required_root_files: vec![
                "manifest.json".to_string(),
                "graph.snapshot.json".to_string(),
                "outputs/index.json".to_string(),
                "provenance.json".to_string(),
                "lineage.snapshot.json".to_string(),
                "observability.events.json".to_string(),
                "observability.timeline.json".to_string(),
                "run.log.jsonl".to_string(),
                "run.schema.json".to_string(),
            ],
            required_node_files: vec![
                "trace.json".to_string(),
                "attempts.json".to_string(),
                "resolved_params.json".to_string(),
                "inputs/index.json".to_string(),
                "outputs/index.json".to_string(),
            ],
            optional_root_files: vec![
                "manifest.finalized.json".to_string(),
                ".run-complete.json".to_string(),
                ".run-incomplete.json".to_string(),
                "observability.root-causes.json".to_string(),
                "observability.metrics.json".to_string(),
                "run.audit.json".to_string(),
                "run.stop-request.json".to_string(),
            ],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayProvenance {
    pub node_action: String,
    pub source_run_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfo {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub class: Option<FailureClass>,
    pub kind: String,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailureClass {
    User,
    Infrastructure,
    Execution,
    Timeout,
    Policy,
}

impl FailureClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Infrastructure => "infrastructure",
            Self::Execution => "execution",
            Self::Timeout => "timeout",
            Self::Policy => "policy",
        }
    }
}

impl FailureInfo {
    pub fn new(
        class: FailureClass,
        kind: impl Into<String>,
        code: impl Into<String>,
        message: impl Into<String>,
        details: Option<serde_json::Value>,
    ) -> Self {
        Self {
            class: Some(class),
            kind: kind.into(),
            code: code.into(),
            message: message.into(),
            details,
        }
    }

    pub fn operator_class(&self) -> FailureClass {
        self.class.unwrap_or_else(|| infer_failure_class(&self.kind, &self.code))
    }
}

fn infer_failure_class(kind: &str, code: &str) -> FailureClass {
    match code {
        "POLICY_DENIED" | "POLICY_UNENFORCEABLE" => FailureClass::Policy,
        "RUN_TIMEOUT" | "EXEC_TIMEOUT" => FailureClass::Timeout,
        "INPUT_MISSING"
        | "OUTPUT_MISSING"
        | "OUTPUT_PATH_INVALID"
        | "OUTPUT_SCHEMA_INVALID"
        | "OUTPUT_UNDECLARED"
        | "BRANCH_OUTPUT_MISSING" => FailureClass::User,
        "CONTAINER_ENGINE_UNAVAILABLE" | "ARTIFACT_ERROR" | "IO_ERROR" => {
            FailureClass::Infrastructure
        }
        _ => match kind {
            "Policy" => FailureClass::Policy,
            "Infrastructure" => FailureClass::Infrastructure,
            "Timeout" => FailureClass::Timeout,
            "User" | "Dependency" => FailureClass::User,
            _ => FailureClass::Execution,
        },
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CacheIdentity {
    pub cache_key: String,
    pub node_definition_fingerprint: String,
    pub declared_environment_fingerprint: String,
    pub input_lineage_fingerprint: String,
    pub params_fingerprint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command_fingerprint: Option<String>,
    pub policy_fingerprint: String,
    pub execution_contract_fingerprint: String,
    pub backend_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkipReason {
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureAffectedGroups {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub failed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cancelled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailureCauseRecord {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finished_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailurePropagationRecord {
    pub node_id: String,
    pub status: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub propagation_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub blocking_nodes: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunFailureSummary {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub roots: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_failure: Option<FailureCauseRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub propagated_failures: Vec<FailurePropagationRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub propagated_skips: Vec<FailurePropagationRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downstream_affected_nodes: Vec<String>,
    #[serde(default)]
    pub downstream_affected_groups: FailureAffectedGroups,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeCounts {
    pub success: u32,
    pub failed: u32,
    pub skipped: u32,
    pub cached: u32,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub cancelled: u32,
}

fn is_zero(value: &u32) -> bool {
    *value == 0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContainerImageReferencePolicy {
    RequireDigest,
    AllowUnpinned,
}

pub fn default_container_image_reference_policy() -> ContainerImageReferencePolicy {
    ContainerImageReferencePolicy::RequireDigest
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyInfo {
    pub deny_network: bool,
    pub deny_env: bool,
    pub deny_clock: bool,
    pub clean_env: bool,
    #[serde(default = "default_container_image_reference_policy")]
    pub container_image_reference_policy: ContainerImageReferencePolicy,
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
    #[serde(default)]
    pub gpu_devices: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OutputSummary {
    pub node_id: String,
    pub node_fingerprint: String,
    pub name: String,
    pub path: String,
    pub kind: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeclaredOutputArtifact {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub media_type: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceOutputArtifact {
    pub name: String,
    pub path: String,
    pub kind: String,
    pub required: bool,
    pub present: bool,
    pub media_type: String,
    pub size_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactIdentity {
    pub canonical_artifact_id: String,
    pub legacy_artifact_id: String,
    pub run_id: String,
    pub node_id: String,
    pub output_name: String,
    pub output_path: String,
    pub node_fingerprint: String,
    pub artifact_sha256: String,
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
    pub name: String,
    pub kind: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub path: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
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
    pub name: String,
    pub path: String,
    pub kind: String,
    pub media_type: String,
    pub size_bytes: u64,
    pub sha256: String,
    pub node_id: String,
    pub node_fingerprint: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub promotable: bool,
}

fn is_false(value: &bool) -> bool {
    !value
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<InputCollection>,
    pub files: Vec<InputFile>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputFile {
    #[serde(alias = "path")]
    pub local_path: String,
    #[serde(alias = "sha256")]
    pub source_sha256: String,
    #[serde(alias = "from_node")]
    pub source_node_id: String,
    #[serde(alias = "from_node_fingerprint")]
    pub source_node_fingerprint: String,
    #[serde(alias = "from_output")]
    pub source_output_name: String,
    #[serde(default = "default_input_materialization_mode")]
    pub materialization_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputCollection {
    pub name: String,
    pub semantic_kind: String,
    pub manifest_path: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub empty_policy: Option<String>,
    #[serde(default)]
    pub items: Vec<InputCollectionItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InputCollectionItem {
    pub input_port: String,
    pub source_node_id: String,
    pub source_output_name: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub local_path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_sha256: Option<String>,
}

fn default_manifest_version() -> String {
    "run-manifest/v0.1".to_string()
}

fn default_run_stop_request_version() -> String {
    "run-stop-request/v0.1".to_string()
}

fn default_planner_contract_version() -> String {
    "bijux-dag-planner/v1".to_string()
}

fn default_input_materialization_mode() -> String {
    "unknown".to_string()
}
