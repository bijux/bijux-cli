use bijux_dag_runtime::{classify_failure, RuntimeFailureClass};

#[test]
fn policy_violation_is_mapped_to_policy_failure_class() {
    assert_eq!(
        classify_failure(false, false, false, true, false, false),
        RuntimeFailureClass::PolicyViolation
    );
}
