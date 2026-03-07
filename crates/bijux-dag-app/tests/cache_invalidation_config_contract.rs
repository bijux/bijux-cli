use bijux_dag_app::{
    config_fingerprint, default_runtime_config, normalize_runtime_config, CacheModeSurface,
    PartialRuntimeSurfaceConfig, PolicySurfaceConfig, resolve_effective_config,
};

#[test]
fn semantically_equivalent_configs_have_same_fingerprint() {
    let defaults = default_runtime_config();

    let a = resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            cache_mode: Some(CacheModeSurface::ReadWrite),
            policy: Some(PolicySurfaceConfig {
                deny_network: true,
                deny_env: false,
                deny_clock: false,
                clean_env: true,
                allowed_env: vec!["path".into(), "HOME".into()],
            }),
            ..PartialRuntimeSurfaceConfig::default()
        },
        None,
        None,
        defaults.clone(),
    );

    let b = normalize_runtime_config(resolve_effective_config(
        PartialRuntimeSurfaceConfig {
            cache_mode: Some(CacheModeSurface::ReadWrite),
            policy: Some(PolicySurfaceConfig {
                deny_network: true,
                deny_env: false,
                deny_clock: false,
                clean_env: true,
                allowed_env: vec!["HOME".into(), "PATH".into(), "PATH".into()],
            }),
            ..PartialRuntimeSurfaceConfig::default()
        },
        None,
        None,
        defaults,
    ));

    assert_eq!(config_fingerprint(&a), config_fingerprint(&b));
}

#[test]
fn semantic_config_change_changes_fingerprint() {
    let defaults = default_runtime_config();
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
        defaults.clone(),
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
        defaults,
    );

    assert_ne!(config_fingerprint(&strict), config_fingerprint(&permissive));
}
