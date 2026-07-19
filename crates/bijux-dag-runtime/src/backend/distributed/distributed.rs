use crate::remote_execution_model::{
    MockRemoteWorker, RemoteNodeExecutionPayload, RemoteNodeExecutionResult, RemoteWorkerExecutor,
};
use crate::remote_executor::{
    RemoteExecutionReceipt, RemoteExecutionRequest, RemoteExecutorSubmitter,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
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
pub struct TaskLeaseSemantics {
    pub lease_duration_ms: u64,
    pub renew_before_expiry_ms: u64,
    pub max_renewals: u32,
    pub recovery_grace_ms: u64,
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
pub struct HeartbeatSemantics {
    pub interval_ms: u64,
    pub timeout_ms: u64,
    pub delayed_threshold_ms: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum HeartbeatClass {
    Healthy,
    Delayed,
    Lost,
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
pub struct RemoteArtifactCommitContract {
    pub run_id: String,
    pub node_id: String,
    pub attempt: u32,
    pub upload_id: String,
    pub committed: bool,
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
pub struct RemoteStatusEvent {
    pub run_id: String,
    pub node_id: String,
    pub sequence: u64,
    pub status: String,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkerPoolCapabilityRequest {
    pub required_min_cpu_capacity: u32,
    pub required_min_memory_mb: u32,
    pub require_gpu: bool,
    pub require_container_support: bool,
    pub required_sandbox_profile: Option<String>,
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
pub enum StatusReportingClass {
    Healthy,
    Partitioned,
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

pub fn validate_task_lease_semantics(semantics: &TaskLeaseSemantics) -> Result<(), String> {
    if semantics.lease_duration_ms == 0 {
        return Err("lease duration must be greater than zero".to_string());
    }
    if semantics.renew_before_expiry_ms >= semantics.lease_duration_ms {
        return Err("renew_before_expiry_ms must be less than lease_duration_ms".to_string());
    }
    if semantics.max_renewals == 0 {
        return Err("max_renewals must be greater than zero".to_string());
    }
    Ok(())
}

pub fn validate_worker_identity(identity: &WorkerIdentity) -> Result<(), String> {
    for field in [
        identity.worker_id.as_str(),
        identity.worker_version.as_str(),
        identity.backend_kind.as_str(),
    ] {
        if field.trim().is_empty() {
            return Err("worker identity fields must be non-empty".to_string());
        }
    }
    Ok(())
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

pub fn recover_lost_lease(
    lease: &WorkLease,
    now_unix_ms: u128,
    semantics: &TaskLeaseSemantics,
) -> bool {
    now_unix_ms.saturating_sub(lease.expires_unix_ms) <= semantics.recovery_grace_ms as u128
}

pub fn classify_heartbeat(
    heartbeat: &WorkerHeartbeat,
    now_unix_ms: u128,
    semantics: &HeartbeatSemantics,
) -> HeartbeatClass {
    let age = now_unix_ms.saturating_sub(heartbeat.unix_ms);
    if age > semantics.timeout_ms as u128 {
        HeartbeatClass::Lost
    } else if age > semantics.delayed_threshold_ms as u128 {
        HeartbeatClass::Delayed
    } else {
        HeartbeatClass::Healthy
    }
}

pub fn is_duplicate_dispatch(
    dispatched_keys: &mut BTreeSet<String>,
    run_id: &str,
    node_id: &str,
) -> bool {
    let key = format!("{run_id}:{node_id}");
    !dispatched_keys.insert(key)
}

pub fn check_worker_version_compatibility(
    worker_version: &str,
    rule: &WorkerVersionCompatibilityRule,
) -> bool {
    worker_version >= rule.minimum_worker_version.as_str() && !rule.planner_version.is_empty()
}

pub fn reject_worker_version_mismatch(
    worker_version: &str,
    rule: &WorkerVersionCompatibilityRule,
) -> Result<(), String> {
    if check_worker_version_compatibility(worker_version, rule) {
        Ok(())
    } else {
        Err(format!(
            "worker version mismatch: worker={worker_version}, minimum_supported={}",
            rule.minimum_worker_version
        ))
    }
}

pub fn artifact_upload_can_commit(
    upload: &RemoteArtifactUploadContract,
    commit: &RemoteArtifactCommitContract,
) -> bool {
    upload.run_id == commit.run_id
        && upload.node_id == commit.node_id
        && !upload.checksum.trim().is_empty()
        && !commit.upload_id.trim().is_empty()
        && commit.committed
}

pub fn verify_remote_artifact_integrity(
    expected_checksum: &str,
    transported_checksum: &str,
) -> bool {
    !expected_checksum.trim().is_empty() && expected_checksum == transported_checksum
}

pub fn normalize_status_events(
    events: &[RemoteStatusEvent],
) -> (Vec<RemoteStatusEvent>, Vec<RemoteStatusEvent>) {
    let mut sorted = events.to_vec();
    sorted.sort_by(|a, b| a.sequence.cmp(&b.sequence).then(a.unix_ms.cmp(&b.unix_ms)));

    let mut deduped = Vec::new();
    let mut duplicates = Vec::new();
    let mut seen = BTreeSet::new();
    for event in sorted {
        let key =
            (event.run_id.clone(), event.node_id.clone(), event.sequence, event.status.clone());
        if seen.insert(key) {
            deduped.push(event);
        } else {
            duplicates.push(event);
        }
    }
    (deduped, duplicates)
}

pub fn classify_status_reporting(
    last_status_unix_ms: u128,
    now_unix_ms: u128,
    timeout_ms: u64,
) -> StatusReportingClass {
    if now_unix_ms.saturating_sub(last_status_unix_ms) > timeout_ms as u128 {
        StatusReportingClass::Partitioned
    } else {
        StatusReportingClass::Healthy
    }
}

pub fn cancellation_delivered_in_time(
    issued_unix_ms: u128,
    delivered_unix_ms: u128,
    deadline_ms: u64,
) -> bool {
    delivered_unix_ms.saturating_sub(issued_unix_ms) <= deadline_ms as u128
}

pub fn worker_pool_satisfies_capability_request(
    worker: &WorkerCapabilities,
    request: &WorkerPoolCapabilityRequest,
) -> bool {
    if worker.cpu_capacity < request.required_min_cpu_capacity
        || worker.memory_mb < request.required_min_memory_mb
    {
        return false;
    }
    if request.require_gpu && !worker.supports_gpu {
        return false;
    }
    if request.require_container_support && !worker.supports_container {
        return false;
    }
    if let Some(profile) = &request.required_sandbox_profile {
        return worker.supports_sandbox_profiles.iter().any(|p| p == profile);
    }
    true
}

#[derive(Default, Clone)]
pub struct MockRemoteBackend {
    submissions: Arc<Mutex<Vec<DistributedExecutionRequest>>>,
    payload_executions: Arc<Mutex<Vec<RemoteNodeExecutionPayload>>>,
}

impl MockRemoteBackend {
    pub fn submissions(&self) -> Vec<DistributedExecutionRequest> {
        self.submissions.lock().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn payload_executions(&self) -> Vec<RemoteNodeExecutionPayload> {
        self.payload_executions.lock().map(|g| g.clone()).unwrap_or_default()
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

    pub fn execute_remote_payload(
        &self,
        payload: RemoteNodeExecutionPayload,
    ) -> Result<RemoteNodeExecutionResult, String> {
        self.payload_executions
            .lock()
            .map_err(|_| "payload execution lock poisoned".to_string())?
            .push(payload.clone());
        MockRemoteWorker.execute_payload(payload)
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
