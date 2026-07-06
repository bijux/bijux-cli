use crate::backend_cluster::{
    map_node_policy_to_k8s_job, map_node_resources_to_k8s, K8sJobPolicyMapping, K8sResourceMapping,
    NodeExecutionContract,
};
use crate::remote_execution_model::{
    execute_modeled_payload, validate_remote_execution_payload, RemoteNodeExecutionPayload,
    RemoteNodeExecutionResult,
};
use crate::{ConstAdapter, ContainerAdapter, FailureClass, NodeResult, NodeStatus, ShellAdapter};
use bijux_dag_core::{Node, NodeKind};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};

const DEFAULT_CPU_UNITS: u32 = 1;
const DEFAULT_MEMORY_MIB: u32 = 256;
const DEFAULT_TIMEOUT_SECONDS: u32 = 60;
const DEFAULT_CANCEL_GRACE_SECONDS: u32 = 30;
const KUBERNETES_SCHEDULER_ID: &str = "kubernetes";
const KUBERNETES_STATUS_MAPPING_ID: &str = "kubernetes-pod-phase";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesExecutionRequest {
    pub payload: RemoteNodeExecutionPayload,
    pub namespace: String,
    pub resources: K8sResourceMapping,
    pub policy: K8sJobPolicyMapping,
    pub workspace: KubernetesWorkspaceTransfer,
    pub workload: KubernetesWorkloadDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesWorkspaceTransfer {
    pub mode: KubernetesWorkspaceTransferMode,
    pub mounts: Vec<KubernetesVolumeMount>,
    pub input_artifact_count: usize,
    pub declared_output_count: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KubernetesWorkspaceTransferMode {
    MountedWorkdir,
    StagedArtifacts,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesVolumeMount {
    pub name: String,
    pub mount_path: String,
    pub readonly: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesWorkloadDescriptor {
    pub kind: KubernetesWorkloadKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub command: Vec<String>,
    #[serde(default)]
    pub gpu_devices: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KubernetesWorkloadKind {
    RuntimeAdapter,
    ContainerNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesPodLifecycleEvent {
    pub job_id: String,
    pub status: KubernetesPodStatus,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesJobRecord {
    pub job_id: String,
    pub metadata: crate::BatchJobMetadata,
    pub lifecycle: Vec<KubernetesPodLifecycleEvent>,
    pub terminal_status: KubernetesPodStatus,
    pub workspace: KubernetesWorkspaceTransfer,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesLogCapture {
    pub stdout_path: String,
    pub stderr_path: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KubernetesPodStatus {
    pub phase: KubernetesPodPhase,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesExecutionResult {
    pub identity: crate::RemoteExecutionIdentity,
    pub job: KubernetesJobRecord,
    pub pod_status: KubernetesPodStatus,
    pub node_status: NodeStatus,
    pub node_result: NodeResult,
    pub logs: KubernetesLogCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KubernetesPodPhase {
    Pending,
    Running,
    Succeeded,
    Failed,
    Unknown,
}

pub trait KubernetesBackendExecutor: Send + Sync {
    fn execute_job(
        &self,
        request: KubernetesExecutionRequest,
    ) -> Result<KubernetesExecutionResult, String>;
}

#[derive(Debug, Clone, Default)]
pub struct MockKubernetesBackend {
    next_job_id: Arc<Mutex<u64>>,
    requests: Arc<Mutex<Vec<KubernetesExecutionRequest>>>,
    jobs: Arc<Mutex<BTreeMap<String, KubernetesJobRecord>>>,
}

pub fn validate_kubernetes_execution_request(
    request: &KubernetesExecutionRequest,
) -> Result<(), String> {
    validate_remote_execution_payload(&request.payload)?;
    if request.namespace.trim().is_empty() {
        return Err("kubernetes namespace must be non-empty".to_string());
    }
    let backend_id = request.payload.identity.backend_id.as_str();
    if !matches!(backend_id, "k8s" | "kubernetes" | "kubernetes-job") {
        return Err(format!(
            "kubernetes execution request requires backend_id to be k8s, kubernetes, or kubernetes-job; got '{backend_id}'"
        ));
    }
    if request.resources.requests.cpu_millis == 0 || request.resources.requests.memory_mib == 0 {
        return Err("kubernetes resource requests must be greater than zero".to_string());
    }
    if request.policy.active_deadline_seconds == 0 {
        return Err("kubernetes active deadline must be greater than zero".to_string());
    }
    if request.workspace.mounts.is_empty() {
        return Err("kubernetes workspace transfer must define at least one mount".to_string());
    }
    if !request
        .workspace
        .mounts
        .iter()
        .any(|mount| mount.name == "inputs" && mount.mount_path == "/bijux/node/inputs")
    {
        return Err("kubernetes workspace transfer must include inputs mount".to_string());
    }
    if !request
        .workspace
        .mounts
        .iter()
        .any(|mount| mount.name == "outputs" && mount.mount_path == "/bijux/node/outputs")
    {
        return Err("kubernetes workspace transfer must include outputs mount".to_string());
    }
    if !request
        .workspace
        .mounts
        .iter()
        .any(|mount| mount.name == "work" && mount.mount_path == "/bijux/node/work")
    {
        return Err("kubernetes workspace transfer must include work mount".to_string());
    }
    if matches!(request.workload.kind, KubernetesWorkloadKind::ContainerNode)
        && request.workload.image.as_ref().is_none_or(|image| image.trim().is_empty())
    {
        return Err("kubernetes container workload must declare a container image".to_string());
    }
    Ok(())
}

pub fn build_kubernetes_execution_request(
    payload: RemoteNodeExecutionPayload,
    namespace: &str,
) -> KubernetesExecutionRequest {
    let node_contract = kubernetes_node_execution_contract(&payload.node, &payload.params);
    KubernetesExecutionRequest {
        resources: map_node_resources_to_k8s(&node_contract),
        policy: map_node_policy_to_k8s_job(&node_contract),
        workspace: kubernetes_workspace_transfer(&payload.node),
        workload: kubernetes_workload_descriptor(&payload.node),
        payload,
        namespace: namespace.to_string(),
    }
}

pub fn map_kubernetes_pod_status_to_node_status(status: &KubernetesPodStatus) -> NodeStatus {
    match (status.phase, status.reason.as_deref()) {
        (KubernetesPodPhase::Succeeded, _) => NodeStatus::Success,
        (KubernetesPodPhase::Failed, Some("Cancelled")) => NodeStatus::Cancelled,
        (KubernetesPodPhase::Failed, _) => NodeStatus::Failed,
        (KubernetesPodPhase::Pending, _)
        | (KubernetesPodPhase::Running, _)
        | (KubernetesPodPhase::Unknown, _) => NodeStatus::Failed,
    }
}

pub fn kubernetes_pod_status_from_node_result(result: &NodeResult) -> KubernetesPodStatus {
    match result.status {
        NodeStatus::Success => {
            KubernetesPodStatus { phase: KubernetesPodPhase::Succeeded, reason: None }
        }
        NodeStatus::Cancelled => KubernetesPodStatus {
            phase: KubernetesPodPhase::Failed,
            reason: Some("Cancelled".to_string()),
        },
        NodeStatus::Failed => KubernetesPodStatus {
            phase: KubernetesPodPhase::Failed,
            reason: match result.failure.as_ref().map(|failure| failure.operator_class()) {
                Some(FailureClass::Timeout) => Some("DeadlineExceeded".to_string()),
                _ => Some("Error".to_string()),
            },
        },
        NodeStatus::Skipped | NodeStatus::Cached => KubernetesPodStatus {
            phase: KubernetesPodPhase::Unknown,
            reason: Some("NotScheduled".to_string()),
        },
    }
}

impl MockKubernetesBackend {
    pub fn submitted_requests(&self) -> Vec<KubernetesExecutionRequest> {
        self.requests.lock().expect("kubernetes request lock poisoned").clone()
    }

    pub fn job_record(&self, job_id: &str) -> Option<KubernetesJobRecord> {
        self.jobs.lock().expect("kubernetes job lock poisoned").get(job_id).cloned()
    }

    fn allocate_job_id(&self) -> String {
        let mut next_job_id =
            self.next_job_id.lock().expect("kubernetes job counter lock poisoned");
        *next_job_id = next_job_id.saturating_add(1);
        format!("k8s-{}", *next_job_id)
    }
}

impl KubernetesBackendExecutor for MockKubernetesBackend {
    fn execute_job(
        &self,
        request: KubernetesExecutionRequest,
    ) -> Result<KubernetesExecutionResult, String> {
        validate_kubernetes_execution_request(&request)?;
        self.requests.lock().expect("kubernetes request lock poisoned").push(request.clone());

        let adapter = kubernetes_backend_adapter(&request.payload.node.kind)?;
        let remote_result = execute_modeled_payload(request.payload.clone(), adapter)?;
        let job_id = self.allocate_job_id();
        let pod_status = kubernetes_pod_status_from_node_result(&remote_result.node_result);
        let node_status = map_kubernetes_pod_status_to_node_status(&pod_status);
        let job = build_kubernetes_job_record(&job_id, &request, &remote_result, &pod_status);
        let logs = capture_logs(&remote_result.node_result)?;

        self.jobs.lock().expect("kubernetes job lock poisoned").insert(job_id, job.clone());

        Ok(KubernetesExecutionResult {
            identity: remote_result.identity,
            job,
            pod_status,
            node_status,
            node_result: remote_result.node_result,
            logs,
        })
    }
}

fn kubernetes_node_execution_contract(node: &Node, params: &Value) -> NodeExecutionContract {
    let resources = node.resources.as_ref();
    let cpu_units =
        resources.map_or(DEFAULT_CPU_UNITS, |resources| resources.cpu.max(DEFAULT_CPU_UNITS));
    let memory_mib =
        resources.map_or(DEFAULT_MEMORY_MIB, |resources| resources.mem_mb.max(DEFAULT_MEMORY_MIB));
    NodeExecutionContract {
        cpu_units,
        memory_mib,
        timeout_seconds: effective_timeout_seconds(node, params),
        max_retries: node.retry.max_attempts,
        retry_backoff_seconds: ((node.retry.backoff_ms.saturating_add(999)) / 1_000) as u32,
        cancel_grace_seconds: DEFAULT_CANCEL_GRACE_SECONDS,
    }
}

fn kubernetes_workspace_transfer(node: &Node) -> KubernetesWorkspaceTransfer {
    KubernetesWorkspaceTransfer {
        mode: match node.kind {
            NodeKind::Container => KubernetesWorkspaceTransferMode::MountedWorkdir,
            NodeKind::Const | NodeKind::Shell | NodeKind::External(_) => {
                KubernetesWorkspaceTransferMode::StagedArtifacts
            }
        },
        mounts: vec![
            KubernetesVolumeMount {
                name: "inputs".to_string(),
                mount_path: "/bijux/node/inputs".to_string(),
                readonly: true,
            },
            KubernetesVolumeMount {
                name: "outputs".to_string(),
                mount_path: "/bijux/node/outputs".to_string(),
                readonly: false,
            },
            KubernetesVolumeMount {
                name: "work".to_string(),
                mount_path: "/bijux/node/work".to_string(),
                readonly: false,
            },
        ],
        input_artifact_count: node.inputs.len(),
        declared_output_count: node.outputs.len(),
    }
}

fn kubernetes_workload_descriptor(node: &Node) -> KubernetesWorkloadDescriptor {
    let gpu_devices = node.resources.as_ref().map_or(0, |resources| resources.gpu_devices);
    match &node.container {
        Some(spec) => KubernetesWorkloadDescriptor {
            kind: KubernetesWorkloadKind::ContainerNode,
            image: Some(spec.image.clone()),
            command: spec.argv.clone(),
            gpu_devices,
        },
        None => KubernetesWorkloadDescriptor {
            kind: KubernetesWorkloadKind::RuntimeAdapter,
            image: None,
            command: Vec::new(),
            gpu_devices,
        },
    }
}

fn effective_timeout_seconds(node: &Node, params: &Value) -> u32 {
    let timeout_ms =
        node.timeout_ms.or_else(|| params.get("timeout_ms").and_then(|value| value.as_u64()));
    timeout_ms
        .map(|timeout_ms| {
            let timeout_seconds = (timeout_ms.saturating_add(999) / 1_000) as u32;
            timeout_seconds.max(1)
        })
        .unwrap_or(DEFAULT_TIMEOUT_SECONDS)
}

fn kubernetes_backend_adapter(kind: &NodeKind) -> Result<Box<dyn crate::adapter::Adapter>, String> {
    match kind {
        NodeKind::Const => Ok(Box::new(ConstAdapter)),
        NodeKind::Shell => Ok(Box::new(ShellAdapter)),
        NodeKind::Container => Ok(Box::new(ContainerAdapter)),
        NodeKind::External(kind) => Err(format!(
            "kubernetes backend model does not yet execute external adapter kind '{kind}'"
        )),
    }
}

fn build_kubernetes_job_record(
    job_id: &str,
    request: &KubernetesExecutionRequest,
    remote_result: &RemoteNodeExecutionResult,
    terminal_status: &KubernetesPodStatus,
) -> KubernetesJobRecord {
    let metadata = crate::BatchJobMetadata {
        scheduler_id: KUBERNETES_SCHEDULER_ID.to_string(),
        submission_time_unix_ms: remote_result.started_unix_ms,
        run_id: request.payload.identity.run_id.clone(),
        node_id: request.payload.identity.node_id.clone(),
        attempt_id: request.payload.identity.attempt_id.clone(),
        resource_request: render_kubernetes_resource_request(request),
        status_mapping: KUBERNETES_STATUS_MAPPING_ID.to_string(),
    };
    let lifecycle = vec![
        KubernetesPodLifecycleEvent {
            job_id: job_id.to_string(),
            status: KubernetesPodStatus { phase: KubernetesPodPhase::Pending, reason: None },
            unix_ms: metadata.submission_time_unix_ms,
        },
        KubernetesPodLifecycleEvent {
            job_id: job_id.to_string(),
            status: KubernetesPodStatus { phase: KubernetesPodPhase::Running, reason: None },
            unix_ms: remote_result.started_unix_ms,
        },
        KubernetesPodLifecycleEvent {
            job_id: job_id.to_string(),
            status: terminal_status.clone(),
            unix_ms: remote_result.finished_unix_ms,
        },
    ];
    KubernetesJobRecord {
        job_id: job_id.to_string(),
        metadata,
        lifecycle,
        terminal_status: terminal_status.clone(),
        workspace: request.workspace.clone(),
    }
}

fn render_kubernetes_resource_request(request: &KubernetesExecutionRequest) -> String {
    let image = request.workload.image.as_deref().unwrap_or("runtime-adapter");
    let transfer_mode = match request.workspace.mode {
        KubernetesWorkspaceTransferMode::MountedWorkdir => "mounted_workdir",
        KubernetesWorkspaceTransferMode::StagedArtifacts => "staged_artifacts",
    };
    format!(
        "namespace={},request_cpu_millis={},request_mem_mib={},limit_cpu_millis={},limit_mem_mib={},active_deadline_seconds={},retry_backoff_seconds={},transfer_mode={transfer_mode},image={image}",
        request.namespace,
        request.resources.requests.cpu_millis,
        request.resources.requests.memory_mib,
        request.resources.limits.cpu_millis,
        request.resources.limits.memory_mib,
        request.policy.active_deadline_seconds,
        request.policy.retry_backoff_seconds,
    )
}

fn capture_logs(result: &NodeResult) -> Result<KubernetesLogCapture, String> {
    Ok(KubernetesLogCapture {
        stdout_path: result.stdout_path.clone(),
        stderr_path: result.stderr_path.clone(),
        stdout: read_log(&result.stdout_path)?,
        stderr: read_log(&result.stderr_path)?,
    })
}

fn read_log(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read kubernetes log '{path}': {error}"))?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AbsolutePathPolicy, PolicyConfig, RemoteExecutionFingerprintSet, RemoteExecutionIdentity,
        RemoteExecutionWorkspace,
    };
    use bijux_dag_core::parse_graph_strict;
    use serde_json::json;

    fn request_payload(kind: &str, timeout_ms: Option<u64>) -> RemoteNodeExecutionPayload {
        let graph = parse_graph_strict(&format!(
            r#"{{
              "spec": "bijux-dag/v0.1",
              "nodes": [
                {{
                  "id": "node-a",
                  "kind": "{kind}",
                  "inputs": ["seed"],
                  "outputs": [{{"name": "value", "path": "value.txt"}}],
                  "params": {{"argv": ["/bin/sh", "-c", "printf ok > ../outputs/value.txt"]}},
                  "resources": {{"cpu": 2, "mem_mb": 1024, "gpu_devices": 1}},
                  "retry": {{"max_attempts": 3, "backoff_ms": 10000}},
                  "container": {{
                    "image": "example.local/runner@sha256:feedface",
                    "argv": ["/bin/sh", "-c", "printf ok > /bijux/node/outputs/value.txt"],
                    "engine": "docker"
                  }}
                }}
              ],
              "edges": []
            }}"#
        ))
        .expect("parse graph");
        let mut node = graph.nodes[0].clone();
        if kind != "container" {
            node.container = None;
        }
        node.timeout_ms = timeout_ms;
        RemoteNodeExecutionPayload {
            identity: RemoteExecutionIdentity {
                run_id: "run-1".to_string(),
                node_id: node.id.clone(),
                attempt_id: "1".to_string(),
                backend_id: "kubernetes".to_string(),
            },
            graph,
            node,
            params: json!({"argv": ["/bin/sh", "-c", "printf ok > ../outputs/value.txt"]}),
            input_artifacts: Vec::new(),
            workspace: RemoteExecutionWorkspace {
                out_base: "/tmp/out".to_string(),
                cache_dir: None,
            },
            policy: PolicyConfig::default(),
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            planner_contract_version: "bijux-dag-planner/v1".to_string(),
            fingerprints: RemoteExecutionFingerprintSet {
                node_fingerprint: "node".to_string(),
                node_definition_fingerprint: "node-def".to_string(),
                declared_environment_fingerprint: "env".to_string(),
                params_fingerprint: "params".to_string(),
                command_fingerprint: Some("command".to_string()),
                execution_fingerprint: "execution".to_string(),
                evidence_fingerprint: "evidence".to_string(),
                execution_contract_fingerprint: "contract".to_string(),
            },
        }
    }

    #[test]
    fn kubernetes_request_maps_resources_policy_and_workspace_contract() {
        let request =
            build_kubernetes_execution_request(request_payload("container", Some(600_000)), "ns");
        assert_eq!(request.namespace, "ns");
        assert_eq!(request.resources.requests.cpu_millis, 2_000);
        assert_eq!(request.resources.requests.memory_mib, 1_024);
        assert_eq!(request.resources.limits.cpu_millis, 4_000);
        assert_eq!(request.resources.limits.memory_mib, 1_536);
        assert_eq!(request.policy.active_deadline_seconds, 600);
        assert_eq!(request.policy.backoff_limit, 3);
        assert_eq!(request.policy.retry_backoff_seconds, 10);
        assert_eq!(request.workspace.mode, KubernetesWorkspaceTransferMode::MountedWorkdir);
        assert_eq!(request.workspace.mounts.len(), 3);
        assert_eq!(request.workspace.input_artifact_count, 1);
        assert_eq!(request.workspace.declared_output_count, 1);
        assert_eq!(request.workload.kind, KubernetesWorkloadKind::ContainerNode);
        assert_eq!(request.workload.image.as_deref(), Some("example.local/runner@sha256:feedface"));
        validate_kubernetes_execution_request(&request).expect("valid request");
    }

    #[test]
    fn kubernetes_request_uses_staged_artifacts_for_runtime_adapter_nodes() {
        let request = build_kubernetes_execution_request(request_payload("shell", None), "ns");
        assert_eq!(request.workspace.mode, KubernetesWorkspaceTransferMode::StagedArtifacts);
        assert_eq!(request.policy.active_deadline_seconds, 60);
        assert_eq!(request.workload.kind, KubernetesWorkloadKind::RuntimeAdapter);
        assert!(request.workload.image.is_none());
    }

    #[test]
    fn kubernetes_status_mapping_preserves_success_timeout_and_cancelled_boundaries() {
        assert_eq!(
            map_kubernetes_pod_status_to_node_status(&KubernetesPodStatus {
                phase: KubernetesPodPhase::Succeeded,
                reason: None,
            }),
            NodeStatus::Success
        );
        assert_eq!(
            map_kubernetes_pod_status_to_node_status(&KubernetesPodStatus {
                phase: KubernetesPodPhase::Failed,
                reason: Some("Cancelled".to_string()),
            }),
            NodeStatus::Cancelled
        );
        assert_eq!(
            kubernetes_pod_status_from_node_result(&NodeResult {
                status: NodeStatus::Failed,
                stdout_path: String::new(),
                stderr_path: String::new(),
                outputs_dir: String::new(),
                output_evidence: Vec::new(),
                failure: Some(crate::FailureInfo::new(
                    FailureClass::Timeout,
                    "Timeout",
                    "EXEC_TIMEOUT",
                    "timed out",
                    None,
                )),
                attempts: 1,
                attempt_events: Vec::new(),
                container_meta: None,
                adapter_binary_sha256: None,
            }),
            KubernetesPodStatus {
                phase: KubernetesPodPhase::Failed,
                reason: Some("DeadlineExceeded".to_string()),
            }
        );
    }
}
