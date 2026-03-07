use bijux_dag_runtime::{classify_failure, RuntimeFailureClass};

#[test]
fn failure_paths_are_classified_explicitly() {
    assert_eq!(classify_failure(true, false, false, false, false, false), RuntimeFailureClass::Timeout);
    assert_eq!(classify_failure(false, true, false, false, false, false), RuntimeFailureClass::Cancelled);
    assert_eq!(classify_failure(false, false, true, false, false, false), RuntimeFailureClass::DependencyFailure);
}
