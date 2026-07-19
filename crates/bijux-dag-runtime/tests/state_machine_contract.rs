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

use bijux_dag_runtime::RuntimeState;

#[test]
fn state_machine_contract_surface_is_linkable() {
    let _ = std::mem::size_of::<RuntimeState>();
}
