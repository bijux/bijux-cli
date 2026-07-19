use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_app::{
    config_fingerprint, default_runtime_config, normalize_runtime_config, resolve_effective_config,
    CacheModeSurface, PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::AllowUnpinned,
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
