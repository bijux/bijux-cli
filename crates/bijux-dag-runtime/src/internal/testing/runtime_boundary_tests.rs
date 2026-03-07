#![cfg(test)]

use crate::{PolicyConfig, RuntimeConfig, Selector, SelectorSet, transition_cause_for_status, NodeStatus};

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
}
