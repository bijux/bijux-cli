use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
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
pub struct SlurmAdapterDesignContractReport {
    pub contract: SlurmExecutorContract,
    pub submit_status_cancel_documented: bool,
    pub log_collection_mode: String,
    pub artifact_collection_mode: String,
    pub failure_mapping_examples: BTreeMap<String, String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesAdapterContractReport {
    pub contract: KubernetesExecutorContractV2,
    pub job_spec_mapping: String,
    pub pod_status_mapping: String,
    pub log_collection_mode: String,
    pub artifact_collection_mode: String,
    pub timeout_cancel_behavior: String,
    pub unsupported_field_rejection: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackendCapabilityDescriptor {
    pub cpu_class: String,
    pub memory_class: String,
    pub gpu_class: Option<String>,
    pub ephemeral_storage_class: String,
    pub network_class: String,
}

pub fn slurm_adapter_design_contract() -> SlurmAdapterDesignContractReport {
    SlurmAdapterDesignContractReport {
        contract: SlurmExecutorContract {
            partition: "cpu-standard".to_string(),
            submit_command: "sbatch".to_string(),
            poll_command: "sacct".to_string(),
            cancel_command: "scancel".to_string(),
            result_mapping: "slurm-exit-code-and-state".to_string(),
        },
        submit_status_cancel_documented: true,
        log_collection_mode: hpc_log_collection_semantics(2).mode,
        artifact_collection_mode: "stage outputs from scratch to run-dir artifact store".to_string(),
        failure_mapping_examples: BTreeMap::from([
            (
                "SLURM_WALLTIME_EXCEEDED".to_string(),
                classify_hpc_failure("SLURM_WALLTIME_EXCEEDED").runtime_failure_kind,
            ),
            (
                "SLURM_PREEMPTED".to_string(),
                classify_hpc_failure("SLURM_PREEMPTED").runtime_failure_kind,
            ),
            (
                "SLURM_INVALID_ACCOUNT".to_string(),
                classify_hpc_failure("SLURM_INVALID_ACCOUNT").runtime_failure_kind,
            ),
        ]),
    }
}

pub fn kubernetes_adapter_contract() -> KubernetesAdapterContractReport {
    KubernetesAdapterContractReport {
        contract: KubernetesExecutorContractV2 {
            namespace: "bijux-dag".to_string(),
            pod_spec_source: "generated-job-spec".to_string(),
            image_resolution_policy: "digest-pinned".to_string(),
            artifact_flow: "mount-workdir-and-collect-outputs".to_string(),
            log_flow: "stdout-stderr-pod-log-stream".to_string(),
            cancellation_behavior: "delete-job-with-grace-period".to_string(),
        },
        job_spec_mapping: "node resources and retry policy map into Job requests, limits, deadline, and backoff".to_string(),
        pod_status_mapping: "terminal pod phases reconcile into runtime success/failure with retry classification".to_string(),
        log_collection_mode: "stdout/stderr streamed from pod logs and copied into node evidence".to_string(),
        artifact_collection_mode: "declared output files collected from mounted workdir after terminal pod state".to_string(),
        timeout_cancel_behavior: "active deadline seconds and termination grace map from node timeout and cancel policy".to_string(),
        unsupported_field_rejection: vec![
            "hostNetwork".to_string(),
            "hostPID".to_string(),
            "privileged".to_string(),
            "hostPath".to_string(),
            "runtimeClassName".to_string(),
        ],
    }
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct K8sCapabilityDeclaration {
    pub supports_node_selector: bool,
    pub supports_node_affinity: bool,
    pub supports_pod_affinity: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcNodeExecutionContract {
    pub cpu_units: u32,
    pub memory_mib: u32,
    pub timeout_seconds: u32,
    pub requested_partition: Option<String>,
    pub requested_queue: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcQueuePartitionMapping {
    pub queue: String,
    pub partition: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcRetryPolicyDecision {
    pub scheduler_retry_enabled: bool,
    pub bijux_retry_enabled: bool,
    pub effective_retry_owner: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcScratchStagingSemantics {
    pub scratch_dir: String,
    pub staging_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcFailureClassification {
    pub runtime_failure_kind: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcLogCollectionSemantics {
    pub mode: String,
    pub chunks_collected: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcSchedulerVersionMetadata {
    pub scheduler_name: String,
    pub scheduler_version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HpcResourceFingerprintInput {
    pub queue: String,
    pub partition: String,
    pub account: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HpcReplayFidelity {
    Exact,
    Downgraded,
}

const TERMINAL_PHASES: [&str; 3] = ["Succeeded", "Failed", "Cancelled"];

pub fn matches_placement_policy(
    required_capability: &str,
    backend_descriptor: &BackendCapabilityDescriptor,
) -> bool {
    required_capability == backend_descriptor.cpu_class
        || required_capability == backend_descriptor.memory_class
        || backend_descriptor.gpu_class.as_ref().map(|g| g == required_capability).unwrap_or(false)
        || required_capability == backend_descriptor.ephemeral_storage_class
        || required_capability == backend_descriptor.network_class
}

pub fn normalize_backend_failure(
    backend_error_code: &str,
    rules: &[BackendFailureMappingRule],
) -> Option<BackendFailureMappingRule> {
    rules.iter().find(|rule| rule.backend_error_code == backend_error_code).cloned()
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
        requests: K8sResourceRequest { cpu_millis: request_cpu, memory_mib: request_mem },
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
        WorkdirVolumeKind::EmptyDir => {
            WorkdirSemantics { survives_pod_restart: false, survives_reschedule: false }
        }
        WorkdirVolumeKind::PersistentVolumeClaim => {
            WorkdirSemantics { survives_pod_restart: true, survives_reschedule: true }
        }
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

pub fn reconcile_k8s_watch_stream(
    previous_terminal: &BTreeMap<String, K8sWatchEvent>,
    events: &[K8sWatchEvent],
) -> BTreeMap<String, K8sWatchEvent> {
    let mut merged = previous_terminal.clone();
    for (node_id, event) in canonical_k8s_terminal_events(events) {
        match merged.get(&node_id) {
            Some(existing) if existing.sequence > event.sequence => {}
            _ => {
                merged.insert(node_id, event);
            }
        }
    }
    merged
}

pub fn k8s_capability_declaration() -> K8sCapabilityDeclaration {
    K8sCapabilityDeclaration {
        supports_node_selector: true,
        supports_node_affinity: true,
        supports_pod_affinity: true,
    }
}

pub fn reject_unsupported_k8s_fields(fields: &[String]) -> Result<(), String> {
    let blocked = ["hostNetwork", "hostPID", "privileged", "hostPath", "runtimeClassName"];
    for field in fields {
        if blocked.iter().any(|blocked_name| field == blocked_name) {
            return Err(format!(
                "unsupported kubernetes-only field is rejected at dag layer: {field}"
            ));
        }
    }
    Ok(())
}

pub fn map_node_to_hpc_queue_partition(
    node: &HpcNodeExecutionContract,
    default_queue: &str,
    default_partition: &str,
) -> HpcQueuePartitionMapping {
    HpcQueuePartitionMapping {
        queue: node.requested_queue.clone().unwrap_or_else(|| default_queue.to_string()),
        partition: node
            .requested_partition
            .clone()
            .unwrap_or_else(|| default_partition.to_string()),
    }
}

pub fn map_timeout_to_hpc_walltime(timeout_seconds: u32) -> String {
    let total = timeout_seconds.max(1);
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn effective_hpc_retry_policy(
    scheduler_native_retry: bool,
    bijux_retry: bool,
) -> HpcRetryPolicyDecision {
    let effective_retry_owner = if scheduler_native_retry {
        "scheduler-native"
    } else if bijux_retry {
        "bijux"
    } else {
        "none"
    };
    HpcRetryPolicyDecision {
        scheduler_retry_enabled: scheduler_native_retry,
        bijux_retry_enabled: bijux_retry,
        effective_retry_owner: effective_retry_owner.to_string(),
    }
}

pub fn hpc_scratch_staging_semantics(run_id: &str, node_id: &str) -> HpcScratchStagingSemantics {
    HpcScratchStagingSemantics {
        scratch_dir: format!("/scratch/{run_id}/{node_id}"),
        staging_dir: format!("/staging/{run_id}/{node_id}"),
    }
}

pub fn classify_hpc_failure(code: &str) -> HpcFailureClassification {
    match code {
        "SLURM_QUEUE_REJECTED" | "SLURM_INVALID_ACCOUNT" => HpcFailureClassification {
            runtime_failure_kind: "configuration".to_string(),
            retryable: false,
        },
        "SLURM_WALLTIME_EXCEEDED" => HpcFailureClassification {
            runtime_failure_kind: "timeout".to_string(),
            retryable: true,
        },
        "SLURM_PREEMPTED" => HpcFailureClassification {
            runtime_failure_kind: "infrastructure".to_string(),
            retryable: true,
        },
        _ => HpcFailureClassification {
            runtime_failure_kind: "execution".to_string(),
            retryable: false,
        },
    }
}

pub fn hpc_poll_response_recovered(last_poll_age_seconds: u32, timeout_seconds: u32) -> bool {
    last_poll_age_seconds <= timeout_seconds.max(1)
}

pub fn hpc_log_collection_semantics(chunks_collected: u32) -> HpcLogCollectionSemantics {
    let mode = if chunks_collected > 0 { "streaming-chunked" } else { "no-logs" };
    HpcLogCollectionSemantics { mode: mode.to_string(), chunks_collected }
}

pub fn staged_input_cleanup_required(run_succeeded: bool) -> bool {
    run_succeeded
}

pub fn scratch_retention_required(run_succeeded: bool, retain_on_failure: bool) -> bool {
    if run_succeeded {
        false
    } else {
        retain_on_failure
    }
}

pub fn hpc_array_job_supported(scheduler: &str) -> bool {
    matches!(scheduler, "slurm")
}

pub fn reject_unsupported_hpc_scheduler_features(features: &[String]) -> Result<(), String> {
    let blocked = ["interactive-shell", "privileged-container", "host-network"];
    for feature in features {
        if blocked.iter().any(|item| feature == item) {
            return Err(format!("unsupported scheduler feature: {feature}"));
        }
    }
    Ok(())
}

pub fn hpc_environment_fingerprint(modules: &[String], env: &BTreeMap<String, String>) -> String {
    let mut hasher = Sha256::new();
    let mut sorted_modules = modules.to_vec();
    sorted_modules.sort();
    for module in sorted_modules {
        hasher.update(module.as_bytes());
        hasher.update(b"\n");
    }
    for (key, value) in env {
        hasher.update(key.as_bytes());
        hasher.update(b"=");
        hasher.update(value.as_bytes());
        hasher.update(b"\n");
    }
    hex::encode(hasher.finalize())
}

pub fn capture_hpc_scheduler_version(
    scheduler_name: &str,
    scheduler_version: &str,
) -> HpcSchedulerVersionMetadata {
    HpcSchedulerVersionMetadata {
        scheduler_name: scheduler_name.to_string(),
        scheduler_version: scheduler_version.to_string(),
    }
}

pub fn hpc_resource_fingerprint(input: &HpcResourceFingerprintInput) -> String {
    let mut hasher = Sha256::new();
    hasher.update(input.queue.as_bytes());
    hasher.update(b"\n");
    hasher.update(input.partition.as_bytes());
    hasher.update(b"\n");
    hasher.update(input.account.as_bytes());
    hex::encode(hasher.finalize())
}

pub fn hpc_replay_fidelity_from_module_fingerprints(
    source_fingerprint: &str,
    target_fingerprint: &str,
) -> HpcReplayFidelity {
    if source_fingerprint == target_fingerprint {
        HpcReplayFidelity::Exact
    } else {
        HpcReplayFidelity::Downgraded
    }
}
