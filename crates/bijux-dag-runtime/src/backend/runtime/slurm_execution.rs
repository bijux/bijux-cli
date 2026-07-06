use crate::backend_cluster::{
    map_node_to_hpc_queue_partition, map_timeout_to_hpc_walltime, HpcNodeExecutionContract,
};
use crate::remote_execution_model::{
    validate_remote_execution_payload, RemoteNodeExecutionPayload,
};
use crate::NodeStatus;
use bijux_dag_core::Node;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const DEFAULT_CPU_CORES: u32 = 1;
const DEFAULT_MEMORY_MIB: u32 = 256;
const DEFAULT_TIMEOUT_SECONDS: u32 = 60;

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
}
