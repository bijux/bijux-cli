use bijux_dag_artifacts as _;
use bijux_dag_core::{parse_graph_strict, Graph};
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2::{Digest, Sha256};
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    build_slurm_execution_request, AbsolutePathPolicy, MockSlurmBackend, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteInputArtifact, RemoteNodeExecutionPayload, SlurmBackendExecutor, SlurmJobStatus,
};
use std::fs;
use std::path::Path;

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    format!("{:x}", hasher.finalize())
}

fn shell_payload(
    backend_id: &str,
    out_base: &Path,
    run_id: &str,
    script: &str,
    timeout_ms: Option<u64>,
) -> RemoteNodeExecutionPayload {
    let graph: Graph = parse_graph_strict(&format!(
        r#"{{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {{
              "id": "shell-node",
              "kind": "shell",
              "outputs": [{{"name": "result", "path": "result.txt"}}],
              "params": {{"argv": ["/bin/sh", "-c", {script:?}]}},
              "resources": {{"cpu": 4, "mem_mb": 8192}},
              "tags": ["slurm.partition:gpu", "slurm.queue:priority", "slurm.account:research"]
            }}
          ],
          "edges": []
        }}"#
    ))
    .expect("parse graph");
    let mut node = graph.nodes[0].clone();
    node.timeout_ms = timeout_ms;
    RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: run_id.to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: backend_id.to_string(),
        },
        graph,
        node,
        params: json!({"argv": ["/bin/sh", "-c", script]}),
        input_artifacts: vec![RemoteInputArtifact {
            relative_path: "seed/in.txt".to_string(),
            sha256: sha256_hex(b"slurm-input"),
            bytes: b"slurm-input".to_vec(),
        }],
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

#[test]
fn mock_slurm_backend_executes_shell_payload_and_captures_logs() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = shell_payload(
        "slurm",
        temp.path(),
        "slurm-shell-success",
        "cat ../inputs/seed/in.txt > ../outputs/result.txt && echo slurm-stdout && echo slurm-stderr 1>&2",
        Some(120_000),
    );
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let backend = MockSlurmBackend::default();

    let result = backend.execute_job(request).expect("slurm execute");

    assert_eq!(result.scheduler_status, SlurmJobStatus::Completed);
    assert_eq!(result.node_status, NodeStatus::Success);
    assert_eq!(result.job.lifecycle.len(), 3);
    assert_eq!(result.job.lifecycle[0].status, SlurmJobStatus::Submitted);
    assert_eq!(result.job.lifecycle[1].status, SlurmJobStatus::Running);
    assert_eq!(result.job.lifecycle[2].status, SlurmJobStatus::Completed);
    assert!(result.job.job_id.starts_with("slurm-"));
    assert!(result.job.metadata.resource_request.contains("queue=priority"));
    assert!(result.job.metadata.resource_request.contains("partition=gpu"));
    assert!(result.job.metadata.resource_request.contains("account=research"));
    assert!(result.logs.stdout.contains("slurm-stdout"));
    assert!(result.logs.stderr.contains("slurm-stderr"));

    let rendered =
        fs::read_to_string(Path::new(&result.node_result.outputs_dir).join("result.txt"))
            .expect("rendered output");
    assert_eq!(rendered, "slurm-input");

    let persisted = backend.job_record(&result.job.job_id).expect("job record");
    assert_eq!(persisted.terminal_status, SlurmJobStatus::Completed);
    assert_eq!(backend.submitted_requests().len(), 1);
}

#[test]
fn mock_slurm_backend_marks_failed_shell_payload_as_failed_job() {
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = shell_payload(
        "hpc",
        temp.path(),
        "slurm-shell-failure",
        "echo broken 1>&2; exit 17",
        Some(120_000),
    );
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let backend = MockSlurmBackend::default();

    let result = backend.execute_job(request).expect("slurm execute");

    assert_eq!(result.scheduler_status, SlurmJobStatus::Failed);
    assert_eq!(result.node_status, NodeStatus::Failed);
    assert_eq!(result.node_result.status, NodeStatus::Failed);
    assert!(result.logs.stderr.contains("broken"));
    assert_eq!(result.job.terminal_status, SlurmJobStatus::Failed);
}
