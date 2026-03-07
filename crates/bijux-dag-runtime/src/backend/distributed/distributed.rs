use crate::remote_executor::{
    RemoteExecutionReceipt, RemoteExecutionRequest, RemoteExecutorSubmitter,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributedExecutionRequest {
    pub run_id: String,
    pub node_id: String,
    pub worker_pool: String,
    pub backend_hint: String,
    pub command: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub attempt: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributedExecutionResult {
    pub run_id: String,
    pub node_id: String,
    pub status: String,
    pub outputs: Vec<String>,
    pub logs: Vec<String>,
    pub diagnostics: Vec<String>,
    pub started_unix_ms: u128,
    pub finished_unix_ms: u128,
    pub provenance: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerIdentity {
    pub worker_id: String,
    pub worker_version: String,
    pub backend_kind: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerCapabilities {
    pub cpu_capacity: u32,
    pub memory_mb: u32,
    pub supports_gpu: bool,
    pub supports_container: bool,
    pub supports_sandbox_profiles: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerRegistration {
    pub identity: WorkerIdentity,
    pub capabilities: WorkerCapabilities,
    pub registered_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkLease {
    pub lease_id: String,
    pub run_id: String,
    pub node_id: String,
    pub worker_id: String,
    pub expires_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerHeartbeat {
    pub worker_id: String,
    pub unix_ms: u128,
    pub inflight_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LivenessPolicy {
    pub heartbeat_timeout_ms: u64,
    pub grace_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReassignmentRule {
    pub trigger: String,
    pub max_reassignments: u32,
    pub preserve_attempt_lineage: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DeliveryGuarantee {
    ExactlyOnce,
    AtLeastOnce,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerSandboxNegotiation {
    pub worker_id: String,
    pub required_profile: String,
    pub accepted: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteLogStreamContract {
    pub run_id: String,
    pub node_id: String,
    pub stream_endpoint: String,
    pub local_fallback_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteArtifactUploadContract {
    pub run_id: String,
    pub node_id: String,
    pub artifact_path: String,
    pub target_store: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RemoteCancellationContract {
    pub run_id: String,
    pub node_id: Option<String>,
    pub propagate_to_worker: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetryLineageRecord {
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub parent_attempt: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlacementHint {
    pub node_id: String,
    pub preferred_pool: String,
    pub preferred_worker_labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerPool {
    pub pool_id: String,
    pub class: String,
    pub workers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DistributedFailureClass {
    Infrastructure,
    TaskLogic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerVersionCompatibilityRule {
    pub planner_version: String,
    pub minimum_worker_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributedSecurityModel {
    pub worker_trust_model: String,
    pub artifact_trust_model: String,
    pub command_trust_model: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributedReadinessChecklist {
    pub typed_transport_contracts: bool,
    pub worker_liveness_contracts: bool,
    pub retry_lineage_contracts: bool,
    pub security_model_documented: bool,
    pub conformance_fixtures_present: bool,
}

pub fn worker_alive(
    heartbeat: &WorkerHeartbeat,
    now_unix_ms: u128,
    policy: &LivenessPolicy,
) -> bool {
    now_unix_ms.saturating_sub(heartbeat.unix_ms) <= policy.heartbeat_timeout_ms as u128
}

pub fn should_reassign(lease: &WorkLease, now_unix_ms: u128) -> bool {
    now_unix_ms > lease.expires_unix_ms
}

pub fn check_worker_version_compatibility(
    worker_version: &str,
    rule: &WorkerVersionCompatibilityRule,
) -> bool {
    worker_version >= rule.minimum_worker_version.as_str() && !rule.planner_version.is_empty()
}

#[derive(Default, Clone)]
pub struct MockRemoteBackend {
    submissions: Arc<Mutex<Vec<DistributedExecutionRequest>>>,
}

impl MockRemoteBackend {
    pub fn submissions(&self) -> Vec<DistributedExecutionRequest> {
        self.submissions
            .lock()
            .map(|g| g.clone())
            .unwrap_or_default()
    }

    pub fn submit_distributed(
        &self,
        request: DistributedExecutionRequest,
    ) -> Result<DistributedExecutionResult, String> {
        self.submissions
            .lock()
            .map_err(|_| "submission lock poisoned".to_string())?
            .push(request.clone());
        Ok(DistributedExecutionResult {
            run_id: request.run_id,
            node_id: request.node_id,
            status: "accepted".to_string(),
            outputs: Vec::new(),
            logs: Vec::new(),
            diagnostics: Vec::new(),
            started_unix_ms: 0,
            finished_unix_ms: 0,
            provenance: BTreeMap::new(),
        })
    }
}

impl RemoteExecutorSubmitter for MockRemoteBackend {
    fn submit(&self, request: RemoteExecutionRequest) -> Result<RemoteExecutionReceipt, String> {
        let distributed = DistributedExecutionRequest {
            run_id: request.run_id,
            node_id: request.node_id,
            worker_pool: "default".to_string(),
            backend_hint: "mock".to_string(),
            command: vec!["mock".to_string()],
            env: BTreeMap::new(),
            attempt: 1,
        };
        let _ = self.submit_distributed(distributed)?;
        Ok(RemoteExecutionReceipt {
            submission_id: format!("mock-{}", request.contract_digest),
            accepted: true,
            reason: None,
        })
    }
}
