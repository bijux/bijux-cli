use bijux_dag_artifacts as _;
use bijux_dag_core::{parse_graph_strict, Graph};
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json::json;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    build_slurm_execution_request, AbsolutePathPolicy, MockSlurmBackend, NodeStatus, PolicyConfig,
    RemoteExecutionFingerprintSet, RemoteExecutionIdentity, RemoteExecutionWorkspace,
    RemoteNodeExecutionPayload, SlurmBackendExecutor, SlurmJobStatus,
};

#[test]
fn runtime_facade_exports_modeled_slurm_backend_surface() {
    let graph: Graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {
              "id": "const-node",
              "kind": "const",
              "outputs": [{"name": "value", "path": "value.txt"}],
              "params": {"value": "hello"},
              "resources": {"cpu": 2, "mem_mb": 2048},
              "tags": ["slurm.partition:cpu", "slurm.queue:general"]
            }
          ],
          "edges": []
        }"#,
    )
    .expect("parse graph");
    let node = graph.nodes[0].clone();
    let temp = tempfile::tempdir().expect("temp dir");
    let payload = RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: "slurm-facade".to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: "slurm".to_string(),
        },
        graph,
        node,
        params: json!({"value": "hello"}),
        input_artifacts: Vec::new(),
        workspace: RemoteExecutionWorkspace {
            out_base: temp.path().display().to_string(),
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
            command_fingerprint: None,
            execution_fingerprint: "execution".to_string(),
            evidence_fingerprint: "evidence".to_string(),
            execution_contract_fingerprint: "contract".to_string(),
        },
    };
    let request = build_slurm_execution_request(payload, "general", "cpu");
    let result = MockSlurmBackend::default().execute_job(request).expect("slurm execute");

    assert!(result.job.job_id.starts_with("slurm-"));
    assert_eq!(result.scheduler_status, SlurmJobStatus::Completed);
    assert_eq!(result.node_status, NodeStatus::Success);
}
