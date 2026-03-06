use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutorBackend {
    Local,
    Subprocess,
    Container,
    Kubernetes,
    Hpc,
    ExternalService,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BackendCapabilities {
    pub supports_container: bool,
    pub supports_network_isolation: bool,
    pub supports_env_allowlist: bool,
    pub supports_artifact_mounts: bool,
    pub supports_remote_logs: bool,
    pub supports_gpu: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendCapabilityRequirement {
    pub container_required: bool,
    pub network_isolation_required: bool,
    pub env_allowlist_required: bool,
    pub artifact_mount_required: bool,
    pub remote_logs_required: bool,
    pub gpu_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDecision {
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContainerExecutionContract {
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub work_dir: Option<String>,
    pub env_allowlist: Vec<String>,
    pub artifact_mount_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct KubernetesExecutorContract {
    pub namespace: String,
    pub pod_template: String,
    pub image_resolution_policy: String,
    pub artifact_mount_strategy: String,
    pub log_collection: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HpcExecutorContract {
    pub scheduler: String,
    pub queue: String,
    pub account: Option<String>,
    pub submit_command: String,
    pub poll_command: String,
    pub cancel_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeSecretContract {
    pub secret_refs: Vec<String>,
    pub injection_mode: String,
    pub redaction_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactStoreBackend {
    Filesystem,
    ObjectStorage,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObjectStorageContract {
    pub provider: String,
    pub bucket: String,
    pub prefix: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RegistryPersistenceBackend {
    Filesystem,
    Database,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiTenantIdentity {
    pub tenant_id: String,
    pub namespace: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueuePartition {
    pub queue_name: String,
    pub tenant_id: Option<String>,
    pub max_concurrency: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerScalingPlan {
    pub worker_count: u32,
    pub sharding_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HighAvailabilitySchedulerPlan {
    pub enabled: bool,
    pub leader_election: String,
    pub durable_queue: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendExecutionRequest {
    pub backend: ExecutorBackend,
    pub run_id: String,
    pub node_id: String,
    pub command: Vec<String>,
    pub environment: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendExecutionCompletion {
    pub backend: ExecutorBackend,
    pub run_id: String,
    pub node_id: String,
    pub status: String,
    pub exit_code: Option<i32>,
    pub diagnostics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactTransportMode {
    LocalCopy,
    Hardlink,
    RemoteUpload,
    RemoteDownload,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactTransportContract {
    pub mode: ArtifactTransportMode,
    pub source: String,
    pub destination: String,
    pub checksum_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendPolicyOverlay {
    pub backend: ExecutorBackend,
    pub policy_id: String,
    pub settings: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackendAcceptanceGate {
    pub backend: ExecutorBackend,
    pub deterministic_replay: bool,
    pub artifact_integrity: bool,
    pub policy_enforcement: bool,
    pub observability_coverage: bool,
    pub reliability_target: String,
}

pub fn negotiate_backend_capabilities(
    capabilities: &BackendCapabilities,
    requirements: &BackendCapabilityRequirement,
) -> CapabilityDecision {
    if requirements.container_required && !capabilities.supports_container {
        return CapabilityDecision {
            accepted: false,
            reason: "container execution is required but unsupported".to_string(),
        };
    }
    if requirements.network_isolation_required && !capabilities.supports_network_isolation {
        return CapabilityDecision {
            accepted: false,
            reason: "network isolation is required but unsupported".to_string(),
        };
    }
    if requirements.env_allowlist_required && !capabilities.supports_env_allowlist {
        return CapabilityDecision {
            accepted: false,
            reason: "env allowlist is required but unsupported".to_string(),
        };
    }
    if requirements.artifact_mount_required && !capabilities.supports_artifact_mounts {
        return CapabilityDecision {
            accepted: false,
            reason: "artifact mounts are required but unsupported".to_string(),
        };
    }
    if requirements.remote_logs_required && !capabilities.supports_remote_logs {
        return CapabilityDecision {
            accepted: false,
            reason: "remote logs are required but unsupported".to_string(),
        };
    }
    if requirements.gpu_required && !capabilities.supports_gpu {
        return CapabilityDecision {
            accepted: false,
            reason: "gpu support is required but unsupported".to_string(),
        };
    }
    CapabilityDecision {
        accepted: true,
        reason: "backend satisfies all declared requirements".to_string(),
    }
}
