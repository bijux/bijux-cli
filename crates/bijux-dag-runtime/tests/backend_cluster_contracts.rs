use bijux_dag_runtime::{
    backend_ready_for_admission, matches_placement_policy, normalize_backend_failure,
    quota_saturation_percent, replay_allowed_across_backends, BackendCapabilityDescriptor,
    BackendFailureMappingRule, BackendMaintenanceMode, BackendReadinessProbe,
    CrossBackendReplayRule,
};

#[test]
fn placement_and_failure_mapping_contracts_are_stable() {
    let descriptor = BackendCapabilityDescriptor {
        cpu_class: "cpu-standard".to_string(),
        memory_class: "mem-high".to_string(),
        gpu_class: Some("gpu-a100".to_string()),
        ephemeral_storage_class: "ephemeral-fast".to_string(),
        network_class: "net-private".to_string(),
    };
    assert!(matches_placement_policy("gpu-a100", &descriptor));
    assert!(!matches_placement_policy("gpu-h100", &descriptor));

    let rules = vec![
        BackendFailureMappingRule {
            backend_error_code: "K8S_NODE_PREEMPTED".to_string(),
            runtime_failure_kind: "infrastructure".to_string(),
            retryable: true,
        },
        BackendFailureMappingRule {
            backend_error_code: "SLURM_JOB_FAILED".to_string(),
            runtime_failure_kind: "execution".to_string(),
            retryable: false,
        },
    ];
    let mapped = normalize_backend_failure("K8S_NODE_PREEMPTED", &rules).expect("mapped");
    assert!(mapped.retryable);
}

#[test]
fn readiness_and_quota_contracts_are_deterministic() {
    let probe = BackendReadinessProbe {
        backend_class: "kubernetes".to_string(),
        healthy: true,
        reason: "ok".to_string(),
    };
    assert!(backend_ready_for_admission(
        &probe,
        &BackendMaintenanceMode::Active
    ));
    assert_eq!(quota_saturation_percent(100, 45), 45);
}

#[test]
fn cross_backend_replay_rules_are_enforced() {
    let rules = vec![
        CrossBackendReplayRule {
            from_backend: "local".to_string(),
            to_backend: "kubernetes".to_string(),
            replay_safe: true,
            reason: "same artifact contract".to_string(),
        },
        CrossBackendReplayRule {
            from_backend: "kubernetes".to_string(),
            to_backend: "slurm".to_string(),
            replay_safe: false,
            reason: "incompatible runtime assumptions".to_string(),
        },
    ];
    assert!(replay_allowed_across_backends("local", "kubernetes", &rules));
    assert!(!replay_allowed_across_backends("kubernetes", "slurm", &rules));
}
