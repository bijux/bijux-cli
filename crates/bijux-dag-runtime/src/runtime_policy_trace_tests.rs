#![cfg(test)]

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
    };
    assert!(!policy_allows_effects(&policy, &[Effect::Network]));
    assert!(!policy_allows_effects(&policy, &[Effect::Clock]));
    assert!(policy_allows_effects(&policy, &[Effect::Filesystem]));
}

#[test]
fn trace_label_mapping_is_stable_without_filesystem() {
    assert_eq!(trace_status_label(&NodeStatus::Success), "success");
    assert_eq!(trace_status_label(&NodeStatus::Failed), "failed");
    assert_eq!(trace_status_label(&NodeStatus::Skipped), "skipped");
    assert_eq!(trace_status_label(&NodeStatus::Cached), "cached");
}
