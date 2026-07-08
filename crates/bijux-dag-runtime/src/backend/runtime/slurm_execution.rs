use crate::backend_cluster::{
    map_node_to_hpc_queue_partition, map_timeout_to_hpc_walltime, HpcNodeExecutionContract,
};
use crate::remote_execution_model::{
    validate_remote_execution_payload, MockRemoteWorker, RemoteNodeExecutionPayload,
    RemoteNodeExecutionResult, RemoteWorkerExecutor,
};
use crate::{BatchJobMetadata, FailureClass, NodeResult, NodeStatus};
use bijux_dag_core::Node;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const DEFAULT_CPU_CORES: u32 = 1;
const DEFAULT_MEMORY_MIB: u32 = 256;
const DEFAULT_TIMEOUT_SECONDS: u32 = 60;
const SLURM_SCHEDULER_ID: &str = "slurm";
const SLURM_STATUS_MAPPING_ID: &str = "slurm-default";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlurmSchedulerRequest {
    pub cpu_cores: u32,
    pub memory_mib: u32,
    pub walltime: String,
    pub queue: String,
    pub partition: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub account: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmExecutionRequest {
    pub payload: RemoteNodeExecutionPayload,
    pub scheduler: SlurmSchedulerRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlurmJobLifecycleEvent {
    pub job_id: String,
    pub status: SlurmJobStatus,
    pub unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlurmJobRecord {
    pub job_id: String,
    pub metadata: BatchJobMetadata,
    pub lifecycle: Vec<SlurmJobLifecycleEvent>,
    pub terminal_status: SlurmJobStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlurmLogCapture {
    pub stdout_path: String,
    pub stderr_path: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlurmExecutionResult {
    pub identity: crate::RemoteExecutionIdentity,
    pub job: SlurmJobRecord,
    pub scheduler_status: SlurmJobStatus,
    pub node_status: NodeStatus,
    pub node_result: NodeResult,
    pub logs: SlurmLogCapture,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SlurmJobStatus {
    Submitted,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    Preempted,
}

pub trait SlurmBackendExecutor: Send + Sync {
    fn execute_job(&self, request: SlurmExecutionRequest) -> Result<SlurmExecutionResult, String>;
}

#[derive(Debug, Clone, Default)]
pub struct MockSlurmBackend {
    next_job_id: Arc<Mutex<u64>>,
    requests: Arc<Mutex<Vec<SlurmExecutionRequest>>>,
    jobs: Arc<Mutex<BTreeMap<String, SlurmJobRecord>>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSlurmBackendConfig {
    pub sbatch_command: String,
    pub sacct_command: String,
    pub poll_interval_ms: u64,
    pub worker_command: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemSlurmPaths {
    pub payload_path: PathBuf,
    pub result_path: PathBuf,
    pub node_dir: PathBuf,
    pub stdout_path: PathBuf,
    pub stderr_path: PathBuf,
    pub script_path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct SystemSlurmBackend {
    config: SystemSlurmBackendConfig,
}

pub fn validate_slurm_scheduler_request(request: &SlurmSchedulerRequest) -> Result<(), String> {
    if request.cpu_cores == 0 {
        return Err("slurm cpu_cores must be greater than zero".to_string());
    }
    if request.memory_mib == 0 {
        return Err("slurm memory_mib must be greater than zero".to_string());
    }
    for (label, value) in [
        ("walltime", request.walltime.as_str()),
        ("queue", request.queue.as_str()),
        ("partition", request.partition.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("slurm {label} must be non-empty"));
        }
    }
    if request.account.as_ref().is_some_and(|account| account.trim().is_empty()) {
        return Err("slurm account must be non-empty when provided".to_string());
    }
    Ok(())
}

pub fn validate_slurm_execution_request(request: &SlurmExecutionRequest) -> Result<(), String> {
    validate_remote_execution_payload(&request.payload)?;
    validate_slurm_scheduler_request(&request.scheduler)?;
    let backend_id = request.payload.identity.backend_id.as_str();
    if !matches!(backend_id, "slurm" | "hpc" | "slurm-backend") {
        return Err(format!(
            "slurm execution request requires backend_id to be slurm, hpc, or slurm-backend; got '{backend_id}'"
        ));
    }
    Ok(())
}

pub fn build_slurm_scheduler_request(
    node: &Node,
    params: &Value,
    default_queue: &str,
    default_partition: &str,
) -> SlurmSchedulerRequest {
    let resources = node.resources.as_ref();
    let cpu_cores =
        resources.map_or(DEFAULT_CPU_CORES, |resources| resources.cpu.max(DEFAULT_CPU_CORES));
    let memory_mib =
        resources.map_or(DEFAULT_MEMORY_MIB, |resources| resources.mem_mb.max(DEFAULT_MEMORY_MIB));
    let timeout_seconds = effective_timeout_seconds(node, params);
    let hpc_contract = HpcNodeExecutionContract {
        cpu_units: cpu_cores,
        memory_mib,
        timeout_seconds,
        requested_partition: slurm_tag_value(&node.tags, "slurm.partition")
            .or_else(|| slurm_tag_value(&node.tags, "hpc.partition")),
        requested_queue: slurm_tag_value(&node.tags, "slurm.queue")
            .or_else(|| slurm_tag_value(&node.tags, "hpc.queue")),
    };
    let mapping = map_node_to_hpc_queue_partition(&hpc_contract, default_queue, default_partition);
    SlurmSchedulerRequest {
        cpu_cores,
        memory_mib,
        walltime: map_timeout_to_hpc_walltime(timeout_seconds),
        queue: mapping.queue,
        partition: mapping.partition,
        account: slurm_tag_value(&node.tags, "slurm.account")
            .or_else(|| slurm_tag_value(&node.tags, "hpc.account")),
    }
}

pub fn build_slurm_execution_request(
    payload: RemoteNodeExecutionPayload,
    default_queue: &str,
    default_partition: &str,
) -> SlurmExecutionRequest {
    let scheduler = build_slurm_scheduler_request(
        &payload.node,
        &payload.params,
        default_queue,
        default_partition,
    );
    SlurmExecutionRequest { payload, scheduler }
}

pub fn map_slurm_job_status_to_node_status(status: SlurmJobStatus) -> NodeStatus {
    match status {
        SlurmJobStatus::Completed => NodeStatus::Success,
        SlurmJobStatus::Cancelled => NodeStatus::Cancelled,
        SlurmJobStatus::Submitted
        | SlurmJobStatus::Running
        | SlurmJobStatus::Failed
        | SlurmJobStatus::Timeout
        | SlurmJobStatus::Preempted => NodeStatus::Failed,
    }
}

pub fn slurm_job_status_from_node_result(result: &NodeResult) -> SlurmJobStatus {
    match result.status {
        NodeStatus::Success => SlurmJobStatus::Completed,
        NodeStatus::Cancelled => SlurmJobStatus::Cancelled,
        NodeStatus::Failed => match result.failure.as_ref().map(|failure| failure.operator_class())
        {
            Some(FailureClass::Timeout) => SlurmJobStatus::Timeout,
            Some(FailureClass::Infrastructure)
                if result
                    .failure
                    .as_ref()
                    .is_some_and(|failure| failure.code == "SLURM_PREEMPTED") =>
            {
                SlurmJobStatus::Preempted
            }
            _ => SlurmJobStatus::Failed,
        },
        NodeStatus::Skipped | NodeStatus::Cached => SlurmJobStatus::Failed,
    }
}

impl MockSlurmBackend {
    pub fn submitted_requests(&self) -> Vec<SlurmExecutionRequest> {
        self.requests.lock().expect("slurm request lock poisoned").clone()
    }

    pub fn job_record(&self, job_id: &str) -> Option<SlurmJobRecord> {
        self.jobs.lock().expect("slurm job lock poisoned").get(job_id).cloned()
    }

    fn allocate_job_id(&self) -> String {
        let mut next_job_id = self.next_job_id.lock().expect("slurm job counter lock poisoned");
        *next_job_id = next_job_id.saturating_add(1);
        format!("slurm-{}", *next_job_id)
    }
}

impl SystemSlurmBackend {
    pub fn new(config: SystemSlurmBackendConfig) -> Result<Self, String> {
        if config.sbatch_command.trim().is_empty() {
            return Err("slurm sbatch command must be non-empty".to_string());
        }
        if config.sacct_command.trim().is_empty() {
            return Err("slurm sacct command must be non-empty".to_string());
        }
        if config.worker_command.is_empty()
            || config.worker_command.iter().any(|entry| entry.trim().is_empty())
        {
            return Err("slurm worker command must be configured".to_string());
        }
        Ok(Self { config })
    }

    pub fn from_runtime_config(config: &crate::SlurmRuntimeConfig) -> Result<Self, String> {
        Self::new(SystemSlurmBackendConfig {
            sbatch_command: config.sbatch_command.clone().unwrap_or_else(|| "sbatch".to_string()),
            sacct_command: config.sacct_command.clone().unwrap_or_else(|| "sacct".to_string()),
            poll_interval_ms: config.poll_interval_ms.max(50),
            worker_command: config.worker_command.clone(),
        })
    }

    fn job_paths(&self, request: &SlurmExecutionRequest) -> Result<SystemSlurmPaths, String> {
        let run_root = Path::new(&request.payload.workspace.out_base);
        let layout = bijux_dag_artifacts::RunDirLayout::preview(
            run_root,
            Some(&request.payload.identity.run_id),
        )
        .map_err(|error| format!("preview slurm run layout: {error}"))?;
        let node_dir = layout
            .node_dir(&request.payload.identity.node_id)
            .join("batch")
            .join("slurm")
            .join(format!("attempt-{}", request.payload.identity.attempt_id));
        Ok(SystemSlurmPaths {
            payload_path: node_dir.join("payload.json"),
            result_path: node_dir.join("result.json"),
            stdout_path: node_dir.join("scheduler.stdout.log"),
            stderr_path: node_dir.join("scheduler.stderr.log"),
            script_path: node_dir.join("submit.sh"),
            node_dir,
        })
    }
}

impl SlurmBackendExecutor for MockSlurmBackend {
    fn execute_job(&self, request: SlurmExecutionRequest) -> Result<SlurmExecutionResult, String> {
        validate_slurm_execution_request(&request)?;
        self.requests.lock().expect("slurm request lock poisoned").push(request.clone());

        let remote_result = MockRemoteWorker.execute_payload(request.payload.clone())?;
        let job_id = self.allocate_job_id();
        let scheduler_status = slurm_job_status_from_node_result(&remote_result.node_result);
        let node_status = map_slurm_job_status_to_node_status(scheduler_status);
        let job = build_slurm_job_record(&job_id, &request, &remote_result, scheduler_status);
        let logs = capture_logs(&remote_result.node_result)?;

        self.jobs.lock().expect("slurm job lock poisoned").insert(job_id, job.clone());

        Ok(SlurmExecutionResult {
            identity: remote_result.identity,
            job,
            scheduler_status,
            node_status,
            node_result: remote_result.node_result,
            logs,
        })
    }
}

impl SlurmBackendExecutor for SystemSlurmBackend {
    fn execute_job(&self, request: SlurmExecutionRequest) -> Result<SlurmExecutionResult, String> {
        validate_slurm_execution_request(&request)?;
        let paths = self.job_paths(&request)?;
        fs::create_dir_all(&paths.node_dir)
            .map_err(|error| format!("create slurm control dir: {error}"))?;
        fs::write(
            &paths.payload_path,
            serde_json::to_vec_pretty(&request.payload)
                .map_err(|error| format!("serialize slurm payload: {error}"))?,
        )
        .map_err(|error| format!("write slurm payload: {error}"))?;
        let worker_invocation = render_worker_invocation(
            &self.config.worker_command,
            &paths.payload_path,
            &paths.result_path,
        );
        fs::write(&paths.script_path, worker_invocation.as_bytes())
            .map_err(|error| format!("write slurm submit script: {error}"))?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&paths.script_path)
                .map_err(|error| format!("stat slurm submit script: {error}"))?
                .permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&paths.script_path, permissions)
                .map_err(|error| format!("chmod slurm submit script: {error}"))?;
        }

        let mut submit = Command::new(&self.config.sbatch_command);
        submit
            .arg("--parsable")
            .arg("--cpus-per-task")
            .arg(request.scheduler.cpu_cores.to_string())
            .arg("--mem")
            .arg(format!("{}M", request.scheduler.memory_mib))
            .arg("--time")
            .arg(&request.scheduler.walltime)
            .arg("--partition")
            .arg(&request.scheduler.partition)
            .arg("--qos")
            .arg(&request.scheduler.queue)
            .arg("--output")
            .arg(&paths.stdout_path)
            .arg("--error")
            .arg(&paths.stderr_path);
        if let Some(account) = &request.scheduler.account {
            submit.arg("--account").arg(account);
        }
        submit.arg(&paths.script_path);
        let submission = submit
            .output()
            .map_err(|error| format!("invoke sbatch '{}': {error}", self.config.sbatch_command))?;
        if !submission.status.success() {
            return Err(format!(
                "sbatch failed: {}",
                String::from_utf8_lossy(&submission.stderr).trim()
            ));
        }
        let job_id = parse_slurm_job_id(&String::from_utf8_lossy(&submission.stdout))?;
        let submitted_unix_ms = current_unix_ms();

        let mut terminal_status = SlurmJobStatus::Submitted;
        let mut remote_result = None;
        loop {
            thread::sleep(Duration::from_millis(self.config.poll_interval_ms));
            let poll = Command::new(&self.config.sacct_command)
                .arg("-P")
                .arg("-n")
                .arg("-j")
                .arg(&job_id)
                .arg("--format")
                .arg("State,ExitCode")
                .output()
                .map_err(|error| {
                    format!("invoke sacct '{}': {error}", self.config.sacct_command)
                })?;
            if !poll.status.success() {
                return Err(format!(
                    "sacct failed: {}",
                    String::from_utf8_lossy(&poll.stderr).trim()
                ));
            }
            if let Some(state) = parse_slurm_job_status(&String::from_utf8_lossy(&poll.stdout)) {
                terminal_status = state;
            }
            if matches!(
                terminal_status,
                SlurmJobStatus::Completed
                    | SlurmJobStatus::Failed
                    | SlurmJobStatus::Cancelled
                    | SlurmJobStatus::Timeout
                    | SlurmJobStatus::Preempted
            ) {
                if paths.result_path.exists() {
                    let raw = fs::read_to_string(&paths.result_path)
                        .map_err(|error| format!("read slurm result payload: {error}"))?;
                    remote_result = Some(
                        serde_json::from_str::<RemoteNodeExecutionResult>(&raw)
                            .map_err(|error| format!("parse slurm result payload: {error}"))?,
                    );
                }
                break;
            }
        }

        let remote_result = remote_result.unwrap_or_else(|| {
            synthesize_remote_result(&request, &paths, terminal_status, submitted_unix_ms)
        });
        let job = build_slurm_job_record(&job_id, &request, &remote_result, terminal_status);
        let logs = capture_logs(&remote_result.node_result)?;
        Ok(SlurmExecutionResult {
            identity: remote_result.identity.clone(),
            scheduler_status: terminal_status,
            node_status: remote_result.node_result.status.clone(),
            node_result: remote_result.node_result,
            job,
            logs,
        })
    }
}

fn render_worker_invocation(
    worker_command: &[String],
    payload_path: &Path,
    result_path: &Path,
) -> String {
    let mut args = worker_command.iter().map(|entry| shell_quote(entry)).collect::<Vec<_>>();
    args.push(shell_quote(payload_path.to_string_lossy().as_ref()));
    args.push("--result".to_string());
    args.push(shell_quote(result_path.to_string_lossy().as_ref()));
    args.push("--in-place".to_string());
    format!("#!/bin/sh\nset -eu\nexport BIJUX_DAG_ENABLE_INTERNAL=1\n{}\n", args.join(" "))
}

fn parse_slurm_job_id(stdout: &str) -> Result<String, String> {
    let job_id = stdout.trim().split(';').next().unwrap_or_default().trim();
    if job_id.is_empty() {
        return Err("sbatch did not return a job id".to_string());
    }
    Ok(job_id.to_string())
}

fn parse_slurm_job_status(stdout: &str) -> Option<SlurmJobStatus> {
    let line = stdout.lines().find(|line| !line.trim().is_empty())?;
    let state = line.split('|').next()?.trim().to_ascii_uppercase();
    Some(match state.as_str() {
        "COMPLETED" => SlurmJobStatus::Completed,
        "FAILED" => SlurmJobStatus::Failed,
        "CANCELLED" => SlurmJobStatus::Cancelled,
        "TIMEOUT" => SlurmJobStatus::Timeout,
        "PREEMPTED" => SlurmJobStatus::Preempted,
        "RUNNING" => SlurmJobStatus::Running,
        _ => SlurmJobStatus::Submitted,
    })
}

fn synthesize_remote_result(
    request: &SlurmExecutionRequest,
    paths: &SystemSlurmPaths,
    terminal_status: SlurmJobStatus,
    submitted_unix_ms: u128,
) -> RemoteNodeExecutionResult {
    let failure = match terminal_status {
        SlurmJobStatus::Timeout => Some(crate::FailureInfo::new(
            FailureClass::Timeout,
            "Timeout",
            "SLURM_TIMEOUT",
            "slurm job exceeded its walltime",
            None,
        )),
        SlurmJobStatus::Cancelled => Some(crate::FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "EXEC_CANCELLED",
            "slurm job was cancelled",
            None,
        )),
        SlurmJobStatus::Preempted => Some(crate::FailureInfo::new(
            FailureClass::Infrastructure,
            "Infrastructure",
            "SLURM_PREEMPTED",
            "slurm job was preempted",
            None,
        )),
        SlurmJobStatus::Completed => None,
        _ => Some(crate::FailureInfo::new(
            FailureClass::Execution,
            "Execution",
            "SLURM_JOB_FAILED",
            "slurm job failed before writing a node result payload",
            None,
        )),
    };
    let status = match terminal_status {
        SlurmJobStatus::Completed => NodeStatus::Success,
        SlurmJobStatus::Cancelled => NodeStatus::Cancelled,
        _ => NodeStatus::Failed,
    };
    RemoteNodeExecutionResult {
        identity: request.payload.identity.clone(),
        node_result: NodeResult {
            status,
            stdout_path: paths.stdout_path.display().to_string(),
            stderr_path: paths.stderr_path.display().to_string(),
            outputs_dir: Path::new(&request.payload.workspace.out_base)
                .join(format!("run.tmp-{}", request.payload.identity.run_id))
                .join("nodes")
                .join(&request.payload.identity.node_id)
                .join("outputs")
                .display()
                .to_string(),
            output_evidence: Vec::new(),
            failure,
            attempts: 1,
            attempt_events: Vec::new(),
            container_meta: None,
            adapter_binary_sha256: None,
        },
        started_unix_ms: submitted_unix_ms,
        finished_unix_ms: current_unix_ms(),
    }
}

fn current_unix_ms() -> u128 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_millis()).unwrap_or(0)
}

fn shell_quote(value: &str) -> String {
    let escaped = value.replace('\'', "'\"'\"'");
    format!("'{escaped}'")
}

fn slurm_tag_value(tags: &[String], key: &str) -> Option<String> {
    tags.iter().find_map(|tag| {
        let value = tag.strip_prefix(key)?.strip_prefix(':')?.trim();
        (!value.is_empty()).then(|| value.to_string())
    })
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

fn build_slurm_job_record(
    job_id: &str,
    request: &SlurmExecutionRequest,
    remote_result: &RemoteNodeExecutionResult,
    terminal_status: SlurmJobStatus,
) -> SlurmJobRecord {
    let metadata = BatchJobMetadata {
        scheduler_id: SLURM_SCHEDULER_ID.to_string(),
        submission_time_unix_ms: remote_result.started_unix_ms,
        run_id: request.payload.identity.run_id.clone(),
        node_id: request.payload.identity.node_id.clone(),
        attempt_id: request.payload.identity.attempt_id.clone(),
        resource_request: render_slurm_resource_request(&request.scheduler),
        status_mapping: SLURM_STATUS_MAPPING_ID.to_string(),
    };
    let lifecycle = vec![
        SlurmJobLifecycleEvent {
            job_id: job_id.to_string(),
            status: SlurmJobStatus::Submitted,
            unix_ms: metadata.submission_time_unix_ms,
        },
        SlurmJobLifecycleEvent {
            job_id: job_id.to_string(),
            status: SlurmJobStatus::Running,
            unix_ms: remote_result.started_unix_ms,
        },
        SlurmJobLifecycleEvent {
            job_id: job_id.to_string(),
            status: terminal_status,
            unix_ms: remote_result.finished_unix_ms,
        },
    ];
    SlurmJobRecord { job_id: job_id.to_string(), metadata, lifecycle, terminal_status }
}

fn render_slurm_resource_request(request: &SlurmSchedulerRequest) -> String {
    let account =
        request.account.as_ref().map_or_else(|| "none".to_string(), |account| account.clone());
    format!(
        "cpu={},mem_mib={},walltime={},queue={},partition={},account={account}",
        request.cpu_cores, request.memory_mib, request.walltime, request.queue, request.partition
    )
}

fn capture_logs(result: &NodeResult) -> Result<SlurmLogCapture, String> {
    Ok(SlurmLogCapture {
        stdout_path: result.stdout_path.clone(),
        stderr_path: result.stderr_path.clone(),
        stdout: read_log(&result.stdout_path)?,
        stderr: read_log(&result.stderr_path)?,
    })
}

fn read_log(path: &str) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read slurm log '{path}': {error}"))?;
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

    fn request_payload(tags: &[&str], timeout_ms: Option<u64>) -> RemoteNodeExecutionPayload {
        let graph = parse_graph_strict(
            r#"{
              "spec": "bijux-dag/v0.1",
              "nodes": [
                {
                  "id": "shell-node",
                  "kind": "shell",
                  "outputs": [{"name": "value", "path": "value.txt"}],
                  "params": {"argv": ["/bin/sh", "-c", "printf value > ../outputs/value.txt"]},
                  "resources": {"cpu": 8, "mem_mb": 16384},
                  "tags": ["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:ml-team"]
                }
              ],
              "edges": []
            }"#,
        )
        .expect("parse graph");
        let mut node = graph.nodes[0].clone();
        node.tags = tags.iter().map(|tag| (*tag).to_string()).collect();
        node.timeout_ms = timeout_ms;
        RemoteNodeExecutionPayload {
            identity: RemoteExecutionIdentity {
                run_id: "run-1".to_string(),
                node_id: node.id.clone(),
                attempt_id: "1".to_string(),
                backend_id: "slurm".to_string(),
            },
            graph,
            node,
            params: json!({"argv": ["/bin/sh", "-c", "printf value > ../outputs/value.txt"]}),
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
    fn scheduler_request_maps_resources_timeout_and_slurm_tags() {
        let payload = request_payload(
            &["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:ml-team"],
            Some(3_661_000),
        );
        let request = build_slurm_execution_request(payload, "default", "cpu");
        assert_eq!(request.scheduler.cpu_cores, 8);
        assert_eq!(request.scheduler.memory_mib, 16_384);
        assert_eq!(request.scheduler.queue, "priority");
        assert_eq!(request.scheduler.partition, "gpu");
        assert_eq!(request.scheduler.account.as_deref(), Some("ml-team"));
        assert_eq!(request.scheduler.walltime, "01:01:01");
    }

    #[test]
    fn scheduler_request_falls_back_to_default_queue_partition_and_timeout() {
        let payload = request_payload(&[], None);
        let request = build_slurm_execution_request(payload, "general", "cpu-standard");
        assert_eq!(request.scheduler.queue, "general");
        assert_eq!(request.scheduler.partition, "cpu-standard");
        assert_eq!(request.scheduler.walltime, "00:01:00");
    }

    #[test]
    fn status_mapping_is_explicit_for_terminal_slurm_states() {
        assert_eq!(
            map_slurm_job_status_to_node_status(SlurmJobStatus::Completed),
            NodeStatus::Success
        );
        assert_eq!(
            map_slurm_job_status_to_node_status(SlurmJobStatus::Cancelled),
            NodeStatus::Cancelled
        );
        assert_eq!(
            map_slurm_job_status_to_node_status(SlurmJobStatus::Timeout),
            NodeStatus::Failed
        );
        assert_eq!(
            map_slurm_job_status_to_node_status(SlurmJobStatus::Preempted),
            NodeStatus::Failed
        );
    }

    #[test]
    fn terminal_status_is_inferred_from_timeout_failure_boundary() {
        let node_result = NodeResult {
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
        };
        assert_eq!(slurm_job_status_from_node_result(&node_result), SlurmJobStatus::Timeout);
    }
}
