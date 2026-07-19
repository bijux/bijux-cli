use crate::policy::policy_allows_effects;
use crate::trace::trace_status_label;
use crate::{NodeStatus, PolicyConfig};
use bijux_dag_core::Effect;

#[test]
fn policy_evaluation_works_without_node_execution() {
    let policy = PolicyConfig {
        deny_network: true,
        deny_env: false,
        deny_clock: true,
        clean_env: true,
        ..PolicyConfig::default()
    };
    assert!(!policy_allows_effects(&policy, &[Effect::Network]));
    assert!(!policy_allows_effects(&policy, &[Effect::Clock]));
    assert!(policy_allows_effects(&policy, &[Effect::Filesystem]));
}

#[test]
fn deny_network_policy_is_consistent_for_shell_and_container_effects() {
    let policy = PolicyConfig {
        deny_network: true,
        deny_env: false,
        deny_clock: false,
        clean_env: true,
        ..PolicyConfig::default()
    };
    assert!(!policy_allows_effects(&policy, &[Effect::Network]));
    assert!(!policy_allows_effects(&policy, &[Effect::Filesystem, Effect::Network]));
}

#[test]
fn clean_env_and_deny_env_interaction_is_deterministic() {
    let strict = PolicyConfig {
        deny_network: false,
        deny_env: true,
        deny_clock: false,
        clean_env: true,
        ..PolicyConfig::default()
    };
    assert!(!policy_allows_effects(&strict, &[Effect::Env]));
    assert!(policy_allows_effects(&strict, &[Effect::Filesystem]));
}

#[test]
fn trace_label_mapping_is_stable_without_filesystem() {
    assert_eq!(trace_status_label(&NodeStatus::Success), "success");
    assert_eq!(trace_status_label(&NodeStatus::Failed), "failed");
    assert_eq!(trace_status_label(&NodeStatus::Skipped), "skipped");
    assert_eq!(trace_status_label(&NodeStatus::Cached), "cached");
}
