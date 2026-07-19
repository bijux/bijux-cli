use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    execution_mode_status, remote_handoff_valid, serialize_node_result_payload,
    validate_remote_execution_payload, validate_remote_identity, AbsolutePathPolicy,
    ExecutionModeStatus, MockRemoteWorker, NodeStatus, PolicyConfig, RemoteArtifactHandoff,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteInputArtifact, RemoteNodeExecutionPayload, RemoteObservabilityHandoff,
    RemoteWorkerExecutor,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn payload_for(graph_json: &str, run_id: &str, out_base: &Path) -> RemoteNodeExecutionPayload {
    let graph = parse_graph_strict(graph_json).expect("parse graph");
    let node = graph.nodes[0].clone();
    let params = serde_json::to_value(node.params.clone()).expect("serialize params");
    RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: run_id.to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: "remote-worker".to_string(),
        },
        graph,
        node,
        params,
        input_artifacts: Vec::new(),
        workspace: RemoteExecutionWorkspace {
            out_base: out_base.display().to_string(),
            cache_dir: None,
        },
        policy: PolicyConfig::default(),
        absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
        planner_contract_version: "bijux-dag-planner/v1".to_string(),
        fingerprints: RemoteExecutionFingerprintSet {
            node_fingerprint: "node-fp".to_string(),
            node_definition_fingerprint: "node-def-fp".to_string(),
            declared_environment_fingerprint: "env-fp".to_string(),
            params_fingerprint: "params-fp".to_string(),
            command_fingerprint: Some("command-fp".to_string()),
            execution_fingerprint: "execution-fp".to_string(),
            evidence_fingerprint: "evidence-fp".to_string(),
            execution_contract_fingerprint: "execution-contract-fp".to_string(),
        },
    }
}

#[test]
fn remote_identity_requires_run_node_attempt_backend_fields() {
    let identity = RemoteExecutionIdentity {
        run_id: "run-1".to_string(),
        node_id: "node-a".to_string(),
        attempt_id: "1".to_string(),
        backend_id: "remote-sim".to_string(),
    };
    assert!(validate_remote_identity(&identity).is_ok());

    let missing_backend = RemoteExecutionIdentity { backend_id: String::new(), ..identity };
    assert!(validate_remote_identity(&missing_backend).is_err());
}

#[test]
fn remote_handoff_requires_artifact_and_observability_fields() {
    let artifact = RemoteArtifactHandoff {
        upload_endpoint: "s3://bucket/upload".to_string(),
        download_endpoint: "s3://bucket/download".to_string(),
        integrity_required: true,
    };
    let observability = RemoteObservabilityHandoff {
        stream_mode: "line-buffered".to_string(),
        trace_forwarding: true,
        retention_days_hint: 14,
    };
    assert!(remote_handoff_valid(&artifact, &observability));
}

#[test]
fn remote_execution_payload_rejects_bad_input_digest_and_mismatched_identity() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut payload = payload_for(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "const-node",
              "kind": "const",
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"value": "hello"}
            }
          ],
          "edges": []
        }"#,
        "payload-invalid",
        temp.path(),
    );
    let bytes = b"seed-data".to_vec();
    payload.input_artifacts.push(RemoteInputArtifact {
        relative_path: "seed/in.txt".to_string(),
        sha256: "wrong".to_string(),
        bytes,
    });
    assert!(validate_remote_execution_payload(&payload).is_err());

    payload.input_artifacts[0].sha256 = sha256_hex(&payload.input_artifacts[0].bytes);
    payload.identity.node_id = "other-node".to_string();
    assert!(validate_remote_execution_payload(&payload).is_err());
}

#[test]
fn mock_remote_worker_executes_const_payload_with_shared_node_result_shape() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = payload_for(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "const-node",
              "kind": "const",
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"value": "hello"}
            }
          ],
          "edges": []
        }"#,
        "remote-const",
        temp.path(),
    );

    let result = MockRemoteWorker.execute_payload(payload).expect("remote const execute");
    assert_eq!(result.identity.backend_id, "remote-worker");
    assert_eq!(result.node_result.status, NodeStatus::Success);

    let serialized =
        serialize_node_result_payload(&result.node_result).expect("serialize node result");
    assert!(serialized["stdout_path"].is_string());
    assert!(serialized["stderr_path"].is_string());
    assert!(serialized["outputs_dir"].is_string());
    assert!(serialized["output_evidence"].is_array());
    assert_eq!(serialized["attempts"], json!(1));
}

#[test]
fn mock_remote_worker_materializes_inputs_for_shell_payloads() {
    let temp = tempfile::tempdir().expect("temp dir");
    let mut payload = payload_for(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "shell-node",
              "kind": "shell",
              "outputs": [{"name": "value", "path": "result.txt"}],
              "params": {
                "argv": [
                  "/bin/sh",
                  "-c",
                  "cat ../inputs/seed/in.txt > ../outputs/result.txt"
                ]
              },
              "effects": ["filesystem"]
            }
          ],
          "edges": []
        }"#,
        "remote-shell",
        temp.path(),
    );
    let bytes = b"remote-input".to_vec();
    payload.input_artifacts.push(RemoteInputArtifact {
        relative_path: "seed/in.txt".to_string(),
        sha256: sha256_hex(&bytes),
        bytes,
    });

    let result = MockRemoteWorker.execute_payload(payload).expect("remote shell execute");
    assert_eq!(result.node_result.status, NodeStatus::Success);
    let rendered =
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("result.txt"))
            .expect("rendered output");
    assert_eq!(rendered, "remote-input");
}

#[test]
fn execution_mode_status_is_explicit_for_container_and_kubernetes() {
    assert_eq!(execution_mode_status("local"), ExecutionModeStatus::Implemented);
    assert_eq!(execution_mode_status("container"), ExecutionModeStatus::Simulated);
    assert_eq!(execution_mode_status("remote-worker"), ExecutionModeStatus::Simulated);
    assert_eq!(execution_mode_status("kubernetes"), ExecutionModeStatus::Implemented);
    assert_eq!(execution_mode_status("hpc"), ExecutionModeStatus::Simulated);
    assert_eq!(execution_mode_status("slurm"), ExecutionModeStatus::Implemented);
}
