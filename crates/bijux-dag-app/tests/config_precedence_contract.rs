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
    default_runtime_config, resolve_effective_config, CacheModeSurface, MaterializeInputsSurface,
    PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
};

#[test]
fn precedence_is_cli_then_explicit_then_env_then_defaults() {
    let defaults = default_runtime_config();
    let env_cfg = PartialRuntimeSurfaceConfig {
        jobs: Some(2),
        cache_mode: Some(CacheModeSurface::Read),
        ..PartialRuntimeSurfaceConfig::default()
    };
    let explicit = PartialRuntimeSurfaceConfig {
        jobs: Some(4),
        materialize_inputs: Some(MaterializeInputsSurface::Direct),
        ..PartialRuntimeSurfaceConfig::default()
    };
    let cli =
        PartialRuntimeSurfaceConfig { jobs: Some(8), ..PartialRuntimeSurfaceConfig::default() };

    let effective = resolve_effective_config(cli, Some(explicit), Some(env_cfg), defaults);
    assert_eq!(effective.jobs, 8);
    assert_eq!(effective.cache_mode, CacheModeSurface::Read);
    assert_eq!(effective.materialize_inputs, MaterializeInputsSurface::Direct);
}

#[test]
fn policy_modes_are_observably_different() {
    let defaults = default_runtime_config();
    let strict = PartialRuntimeSurfaceConfig {
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
    };
    let permissive = PartialRuntimeSurfaceConfig {
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
    };

    let strict_effective = resolve_effective_config(strict, None, None, defaults.clone());
    let permissive_effective = resolve_effective_config(permissive, None, None, defaults);

    assert_ne!(strict_effective.policy, permissive_effective.policy);
}
