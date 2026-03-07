use bijux_dag_runtime::{dependency_resolution_is_complete, retry_allowed, RetryPolicySemantics};
use std::collections::BTreeSet;

#[test]
fn node_execution_requires_dependencies_and_retry_budget() {
    let retry = RetryPolicySemantics { max_attempts: 2, initial_backoff_ms: 10, exponential: false };
    assert!(retry_allowed(1, &retry));
    assert!(!retry_allowed(2, &retry));

    let resolved = BTreeSet::from(["extract".to_string(), "transform".to_string()]);
    assert!(dependency_resolution_is_complete(&["extract".to_string(), "transform".to_string()], &resolved));
}
