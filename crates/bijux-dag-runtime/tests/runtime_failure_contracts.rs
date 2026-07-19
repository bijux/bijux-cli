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
fn failure_paths_are_classified_explicitly() {
    assert_eq!(
        classify_failure(true, false, false, false, false, false),
        RuntimeFailureClass::Timeout
    );
    assert_eq!(
        classify_failure(false, true, false, false, false, false),
        RuntimeFailureClass::Cancelled
    );
    assert_eq!(
        classify_failure(false, false, true, false, false, false),
        RuntimeFailureClass::DependencyFailure
    );
}

#[test]
fn failure_classification_matrix_covers_policy_cache_artifact_and_adapter() {
    assert_eq!(
        classify_failure(false, false, false, true, false, false),
        RuntimeFailureClass::PolicyViolation
    );
    assert_eq!(
        classify_failure(false, false, false, false, true, false),
        RuntimeFailureClass::CacheInvalid
    );
    assert_eq!(
        classify_failure(false, false, false, false, false, true),
        RuntimeFailureClass::ArtifactCorruption
    );
    assert_eq!(
        classify_failure(false, false, false, false, false, false),
        RuntimeFailureClass::AdapterFailure
    );
}

fn operational_group(class: RuntimeFailureClass) -> &'static str {
    match class {
        RuntimeFailureClass::Timeout | RuntimeFailureClass::AdapterFailure => "transient",
        RuntimeFailureClass::PolicyViolation | RuntimeFailureClass::ArtifactCorruption => {
            "permanent"
        }
        RuntimeFailureClass::Cancelled
        | RuntimeFailureClass::DependencyFailure
        | RuntimeFailureClass::CacheInvalid => "conditional",
    }
}

#[test]
fn failure_taxonomy_transient_and_permanent_mapping_is_explicit() {
    assert_eq!(operational_group(RuntimeFailureClass::Timeout), "transient");
    assert_eq!(operational_group(RuntimeFailureClass::AdapterFailure), "transient");
    assert_eq!(operational_group(RuntimeFailureClass::PolicyViolation), "permanent");
    assert_eq!(operational_group(RuntimeFailureClass::ArtifactCorruption), "permanent");
}
