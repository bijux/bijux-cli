use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::cancellation_is_terminal;

#[test]
fn cancellation_requires_terminal_node_state() {
    assert!(cancellation_is_terminal(true, true));
    assert!(!cancellation_is_terminal(true, false));
}
