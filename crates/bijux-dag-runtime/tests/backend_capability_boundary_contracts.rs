use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    hpc_environment_fingerprint, hpc_resource_fingerprint,
    reject_unsupported_hpc_scheduler_features, reject_unsupported_k8s_fields,
    replay_allowed_across_backends, CrossBackendReplayRule, HpcResourceFingerprintInput,
};
use std::collections::BTreeMap;

#[test]
fn unsupported_features_are_rejected_instead_of_downgraded() {
    let k8s = reject_unsupported_k8s_fields(&["hostNetwork".to_string(), "safe-field".to_string()]);
    assert!(k8s.is_err());

    let hpc = reject_unsupported_hpc_scheduler_features(&[
        "host-network".to_string(),
        "safe-feature".to_string(),
    ]);
    assert!(hpc.is_err());
}

#[test]
fn backend_capability_declarations_support_replay_gate_checks() {
    let rules = vec![
        CrossBackendReplayRule {
            from_backend: "local".to_string(),
            to_backend: "local".to_string(),
            replay_safe: true,
            reason: "same backend".to_string(),
        },
        CrossBackendReplayRule {
            from_backend: "local".to_string(),
            to_backend: "kubernetes".to_string(),
            replay_safe: false,
            reason: "environment mismatch".to_string(),
        },
    ];
    assert!(replay_allowed_across_backends("local", "local", &rules));
    assert!(!replay_allowed_across_backends("local", "kubernetes", &rules));
}

#[test]
fn cross_node_compatibility_fingerprints_are_stable_for_same_inputs() {
    let modules = vec!["python/3.11".to_string(), "cuda/12.2".to_string()];
    let env = BTreeMap::from([
        ("OMP_NUM_THREADS".to_string(), "8".to_string()),
        ("TZ".to_string(), "UTC".to_string()),
    ]);
    let first_env_fp = hpc_environment_fingerprint(&modules, &env);
    let second_env_fp = hpc_environment_fingerprint(&modules, &env);
    assert_eq!(first_env_fp, second_env_fp);

    let input = HpcResourceFingerprintInput {
        queue: "cpu".to_string(),
        partition: "general".to_string(),
        account: "research".to_string(),
    };
    let first_resource_fp = hpc_resource_fingerprint(&input);
    let second_resource_fp = hpc_resource_fingerprint(&input);
    assert_eq!(first_resource_fp, second_resource_fp);
}
