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
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemKubernetesBackendConfig {
    pub kubectl_command: String,
    pub shared_volume_claim: String,
    pub shared_local_root: PathBuf,
    pub poll_interval_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemKubernetesPaths {
    pub control_dir: PathBuf,
    pub job_spec_path: PathBuf,
    pub submit_response_path: PathBuf,
    pub pod_status_path: PathBuf,
    pub pod_logs_path: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub outputs_dir: PathBuf,
    pub inputs_dir: PathBuf,
    pub work_dir: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SystemKubernetesBackend {
    config: SystemKubernetesBackendConfig,
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

impl SystemKubernetesBackend {
    pub fn new(config: SystemKubernetesBackendConfig) -> Result<Self, String> {
        if config.kubectl_command.trim().is_empty() {
            return Err("kubernetes kubectl command must be non-empty".to_string());
        }
        if config.shared_volume_claim.trim().is_empty() {
            return Err("kubernetes shared volume claim must be non-empty".to_string());
        }
        if config.shared_local_root.as_os_str().is_empty() {
            return Err("kubernetes shared local root must be configured".to_string());
        }
        Ok(Self { config })
    }

    pub fn from_runtime_config(config: &crate::KubernetesRuntimeConfig) -> Result<Self, String> {
        Self::new(SystemKubernetesBackendConfig {
            kubectl_command: config
                .kubectl_command
                .clone()
                .unwrap_or_else(|| "kubectl".to_string()),
            shared_volume_claim: config.shared_volume_claim.clone(),
            shared_local_root: config.shared_local_root.clone(),
            poll_interval_ms: config.poll_interval_ms.max(50),
        })
    }

    fn job_paths(
        &self,
        request: &KubernetesExecutionRequest,
    ) -> Result<SystemKubernetesPaths, String> {
        let run_root = Path::new(&request.payload.workspace.out_base);
        let layout = bijux_dag_artifacts::RunDirLayout::preview(
            run_root,
            Some(&request.payload.identity.run_id),
        )
        .map_err(|error| format!("preview kubernetes run layout: {error}"))?;
        let control_dir = layout
            .node_dir(&request.payload.identity.node_id)
            .join("batch")
            .join("kubernetes")
            .join(format!("attempt-{}", request.payload.identity.attempt_id));
        Ok(SystemKubernetesPaths {
            job_spec_path: control_dir.join("job.json"),
            submit_response_path: control_dir.join("submission.json"),
            pod_status_path: control_dir.join("pod-status.json"),
            pod_logs_path: control_dir.join("pod.log"),
            stdout_path: layout.node_dir(&request.payload.identity.node_id).join("stdout.log"),
            stderr_path: layout.node_dir(&request.payload.identity.node_id).join("stderr.log"),
            outputs_dir: layout.node_outputs_dir(&request.payload.identity.node_id),
            inputs_dir: layout.node_inputs_dir(&request.payload.identity.node_id),
            work_dir: layout.node_work_dir(&request.payload.identity.node_id),
            control_dir,
        })
    }

    fn relative_shared_path(&self, path: &Path) -> Result<String, String> {
        let relative = path.strip_prefix(&self.config.shared_local_root).map_err(|_| {
            format!(
                "kubernetes shared volume root {} does not contain path {}",
                self.config.shared_local_root.display(),
                path.display()
            )
        })?;
        Ok(relative.to_string_lossy().replace('\\', "/"))
    }

    fn volume_mounts(
        &self,
        paths: &SystemKubernetesPaths,
        request: &KubernetesExecutionRequest,
    ) -> Result<Vec<Value>, String> {
        let mut rendered = Vec::with_capacity(request.workspace.mounts.len());
        for mount in &request.workspace.mounts {
            let host_path = match mount.mount_path.as_str() {
                "/bijux/node/inputs" => &paths.inputs_dir,
                "/bijux/node/outputs" => &paths.outputs_dir,
                "/bijux/node/work" => &paths.work_dir,
                other => {
                    return Err(format!(
                        "unsupported kubernetes mount target for system backend: {other}"
                    ))
                }
            };
            rendered.push(json!({
                "name": "shared-run-root",
                "mountPath": mount.mount_path,
                "subPath": self.relative_shared_path(host_path)?,
                "readOnly": mount.readonly
            }));
        }
        Ok(rendered)
    }

    fn container_command(
        &self,
        request: &KubernetesExecutionRequest,
    ) -> Result<Vec<String>, String> {
        let spec = request
            .payload
            .node
            .container
            .as_ref()
            .ok_or_else(|| "kubernetes system backend requires a container node".to_string())?;
        if !crate::effective_env_allowlist(&request.payload.node).is_empty() {
            return Err(
                "kubernetes job backend does not inject ambient env allowlists into direct container jobs"
                    .to_string(),
            );
        }
        crate::resolve_container_argv(&spec.argv, &crate::NodePathBindings::for_container())
    }

    fn container_workdir(&self, request: &KubernetesExecutionRequest) -> Result<String, String> {
        let spec = request
            .payload
            .node
            .container
            .as_ref()
            .ok_or_else(|| "kubernetes system backend requires a container node".to_string())?;
        crate::resolve_container_workdir(
            spec.workdir.as_deref(),
            &crate::NodePathBindings::for_container(),
            request.payload.absolute_path_policy,
        )
    }

    fn job_name(&self, request: &KubernetesExecutionRequest) -> String {
        let base = format!(
            "bijux-{}-{}-a{}-{}",
            sanitize_dns_label(&request.payload.identity.run_id),
            sanitize_dns_label(&request.payload.identity.node_id),
            sanitize_dns_label(&request.payload.identity.attempt_id),
            current_unix_ms(),
        );
        truncate_dns_label(&base, 63)
    }

    fn build_job_spec(
        &self,
        request: &KubernetesExecutionRequest,
        paths: &SystemKubernetesPaths,
        job_id: &str,
    ) -> Result<Value, String> {
        if request.workload.kind != KubernetesWorkloadKind::ContainerNode {
            return Err(format!(
                "kubernetes job backend currently supports container nodes only; node '{}' is {:?}",
                request.payload.node.id, request.payload.node.kind
            ));
        }

        let image = request
            .workload
            .image
            .as_ref()
            .ok_or_else(|| "kubernetes container workload must declare an image".to_string())?;
        let command = self.container_command(request)?;
        let working_dir = self.container_workdir(request)?;
        let volume_mounts = self.volume_mounts(paths, request)?;

        let mut requests_map = serde_json::Map::from_iter([
            ("cpu".to_string(), json!(format!("{}m", request.resources.requests.cpu_millis))),
            ("memory".to_string(), json!(format!("{}Mi", request.resources.requests.memory_mib))),
        ]);
        let mut limits_map = serde_json::Map::from_iter([
            ("cpu".to_string(), json!(format!("{}m", request.resources.limits.cpu_millis))),
            ("memory".to_string(), json!(format!("{}Mi", request.resources.limits.memory_mib))),
        ]);
        if request.workload.gpu_devices > 0 {
            requests_map.insert(
                "nvidia.com/gpu".to_string(),
                json!(request.workload.gpu_devices.to_string()),
            );
            limits_map.insert(
                "nvidia.com/gpu".to_string(),
                json!(request.workload.gpu_devices.to_string()),
            );
        }

        Ok(json!({
            "apiVersion": "batch/v1",
            "kind": "Job",
            "metadata": {
                "name": job_id,
                "namespace": request.namespace,
                "labels": {
                    "app.kubernetes.io/name": "bijux-dag",
                    "bijux.run_id": request.payload.identity.run_id,
                    "bijux.node_id": request.payload.identity.node_id,
                    "bijux.attempt_id": request.payload.identity.attempt_id
                }
            },
            "spec": {
                "backoffLimit": request.policy.backoff_limit,
                "activeDeadlineSeconds": request.policy.active_deadline_seconds,
                "template": {
                    "metadata": {
                        "labels": {
                            "job-name": job_id,
                            "bijux.run_id": request.payload.identity.run_id,
                            "bijux.node_id": request.payload.identity.node_id
                        }
                    },
                    "spec": {
                        "restartPolicy": "Never",
                        "containers": [{
                            "name": "node",
                            "image": image,
                            "command": command,
                            "workingDir": working_dir,
                            "resources": {
                                "requests": Value::Object(requests_map),
                                "limits": Value::Object(limits_map)
                            },
                            "volumeMounts": volume_mounts
                        }],
                        "volumes": [{
                            "name": "shared-run-root",
                            "persistentVolumeClaim": {
                                "claimName": self.config.shared_volume_claim
                            }
                        }]
                    }
                }
            }
        }))
    }

    fn kubectl(&self) -> Command {
        Command::new(&self.config.kubectl_command)
    }

    fn read_pod_status(
        &self,
        namespace: &str,
        job_id: &str,
    ) -> Result<(KubernetesPodStatus, Value), String> {
        let output = self
            .kubectl()
            .args([
                "get",
                "pods",
                "-n",
                namespace,
                "-l",
                &format!("job-name={job_id}"),
                "-o",
                "json",
            ])
            .output()
            .map_err(|error| {
                format!("invoke kubectl '{}' get pods: {error}", self.config.kubectl_command)
            })?;
        if !output.status.success() {
            return Err(format!(
                "kubectl get pods failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        let payload: Value = serde_json::from_slice(&output.stdout)
            .map_err(|error| format!("parse kubernetes pod status: {error}"))?;
        let Some(pod_status) = payload
            .get("items")
            .and_then(Value::as_array)
            .and_then(|items| items.first())
            .and_then(extract_pod_status)
        else {
            return Ok((
                KubernetesPodStatus { phase: KubernetesPodPhase::Pending, reason: None },
                payload,
            ));
        };
        Ok((pod_status, payload))
    }

    fn read_job_logs(&self, namespace: &str, job_id: &str) -> Result<String, String> {
        let output = self
            .kubectl()
            .args(["logs", "-n", namespace, &format!("job/{job_id}")])
            .output()
            .map_err(|error| {
                format!("invoke kubectl '{}' logs: {error}", self.config.kubectl_command)
            })?;
        if !output.status.success() {
            return Err(format!(
                "kubectl logs failed: {}",
                String::from_utf8_lossy(&output.stderr).trim()
            ));
        }
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    }
}

impl KubernetesBackendExecutor for SystemKubernetesBackend {
    fn execute_job(
        &self,
        request: KubernetesExecutionRequest,
    ) -> Result<KubernetesExecutionResult, String> {
        validate_kubernetes_execution_request(&request)?;
        let paths = self.job_paths(&request)?;
        fs::create_dir_all(&paths.control_dir)
            .map_err(|error| format!("create kubernetes control dir: {error}"))?;
        fs::create_dir_all(&paths.outputs_dir)
            .map_err(|error| format!("create kubernetes outputs dir: {error}"))?;
        fs::create_dir_all(&paths.work_dir)
            .map_err(|error| format!("create kubernetes work dir: {error}"))?;

        let job_id = self.job_name(&request);
        let job_spec = self.build_job_spec(&request, &paths, &job_id)?;
        fs::write(
            &paths.job_spec_path,
            serde_json::to_vec_pretty(&job_spec)
                .map_err(|error| format!("serialize kubernetes job spec: {error}"))?,
        )
        .map_err(|error| format!("write kubernetes job spec: {error}"))?;

        let submission = self
            .kubectl()
            .args(["create", "-f", paths.job_spec_path.to_string_lossy().as_ref(), "-o", "json"])
            .output()
            .map_err(|error| {
                format!("invoke kubectl '{}' create: {error}", self.config.kubectl_command)
            })?;
        if !submission.status.success() {
            return Err(format!(
                "kubectl create failed: {}",
                String::from_utf8_lossy(&submission.stderr).trim()
            ));
        }
        fs::write(&paths.submit_response_path, &submission.stdout)
            .map_err(|error| format!("write kubernetes submission response: {error}"))?;

        let submitted_unix_ms = current_unix_ms();
        let mut observed_status =
            KubernetesPodStatus { phase: KubernetesPodPhase::Pending, reason: None };
        let mut lifecycle = vec![KubernetesPodLifecycleEvent {
            job_id: job_id.clone(),
            status: observed_status.clone(),
            unix_ms: submitted_unix_ms,
        }];

        loop {
            thread::sleep(Duration::from_millis(self.config.poll_interval_ms));
            let (status, pod_payload) = self.read_pod_status(&request.namespace, &job_id)?;
            fs::write(
                &paths.pod_status_path,
                serde_json::to_vec_pretty(&pod_payload)
                    .map_err(|error| format!("serialize kubernetes pod status payload: {error}"))?,
            )
            .map_err(|error| format!("write kubernetes pod status payload: {error}"))?;
            if lifecycle.last().map(|event| &event.status) != Some(&status) {
                lifecycle.push(KubernetesPodLifecycleEvent {
                    job_id: job_id.clone(),
                    status: status.clone(),
                    unix_ms: current_unix_ms(),
                });
            }
            observed_status = status;
            if matches!(
                observed_status.phase,
                KubernetesPodPhase::Succeeded
                    | KubernetesPodPhase::Failed
                    | KubernetesPodPhase::Unknown
            ) {
                break;
            }
        }

        let combined_logs = self.read_job_logs(&request.namespace, &job_id).unwrap_or_default();
        fs::write(&paths.pod_logs_path, combined_logs.as_bytes())
            .map_err(|error| format!("write kubernetes pod logs: {error}"))?;
        fs::write(&paths.stdout_path, combined_logs.as_bytes())
            .map_err(|error| format!("write kubernetes stdout log: {error}"))?;
        fs::write(&paths.stderr_path, [])
            .map_err(|error| format!("write kubernetes stderr log: {error}"))?;

        let finished_unix_ms = current_unix_ms();
        let node_result = synthesize_node_result(&request, &paths, &observed_status);
        let remote_result = RemoteNodeExecutionResult {
            identity: request.payload.identity.clone(),
            node_result: node_result.clone(),
            started_unix_ms: submitted_unix_ms,
            finished_unix_ms,
        };
        let mut job =
            build_kubernetes_job_record(&job_id, &request, &remote_result, &observed_status);
        job.lifecycle = lifecycle;
        let logs = KubernetesLogCapture {
            stdout_path: paths.stdout_path.display().to_string(),
            stderr_path: paths.stderr_path.display().to_string(),
            stdout: combined_logs,
            stderr: String::new(),
        };

        Ok(KubernetesExecutionResult {
            identity: request.payload.identity,
            job,
            pod_status: observed_status.clone(),
            node_status: map_kubernetes_pod_status_to_node_status(&observed_status),
            node_result,
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
            NodeKind::Const
            | NodeKind::Shell
            | NodeKind::Python
            | NodeKind::Http
            | NodeKind::FileTransform
            | NodeKind::External(_) => KubernetesWorkspaceTransferMode::StagedArtifacts,
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
        NodeKind::Http => Ok(Box::new(crate::http_adapter::HttpRequestAdapter)),
        NodeKind::FileTransform => {
            Ok(Box::new(crate::file_transform_adapter::FileTransformAdapter))
        }
        NodeKind::Shell => Ok(Box::new(ShellAdapter)),
        NodeKind::Python => Ok(Box::new(crate::python_adapter::PythonFunctionAdapter)),
        NodeKind::Container => Ok(Box::new(ContainerAdapter)),
        NodeKind::External(kind) => Err(format!(
            "kubernetes backend model does not yet execute external adapter kind '{kind}'"
        )),
    }
}

fn synthesize_node_result(
    _request: &KubernetesExecutionRequest,
    paths: &SystemKubernetesPaths,
    pod_status: &KubernetesPodStatus,
) -> NodeResult {
    let failure = match pod_status.reason.as_deref() {
        Some("DeadlineExceeded") => Some(crate::FailureInfo::new(
            FailureClass::Timeout,
            "Timeout",
            "KUBERNETES_DEADLINE_EXCEEDED",
            "kubernetes job exceeded its active deadline",
            None,
        )),
        Some("Cancelled") => Some(crate::FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "EXEC_CANCELLED",
            "kubernetes job was cancelled",
            None,
        )),
        Some(reason) if pod_status.phase == KubernetesPodPhase::Failed => {
            Some(crate::FailureInfo::new(
                FailureClass::Execution,
                "Execution",
                "KUBERNETES_JOB_FAILED",
                &format!("kubernetes job failed with reason {reason}"),
                None,
            ))
        }
        _ if pod_status.phase == KubernetesPodPhase::Failed => Some(crate::FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "KUBERNETES_JOB_FAILED",
            "kubernetes job failed",
            None,
        )),
        _ => None,
    };
    let status = map_kubernetes_pod_status_to_node_status(pod_status);
    NodeResult {
        status,
        stdout_path: paths.stdout_path.display().to_string(),
        stderr_path: paths.stderr_path.display().to_string(),
        outputs_dir: paths.outputs_dir.display().to_string(),
        output_evidence: Vec::new(),
        failure,
        attempts: 1,
        attempt_events: Vec::new(),
        container_meta: None,
        adapter_binary_sha256: None,
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

fn extract_pod_status(pod: &Value) -> Option<KubernetesPodStatus> {
    let status = pod.get("status")?;
    let phase = match status.get("phase").and_then(Value::as_str)? {
        "Pending" => KubernetesPodPhase::Pending,
        "Running" => KubernetesPodPhase::Running,
        "Succeeded" => KubernetesPodPhase::Succeeded,
        "Failed" => KubernetesPodPhase::Failed,
        _ => KubernetesPodPhase::Unknown,
    };
    let reason = status
        .get("containerStatuses")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .and_then(|entry| entry.get("state"))
        .and_then(Value::as_object)
        .and_then(|state| {
            for key in ["terminated", "waiting", "running"] {
                if let Some(reason) = state
                    .get(key)
                    .and_then(Value::as_object)
                    .and_then(|object| object.get("reason"))
                    .and_then(Value::as_str)
                {
                    return Some(reason.to_string());
                }
            }
            None
        })
        .or_else(|| status.get("reason").and_then(Value::as_str).map(ToOwned::to_owned));
    Some(KubernetesPodStatus { phase, reason })
}

fn current_unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis()).unwrap_or(0)
}

fn sanitize_dns_label(value: &str) -> String {
    let mut rendered = String::with_capacity(value.len());
    for ch in value.chars() {
        let mapped = if ch.is_ascii_alphanumeric() { ch.to_ascii_lowercase() } else { '-' };
        rendered.push(mapped);
    }
    rendered.trim_matches('-').to_string()
}

fn truncate_dns_label(value: &str, max_len: usize) -> String {
    let truncated = value.chars().take(max_len).collect::<String>();
    truncated.trim_matches('-').to_string()
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
    use std::fs;
    use std::path::Path;

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

    fn system_backend_payload(out_base: &Path, run_id: &str) -> RemoteNodeExecutionPayload {
        let graph = parse_graph_strict(
            r#"{
              "spec": "bijux-dag/v0.1",
              "nodes": [
                {
                  "id": "render",
                  "kind": "container",
                  "inputs": ["seed"],
                  "outputs": [
                    {"name": "value", "path": "value.txt"},
                    {"name": "workdir", "path": "workdir.txt"}
                  ],
                  "resources": {"cpu": 2, "mem_mb": 1024},
                  "retry": {"max_attempts": 2, "backoff_ms": 5000},
                  "container": {
                    "image": "example.local/runner@sha256:feedface",
                    "argv": ["/bin/sh", "-c", "printf ignored > /bijux/node/outputs/value.txt"],
                    "workdir": "scratch",
                    "engine": "docker"
                  },
                  "params": {}
                }
              ],
              "edges": []
            }"#,
        )
        .expect("parse graph");
        let node = graph.nodes[0].clone();
        RemoteNodeExecutionPayload {
            identity: RemoteExecutionIdentity {
                run_id: run_id.to_string(),
                node_id: node.id.clone(),
                attempt_id: "1".to_string(),
                backend_id: "kubernetes".to_string(),
            },
            graph,
            node,
            params: json!({}),
            input_artifacts: Vec::new(),
            workspace: RemoteExecutionWorkspace {
                out_base: out_base.display().to_string(),
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

    fn write_executable(path: &Path, contents: &str) {
        fs::write(path, contents).expect("write executable");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).expect("meta").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("chmod");
        }
    }

    #[test]
    fn system_kubernetes_backend_executes_container_jobs_through_shared_volume_contract() {
        let temp = tempfile::tempdir().expect("temp dir");
        let state_dir = temp.path().join("state");
        fs::create_dir_all(&state_dir).expect("state dir");
        let kubectl = temp.path().join("kubectl");
        write_executable(
            &kubectl,
            &format!(
                r#"#!/bin/sh
set -eu
STATE_DIR={state:?}
SHARED_ROOT={shared_root:?}
command="$1"
shift
case "$command" in
  create)
    spec=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -f) spec="$2"; shift 2 ;;
        -o) shift 2 ;;
        *) shift ;;
      esac
    done
    job_id=$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["metadata"]["name"])' "$spec")
    outputs_sub=$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); mounts=data["spec"]["template"]["spec"]["containers"][0]["volumeMounts"]; print(next(m["subPath"] for m in mounts if m["mountPath"]=="/bijux/node/outputs"))' "$spec")
    workdir=$(python3 -c 'import json,sys; data=json.load(open(sys.argv[1])); print(data["spec"]["template"]["spec"]["containers"][0]["workingDir"])' "$spec")
    mkdir -p "$STATE_DIR" "$SHARED_ROOT/$outputs_sub"
    printf 'k8s-system-value' > "$SHARED_ROOT/$outputs_sub/value.txt"
    printf '%s' "$workdir" > "$SHARED_ROOT/$outputs_sub/workdir.txt"
    printf 'Succeeded' > "$STATE_DIR/$job_id.phase"
    printf 'Completed' > "$STATE_DIR/$job_id.reason"
    printf 'kubernetes backend log\n' > "$STATE_DIR/$job_id.log"
    python3 - "$job_id" <<'PY'
import json, sys
print(json.dumps({{"metadata": {{"name": sys.argv[1]}}}}))
PY
    ;;
  get)
    label=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -l) label="$2"; shift 2 ;;
        -o) shift 2 ;;
        -n) shift 2 ;;
        pods) shift ;;
        *) shift ;;
      esac
    done
    job_id=${{label#job-name=}}
    phase=$(cat "$STATE_DIR/$job_id.phase")
    reason=$(cat "$STATE_DIR/$job_id.reason")
    python3 - "$phase" "$reason" <<'PY'
import json, sys
phase, reason = sys.argv[1], sys.argv[2]
print(json.dumps({{
  "items": [{{
    "status": {{
      "phase": phase,
      "containerStatuses": [{{
        "state": {{"terminated": {{"reason": reason}}}}
      }}]
    }}
  }}]
}}))
PY
    ;;
  logs)
    target=""
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -n) shift 2 ;;
        *) target="$1"; shift ;;
      esac
    done
    job_id=${{target#job/}}
    cat "$STATE_DIR/$job_id.log"
    ;;
  *)
    echo "unexpected kubectl command: $command" >&2
    exit 1
    ;;
esac
"#,
                state = state_dir.display().to_string(),
                shared_root = temp.path().display().to_string(),
            ),
        );

        let payload = system_backend_payload(temp.path(), "k8s-system-proof");
        let request = build_kubernetes_execution_request(payload, "bijux-jobs");
        let layout =
            bijux_dag_artifacts::RunDirLayout::preview(temp.path(), Some("k8s-system-proof"))
                .expect("layout");
        fs::create_dir_all(layout.node_inputs_dir("render").join("seed")).expect("inputs");
        fs::create_dir_all(layout.node_outputs_dir("render")).expect("outputs");
        fs::create_dir_all(layout.node_work_dir("render")).expect("work");

        let backend = SystemKubernetesBackend::new(SystemKubernetesBackendConfig {
            kubectl_command: kubectl.display().to_string(),
            shared_volume_claim: "bijux-run-pvc".to_string(),
            shared_local_root: temp.path().to_path_buf(),
            poll_interval_ms: 50,
        })
        .expect("backend");

        let result = backend.execute_job(request).expect("kubernetes execute");

        assert_eq!(result.pod_status.phase, KubernetesPodPhase::Succeeded);
        assert_eq!(result.node_status, NodeStatus::Success);
        assert_eq!(
            fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("value.txt"))
                .expect("value output"),
            "k8s-system-value"
        );
        assert_eq!(
            fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("workdir.txt"))
                .expect("workdir output"),
            "/bijux/node/work/scratch"
        );
        assert!(result.logs.stdout.contains("kubernetes backend log"));
        assert_eq!(result.job.metadata.scheduler_id, "kubernetes");
        assert!(result.job.metadata.resource_request.contains("transfer_mode=mounted_workdir"));
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
