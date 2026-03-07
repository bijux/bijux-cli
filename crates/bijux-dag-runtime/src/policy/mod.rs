//! Runtime policy configuration.
pub(crate) mod evaluator;
pub(crate) mod trace;

pub use crate::PolicyConfig;
use bijux_dag_core::Effect;

pub fn policy_allows_effects(policy: &PolicyConfig, effects: &[Effect]) -> bool {
    for effect in effects {
        match effect {
            Effect::Network if policy.deny_network => return false,
            Effect::Env if policy.deny_env => return false,
            Effect::Clock if policy.deny_clock => return false,
            _ => {}
        }
    }
    true
}
