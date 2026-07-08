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
    default_runtime_config, policy_evaluation_trace, resolve_effective_config,
    PartialRuntimeSurfaceConfig, PolicySurfaceConfig,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::RequireDigest,
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
                container_image_reference_policy:
                    bijux_dag_runtime::ContainerImageReferencePolicy::AllowUnpinned,
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
    assert!(strict_trace
        .iter()
        .any(|e| e.contains("rule:container_image_reference decision:require_digest")));
    assert!(permissive_trace
        .iter()
        .any(|e| e.contains("rule:container_image_reference decision:allow_unpinned")));
}
