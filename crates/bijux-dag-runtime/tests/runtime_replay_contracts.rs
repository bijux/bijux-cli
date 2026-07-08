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

use bijux_dag_runtime::replay_equivalent;

#[test]
fn replay_mismatch_is_detected() {
    assert!(!replay_equivalent("fingerprint-a", "fingerprint-b"));
}
