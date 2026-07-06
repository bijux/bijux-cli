use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    ExecutionContext, LocalExecutor, LocalWorkerExecution, LocalWorkerPool, NodeExecutionContext,
    NodeResult, NodeStatus, RunContext,
};

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
