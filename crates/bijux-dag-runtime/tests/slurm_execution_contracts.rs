use bijux_dag_artifacts as _;
use bijux_dag_core::{parse_graph_strict, Graph};
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    build_slurm_execution_request, map_slurm_job_status_to_node_status,
    validate_slurm_execution_request, AbsolutePathPolicy, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteNodeExecutionPayload, SlurmJobStatus,
};

fn payload_with_backend(backend_id: &str) -> RemoteNodeExecutionPayload {
    let graph: Graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "shell-node",
              "kind": "shell",
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"argv": ["/bin/sh", "-c", "printf value > ../outputs/value.txt"]},
              "resources": {"cpu": 4, "mem_mb": 8192},
              "tags": ["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:research"]
            }
          ],
          "edges": []
        }"#,
    )
    .expect("parse graph");
    let node = graph.nodes[0].clone();
    let params: Value = json!({"argv": ["/bin/sh", "-c", "printf value > ../outputs/value.txt"], "timeout_ms": 125000});
    RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: "run-1".to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: backend_id.to_string(),
        },
        graph,
        node,
        params,
        input_artifacts: Vec::new(),
        workspace: RemoteExecutionWorkspace {
            out_base: "/tmp/slurm-contracts".to_string(),
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
fn slurm_request_builder_maps_node_contract_into_scheduler_request() {
    let request = build_slurm_execution_request(payload_with_backend("slurm"), "general", "cpu");
    assert_eq!(request.scheduler.cpu_cores, 4);
    assert_eq!(request.scheduler.memory_mib, 8192);
    assert_eq!(request.scheduler.queue, "priority");
    assert_eq!(request.scheduler.partition, "gpu");
    assert_eq!(request.scheduler.account.as_deref(), Some("research"));
    assert_eq!(request.scheduler.walltime, "00:02:05");
    validate_slurm_execution_request(&request).expect("valid slurm request");
}

#[test]
fn slurm_request_validation_rejects_non_slurm_backend_identity() {
    let request =
        build_slurm_execution_request(payload_with_backend("remote-worker"), "general", "cpu");
    let error = validate_slurm_execution_request(&request).expect_err("invalid backend identity");
    assert!(error.contains("backend_id"));
}

#[test]
fn slurm_status_mapping_preserves_success_and_cancelled_boundaries() {
    assert_eq!(map_slurm_job_status_to_node_status(SlurmJobStatus::Completed), NodeStatus::Success);
    assert_eq!(
        map_slurm_job_status_to_node_status(SlurmJobStatus::Cancelled),
        NodeStatus::Cancelled
    );
    assert_eq!(map_slurm_job_status_to_node_status(SlurmJobStatus::Running), NodeStatus::Failed);
}
