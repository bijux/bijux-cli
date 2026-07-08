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
    serialize_node_result_payload, AbsolutePathPolicy, ExecutionContext, LocalExecutor,
    LocalWorkerExecution, LocalWorkerPool, MockRemoteWorker, NodeExecutionContext, NodeResult,
    NodeStatus, PolicyConfig, RemoteExecutionFingerprintSet, RemoteExecutionIdentity,
    RemoteExecutionWorkspace, RemoteNodeExecutionPayload, RemoteWorkerExecutor, RunContext,
};
use serde_json::json;

#[test]
fn execution_facade_exports_local_executor_surface() {
    let mut exec = LocalExecutor::new(2);
    exec.submit("a".to_string()).expect("submit a");
    exec.submit("b".to_string()).expect("submit b");
    assert_eq!(exec.queue_depth(), 2);
    assert_eq!(exec.start_next().as_deref(), Some("a"));
    exec.mark_finished();
}

#[test]
fn execution_facade_exports_local_worker_pool_surface() {
    let mut pool = LocalWorkerPool::<&'static str>::new(1);
    pool.submit(
        "alpha".to_string(),
        Box::new(|| LocalWorkerExecution { started_unix_ms: 1, finished_unix_ms: 2, result: "ok" }),
    )
    .expect("submit alpha");
    let completion = pool.wait_for_completion().expect("completion");
    assert_eq!(completion.node_id, "alpha");
    assert_eq!(pool.available_workers(), 1);
}

#[test]
fn execution_context_aliases_match_runtime_context_types() {
    assert_eq!(std::mem::size_of::<ExecutionContext>(), std::mem::size_of::<RunContext>());
    assert!(std::mem::size_of::<Option<NodeExecutionContext<'static>>>() > 0);
}

#[test]
fn node_result_surface_exports_runtime_node_status() {
    let status = NodeStatus::Cached;
    assert!(matches!(status, NodeStatus::Cached));
    let status_alias: NodeStatus = status;
    assert!(matches!(status_alias, NodeStatus::Cached));
    let _ = std::mem::size_of::<NodeResult>();
}

#[test]
fn execution_facade_exports_remote_worker_payload_surface() {
    let temp = tempfile::tempdir().expect("temp dir");
    let graph = parse_graph_strict(
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
    )
    .expect("graph");
    let node = graph.nodes[0].clone();
    let payload = RemoteNodeExecutionPayload {
        identity: RemoteExecutionIdentity {
            run_id: "entrypoint-run".to_string(),
            node_id: node.id.clone(),
            attempt_id: "1".to_string(),
            backend_id: "remote-worker".to_string(),
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
            node_fingerprint: "node-fp".to_string(),
            node_definition_fingerprint: "node-def-fp".to_string(),
            declared_environment_fingerprint: "env-fp".to_string(),
            params_fingerprint: "params-fp".to_string(),
            command_fingerprint: Some("command-fp".to_string()),
            execution_fingerprint: "execution-fp".to_string(),
            evidence_fingerprint: "evidence-fp".to_string(),
            execution_contract_fingerprint: "execution-contract-fp".to_string(),
        },
    };

    let result = MockRemoteWorker.execute_payload(payload).expect("execute remote payload");
    assert_eq!(result.node_result.status, NodeStatus::Success);
    let serialized = serialize_node_result_payload(&result.node_result).expect("serialize result");
    assert!(serialized["outputs_dir"].is_string());
}
