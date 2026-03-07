use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesExecutorContractV2 {
    pub namespace: String,
    pub pod_spec_source: String,
    pub image_resolution_policy: String,
    pub artifact_flow: String,
    pub log_flow: String,
    pub cancellation_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlurmExecutorContract {
    pub partition: String,
    pub submit_command: String,
    pub poll_command: String,
    pub cancel_command: String,
    pub result_mapping: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GenericBatchExecutorContract {
    pub platform_name: String,
    pub submit_api: String,
    pub poll_api: String,
    pub cancel_api: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCapabilityDescriptor {
    pub cpu_class: String,
    pub memory_class: String,
    pub gpu_class: Option<String>,
    pub ephemeral_storage_class: String,
    pub network_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlacementPolicyRule {
    pub rule_id: String,
    pub required_capability: String,
    pub backend_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendFailureMappingRule {
    pub backend_error_code: String,
    pub runtime_failure_kind: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageResolutionProvenance {
    pub image_ref: String,
    pub resolved_digest: String,
    pub resolver_identity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendLogCollectionContract {
    pub stream_mode: String,
    pub partial_recovery_supported: bool,
    pub retention_hint_days: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteArtifactStagingProtocol {
    pub upload_endpoint: String,
    pub download_endpoint: String,
    pub integrity_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCleanupGuarantee {
    pub cleanup_on_cancel: bool,
    pub cleanup_on_failure: bool,
    pub max_cleanup_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueBackendRoutingPolicy {
    pub queue: String,
    pub backend_class: String,
    pub cost_tier: String,
    pub trust_level: String,
    pub latency_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeAffinityHint {
    pub required_labels: BTreeMap<String, String>,
    pub anti_affinity_labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendReadinessProbe {
    pub backend_class: String,
    pub healthy: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendMaintenanceMode {
    Active,
    Draining,
    Maintenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendQuotaMetrics {
    pub backend_class: String,
    pub quota_limit: u64,
    pub quota_used: u64,
    pub saturation_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendConformanceSuite {
    pub backend_class: String,
    pub required_checks: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossBackendReplayRule {
    pub from_backend: String,
    pub to_backend: String,
    pub replay_safe: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendOutageSimulationFixture {
    pub fixture_id: String,
    pub degraded_backends: Vec<String>,
    pub expected_routing_shift: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendProductionReadinessChecklist {
    pub backend_class: String,
    pub deterministic_replay: bool,
    pub conformance_passed: bool,
    pub cleanup_guarantees_verified: bool,
    pub observability_integrated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sResourceRequest {
    pub cpu_millis: u32,
    pub memory_mib: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sResourceMapping {
    pub requests: K8sResourceRequest,
    pub limits: K8sResourceRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeExecutionContract {
    pub cpu_units: u32,
    pub memory_mib: u32,
    pub timeout_seconds: u32,
    pub max_retries: u32,
    pub retry_backoff_seconds: u32,
    pub cancel_grace_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sJobPolicyMapping {
    pub active_deadline_seconds: u32,
    pub backoff_limit: u32,
    pub retry_backoff_seconds: u32,
    pub termination_grace_period_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum K8sFailureClass {
    InfrastructureRetryable,
    InfrastructureFatal,
    Configuration,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeFailureClassification {
    pub runtime_failure_kind: String,
    pub retryable: bool,
    pub class: K8sFailureClass,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sInjectionRequest {
    pub required_secrets: Vec<String>,
    pub required_configs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sInjectionAvailability {
    pub available_secrets: Vec<String>,
    pub available_configs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactCollectionState {
    Complete,
    Partial,
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkdirVolumeKind {
    EmptyDir,
    PersistentVolumeClaim,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkdirSemantics {
    pub survives_pod_restart: bool,
    pub survives_reschedule: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterExecutionOutcome {
    pub dag_shape: String,
    pub node_statuses: BTreeMap<String, String>,
    pub output_hashes: BTreeMap<String, String>,
    pub stdout: String,
    pub stderr: String,
    pub cache_hit_nodes: Vec<String>,
    pub replayed_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sWatchEvent {
    pub node_id: String,
    pub phase: String,
    pub observed_at_millis: u64,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sBackendVersionMetadata {
    pub k8s_version: String,
    pub api_server: String,
    pub cluster_uid: String,
}

const TERMINAL_PHASES: [&str; 3] = ["Succeeded", "Failed", "Cancelled"];

pub fn matches_placement_policy(
    required_capability: &str,
    backend_descriptor: &BackendCapabilityDescriptor,
) -> bool {
    required_capability == backend_descriptor.cpu_class
        || required_capability == backend_descriptor.memory_class
        || backend_descriptor
            .gpu_class
            .as_ref()
            .map(|g| g == required_capability)
            .unwrap_or(false)
        || required_capability == backend_descriptor.ephemeral_storage_class
        || required_capability == backend_descriptor.network_class
}

pub fn normalize_backend_failure(
    backend_error_code: &str,
    rules: &[BackendFailureMappingRule],
) -> Option<BackendFailureMappingRule> {
    rules
        .iter()
        .find(|rule| rule.backend_error_code == backend_error_code)
        .cloned()
}

pub fn backend_ready_for_admission(
    probe: &BackendReadinessProbe,
    mode: &BackendMaintenanceMode,
) -> bool {
    probe.healthy && matches!(mode, BackendMaintenanceMode::Active)
}

pub fn quota_saturation_percent(limit: u64, used: u64) -> u8 {
    if limit == 0 {
        return 100;
    }
    ((used.saturating_mul(100) / limit).min(100)) as u8
}

pub fn replay_allowed_across_backends(
    from_backend: &str,
    to_backend: &str,
    rules: &[CrossBackendReplayRule],
) -> bool {
    rules
        .iter()
        .any(|r| r.from_backend == from_backend && r.to_backend == to_backend && r.replay_safe)
}

pub fn map_node_resources_to_k8s(node: &NodeExecutionContract) -> K8sResourceMapping {
    let request_cpu = node.cpu_units.saturating_mul(1000);
    let request_mem = node.memory_mib;
    // Keep limits deterministic and explicit: 2x cpu, 1.5x memory floor-rounded.
    let limit_cpu = request_cpu.saturating_mul(2);
    let limit_mem = ((request_mem as u64 * 3) / 2) as u32;
    K8sResourceMapping {
        requests: K8sResourceRequest {
            cpu_millis: request_cpu,
            memory_mib: request_mem,
        },
        limits: K8sResourceRequest {
            cpu_millis: limit_cpu,
            memory_mib: limit_mem.max(request_mem),
        },
    }
}

pub fn map_node_policy_to_k8s_job(node: &NodeExecutionContract) -> K8sJobPolicyMapping {
    K8sJobPolicyMapping {
        active_deadline_seconds: node.timeout_seconds.max(1),
        backoff_limit: node.max_retries,
        retry_backoff_seconds: node.retry_backoff_seconds,
        termination_grace_period_seconds: node.cancel_grace_seconds.max(1),
    }
}

pub fn classify_k8s_failure(code: &str) -> RuntimeFailureClassification {
    match code {
        "K8S_POD_EVICTED" => RuntimeFailureClassification {
            runtime_failure_kind: "infrastructure".to_string(),
            retryable: true,
            class: K8sFailureClass::InfrastructureRetryable,
        },
        "K8S_IMAGE_PULL_BACKOFF" => RuntimeFailureClassification {
            runtime_failure_kind: "configuration".to_string(),
            retryable: false,
            class: K8sFailureClass::Configuration,
        },
        "K8S_POD_PENDING_TIMEOUT" => RuntimeFailureClassification {
            runtime_failure_kind: "infrastructure".to_string(),
            retryable: true,
            class: K8sFailureClass::InfrastructureRetryable,
        },
        _ => RuntimeFailureClassification {
            runtime_failure_kind: "execution".to_string(),
            retryable: false,
            class: K8sFailureClass::InfrastructureFatal,
        },
    }
}

pub fn validate_k8s_injection(
    requested: &K8sInjectionRequest,
    available: &K8sInjectionAvailability,
) -> Result<(), String> {
    for secret in &requested.required_secrets {
        if !available.available_secrets.iter().any(|s| s == secret) {
            return Err(format!("missing required secret: {secret}"));
        }
    }
    for cfg in &requested.required_configs {
        if !available.available_configs.iter().any(|c| c == cfg) {
            return Err(format!("missing required config: {cfg}"));
        }
    }
    Ok(())
}

pub fn outputs_logs_equivalent(
    local: &AdapterExecutionOutcome,
    k8s: &AdapterExecutionOutcome,
) -> bool {
    local.output_hashes == k8s.output_hashes
        && local.stdout == k8s.stdout
        && local.stderr == k8s.stderr
}

pub fn equivalent_to_local(local: &AdapterExecutionOutcome, k8s: &AdapterExecutionOutcome) -> bool {
    local.dag_shape == k8s.dag_shape
        && local.node_statuses == k8s.node_statuses
        && local.output_hashes == k8s.output_hashes
        && local.cache_hit_nodes == k8s.cache_hit_nodes
        && local.replayed_nodes == k8s.replayed_nodes
}

pub fn artifact_collection_state(expected: usize, collected: usize) -> ArtifactCollectionState {
    if collected == 0 {
        ArtifactCollectionState::Missing
    } else if collected >= expected {
        ArtifactCollectionState::Complete
    } else {
        ArtifactCollectionState::Partial
    }
}

pub fn workdir_semantics(kind: WorkdirVolumeKind) -> WorkdirSemantics {
    match kind {
        WorkdirVolumeKind::EmptyDir => WorkdirSemantics {
            survives_pod_restart: false,
            survives_reschedule: false,
        },
        WorkdirVolumeKind::PersistentVolumeClaim => WorkdirSemantics {
            survives_pod_restart: true,
            survives_reschedule: true,
        },
    }
}

pub fn canonical_k8s_terminal_events(events: &[K8sWatchEvent]) -> BTreeMap<String, K8sWatchEvent> {
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|e| (e.sequence, e.observed_at_millis));
    let mut out = BTreeMap::new();
    for event in sorted {
        if !TERMINAL_PHASES.iter().any(|phase| *phase == event.phase) {
            continue;
        }
        out.insert(event.node_id.clone(), event);
    }
    out
}
