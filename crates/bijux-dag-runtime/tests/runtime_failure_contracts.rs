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

use bijux_dag_runtime::{classify_failure, RuntimeFailureClass};

#[test]
fn failure_paths_are_classified_explicitly() {
    assert_eq!(classify_failure(true, false, false, false, false, false), RuntimeFailureClass::Timeout);
    assert_eq!(classify_failure(false, true, false, false, false, false), RuntimeFailureClass::Cancelled);
    assert_eq!(classify_failure(false, false, true, false, false, false), RuntimeFailureClass::DependencyFailure);
}
