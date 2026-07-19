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
    build_kubernetes_execution_request, map_kubernetes_pod_status_to_node_status,
    validate_kubernetes_execution_request, AbsolutePathPolicy, KubernetesPodPhase,
    KubernetesPodStatus, KubernetesWorkloadKind, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteNodeExecutionPayload,
};

fn payload_with_backend(backend_id: &str) -> RemoteNodeExecutionPayload {
    let graph: Graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "container-node",
              "kind": "container",
              "inputs": ["seed"],
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {},
              "resources": {"cpu": 4, "mem_mb": 2048, "gpu_devices": 1},
              "retry": {"max_attempts": 2, "backoff_ms": 5000},
              "container": {
                "image": "example.local/runner@sha256:feedface",
                "argv": ["/bin/sh", "-c", "printf value > /bijux/node/outputs/value.txt"],
                "engine": "docker"
              }
            }
          ],
          "edges": []
        }"#,
    )
    .expect("parse graph");
    let node = graph.nodes[0].clone();
    let params: Value = json!({"timeout_ms": 125000});
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
            out_base: "/tmp/kubernetes-contracts".to_string(),
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
fn kubernetes_request_builder_maps_node_contract_into_job_request() {
    let request = build_kubernetes_execution_request(payload_with_backend("kubernetes"), "bijux");
    assert_eq!(request.namespace, "bijux");
    assert_eq!(request.resources.requests.cpu_millis, 4000);
    assert_eq!(request.resources.requests.memory_mib, 2048);
    assert_eq!(request.resources.limits.cpu_millis, 8000);
    assert_eq!(request.resources.limits.memory_mib, 3072);
    assert_eq!(request.policy.active_deadline_seconds, 125);
    assert_eq!(request.policy.backoff_limit, 2);
    assert_eq!(request.policy.retry_backoff_seconds, 5);
    assert_eq!(request.workspace.input_artifact_count, 1);
    assert_eq!(request.workspace.declared_output_count, 1);
    assert_eq!(request.workload.kind, KubernetesWorkloadKind::ContainerNode);
    validate_kubernetes_execution_request(&request).expect("valid kubernetes request");
}

#[test]
fn kubernetes_request_validation_rejects_non_kubernetes_backend_identity() {
    let request =
        build_kubernetes_execution_request(payload_with_backend("remote-worker"), "bijux");
    let error =
        validate_kubernetes_execution_request(&request).expect_err("invalid backend identity");
    assert!(error.contains("backend_id"));
}

#[test]
fn kubernetes_status_mapping_preserves_success_and_cancelled_boundaries() {
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
        map_kubernetes_pod_status_to_node_status(&KubernetesPodStatus {
            phase: KubernetesPodPhase::Running,
            reason: None,
        }),
        NodeStatus::Failed
    );
}
