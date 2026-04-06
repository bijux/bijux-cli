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

use bijux_dag_runtime::{execution, execution_context, node_result, run_context, NodeStatus};

#[test]
fn execution_facade_exports_local_executor_surface() {
    let mut exec = execution::LocalExecutor::new(2);
    exec.submit("a".to_string()).expect("submit a");
    exec.submit("b".to_string()).expect("submit b");
    assert_eq!(exec.queue_depth(), 2);
    assert_eq!(exec.start_next().as_deref(), Some("a"));
    exec.mark_finished();
}

#[test]
fn execution_context_aliases_match_runtime_context_types() {
    assert_eq!(
        std::mem::size_of::<execution_context::ExecutionContext>(),
        std::mem::size_of::<run_context::RunContext>()
    );
    assert_eq!(
        std::any::type_name::<execution_context::NodeExecutionContext<'static>>(),
        std::any::type_name::<execution_context::NodeCtx<'static>>()
    );
}

#[test]
fn node_result_surface_exports_runtime_node_status() {
    let status = node_result::NodeStatus::Cached;
    assert!(matches!(status, NodeStatus::Cached));
    let _status_alias: node_result::NodeStatus = status;
}
