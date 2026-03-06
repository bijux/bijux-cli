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
    rules.iter()
        .find(|rule| rule.backend_error_code == backend_error_code)
        .cloned()
}

pub fn backend_ready_for_admission(probe: &BackendReadinessProbe, mode: &BackendMaintenanceMode) -> bool {
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
    rules.iter().any(|r| {
        r.from_backend == from_backend && r.to_backend == to_backend && r.replay_safe
    })
}
