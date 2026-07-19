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

use bijux_dag_runtime::{classify_failure, RuntimeFailureClass};

#[test]
fn policy_violation_is_mapped_to_policy_failure_class() {
    assert_eq!(
        classify_failure(false, false, false, true, false, false),
        RuntimeFailureClass::PolicyViolation
    );
}
