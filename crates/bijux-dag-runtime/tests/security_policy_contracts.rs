use bijux_dag_core::Effect;
use bijux_dag_runtime::{
    leak_conformance_check, policy_allows_effects, redact_secret_payload, PolicyConfig,
};

#[test]
fn deny_flags_block_declared_effects() {
    let policy = PolicyConfig {
        deny_network: true,
        deny_env: true,
        deny_clock: true,
        clean_env: true,
    };
    assert!(!policy_allows_effects(&policy, &[Effect::Network]));
    assert!(!policy_allows_effects(&policy, &[Effect::Env]));
    assert!(!policy_allows_effects(&policy, &[Effect::Clock]));
    assert!(policy_allows_effects(&policy, &[Effect::Filesystem]));
}

#[test]
fn redaction_and_leak_detection_cover_common_secret_tokens() {
    let payload = "token=abc password=xyz secret=hidden";
    let redacted = redact_secret_payload(payload, &["abc".to_string(), "xyz".to_string(), "hidden".to_string()]);
    assert!(!redacted.contains("abc"));
    assert!(!redacted.contains("xyz"));
    assert!(!redacted.contains("hidden"));
    assert!(!leak_conformance_check(&[payload.to_string()]));
    assert!(leak_conformance_check(&["status=ok".to_string()]));
}
