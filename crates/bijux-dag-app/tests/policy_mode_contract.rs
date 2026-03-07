use bijux_dag_app::{
    default_runtime_config, policy_evaluation_trace, PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
    resolve_effective_config,
};

#[test]
fn strict_and_permissive_modes_differ_in_effect_policy() {
    let strict = resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            policy: Some(PolicySurfaceConfig {
                deny_network: true,
                deny_env: true,
                deny_clock: true,
                clean_env: true,
                allowed_env: vec!["PATH".into()],
            }),
            ..PartialRuntimeSurfaceConfig::default()
        },
        None,
        None,
        default_runtime_config(),
    );

    let permissive = resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            policy: Some(PolicySurfaceConfig {
                deny_network: false,
                deny_env: false,
                deny_clock: false,
                clean_env: false,
                allowed_env: vec!["PATH".into(), "HOME".into()],
            }),
            ..PartialRuntimeSurfaceConfig::default()
        },
        None,
        None,
        default_runtime_config(),
    );

    assert!(strict.policy.deny_network);
    assert!(!permissive.policy.deny_network);
    assert_ne!(strict.policy.clean_env, permissive.policy.clean_env);

    let strict_trace = policy_evaluation_trace(&strict.policy);
    let permissive_trace = policy_evaluation_trace(&permissive.policy);
    assert!(strict_trace.iter().any(|e| e.contains("rule:deny_network decision:deny")));
    assert!(permissive_trace.iter().any(|e| e.contains("rule:deny_network decision:allow")));
}
