use crate::{
    transition_cause_for_failure, transition_cause_for_skip_reason, transition_cause_for_status,
    FailureInfo, NodeStatus, PolicyConfig, RuntimeConfig, Selector, SelectorSet,
};

#[test]
fn runtime_config_default_is_execution_independent() {
    let a = RuntimeConfig::default();
    let b = RuntimeConfig::default();
    assert_eq!(a.jobs, b.jobs);
    assert_eq!(a.cache_dir, b.cache_dir);
    assert_eq!(a.policy.clean_env, b.policy.clean_env);
}

#[test]
fn selector_set_is_deterministic_value_model() {
    let set = SelectorSet {
        include: vec![Selector::Kind("shell".to_string()), Selector::Tag("etl".to_string())],
        exclude: vec![Selector::IdPrefix("tmp_".to_string())],
    };
    let set2 = set.clone();
    assert_eq!(set.include.len(), set2.include.len());
    assert_eq!(set.exclude.len(), set2.exclude.len());
}

#[test]
fn policy_defaults_do_not_require_execution() {
    let policy = PolicyConfig::default();
    assert!(!policy.deny_network);
    assert!(!policy.deny_env);
    assert!(!policy.deny_clock);
    assert!(policy.clean_env);
}

#[test]
fn transition_cause_mapping_is_stable() {
    assert_eq!(transition_cause_for_status(&NodeStatus::Success), "ExecutionSucceeded");
    assert_eq!(transition_cause_for_status(&NodeStatus::Failed), "ExecutionFailed");
    assert_eq!(transition_cause_for_status(&NodeStatus::Skipped), "SelectionFiltered");
    assert_eq!(transition_cause_for_status(&NodeStatus::Cached), "CachedReuse");
    assert_eq!(
        transition_cause_for_failure(Some(&FailureInfo {
            class: Some(bijux_dag_artifacts::FailureClass::Policy),
            kind: "Policy".to_string(),
            code: "POLICY_DENIED".to_string(),
            message: "policy denied".to_string(),
            details: None,
        })),
        "PolicyDenied"
    );
    assert_eq!(
        transition_cause_for_failure(Some(&FailureInfo {
            class: Some(bijux_dag_artifacts::FailureClass::Timeout),
            kind: "Execution".to_string(),
            code: "EXEC_TIMEOUT".to_string(),
            message: "timed out".to_string(),
            details: None,
        })),
        "TimeoutExceeded"
    );
    assert_eq!(
        transition_cause_for_failure(Some(&FailureInfo {
            class: Some(bijux_dag_artifacts::FailureClass::Infrastructure),
            kind: "Infrastructure".to_string(),
            code: "CONTAINER_ENGINE_UNAVAILABLE".to_string(),
            message: "missing engine".to_string(),
            details: None,
        })),
        "InfrastructureFailed"
    );
    assert_eq!(
        transition_cause_for_failure(Some(&FailureInfo {
            class: Some(bijux_dag_artifacts::FailureClass::User),
            kind: "Execution".to_string(),
            code: "OUTPUT_MISSING".to_string(),
            message: "missing output".to_string(),
            details: None,
        })),
        "MissingRequiredOutput"
    );
}

#[test]
fn skip_transition_causes_follow_recorded_skip_reason() {
    assert_eq!(transition_cause_for_skip_reason("filtered"), "SelectionFiltered");
    assert_eq!(
        transition_cause_for_skip_reason("not_selected_by_include_selector"),
        "SelectionFiltered"
    );
    assert_eq!(transition_cause_for_skip_reason("excluded_by_selector"), "SelectionFiltered");
    assert_eq!(transition_cause_for_skip_reason("upstream_failed"), "DependencyFailed");
    assert_eq!(transition_cause_for_skip_reason("cancelled"), "CancelRequested");
}
