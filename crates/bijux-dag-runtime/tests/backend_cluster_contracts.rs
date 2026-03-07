use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    artifact_collection_state, backend_ready_for_admission, canonical_k8s_terminal_events,
    classify_k8s_failure, equivalent_to_local, k8s_capability_declaration,
    map_node_policy_to_k8s_job, map_node_resources_to_k8s, matches_placement_policy,
    normalize_backend_failure, outputs_logs_equivalent, quota_saturation_percent,
    reconcile_k8s_watch_stream, reject_unsupported_k8s_fields, replay_allowed_across_backends,
    validate_k8s_injection, workdir_semantics, AdapterExecutionOutcome, ArtifactCollectionState,
    BackendCapabilityDescriptor, BackendFailureMappingRule, BackendMaintenanceMode,
    BackendReadinessProbe, CrossBackendReplayRule, K8sInjectionAvailability, K8sInjectionRequest,
    K8sWatchEvent, NodeExecutionContract, WorkdirVolumeKind,
};
use std::collections::BTreeMap;

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
    assert!(replay_allowed_across_backends(
        "local",
        "kubernetes",
        &rules
    ));
    assert!(!replay_allowed_across_backends(
        "kubernetes",
        "slurm",
        &rules
    ));
}

fn outcome(shape: &str) -> AdapterExecutionOutcome {
    AdapterExecutionOutcome {
        dag_shape: shape.to_string(),
        node_statuses: BTreeMap::from([
            ("extract".to_string(), "succeeded".to_string()),
            ("train".to_string(), "succeeded".to_string()),
        ]),
        output_hashes: BTreeMap::from([("model.bin".to_string(), "sha256:abc".to_string())]),
        stdout: "ok".to_string(),
        stderr: String::new(),
        cache_hit_nodes: vec![],
        replayed_nodes: vec![],
    }
}

#[test]
fn k8s_node_contract_maps_resources_timeout_retry_and_cancel_deterministically() {
    let node = NodeExecutionContract {
        cpu_units: 2,
        memory_mib: 1024,
        timeout_seconds: 600,
        max_retries: 3,
        retry_backoff_seconds: 10,
        cancel_grace_seconds: 30,
    };
    let mapped_resources = map_node_resources_to_k8s(&node);
    assert_eq!(mapped_resources.requests.cpu_millis, 2000);
    assert_eq!(mapped_resources.requests.memory_mib, 1024);
    assert_eq!(mapped_resources.limits.cpu_millis, 4000);
    assert_eq!(mapped_resources.limits.memory_mib, 1536);

    let mapped_policy = map_node_policy_to_k8s_job(&node);
    assert_eq!(mapped_policy.active_deadline_seconds, 600);
    assert_eq!(mapped_policy.backoff_limit, 3);
    assert_eq!(mapped_policy.retry_backoff_seconds, 10);
    assert_eq!(mapped_policy.termination_grace_period_seconds, 30);
}

#[test]
fn local_and_k8s_outcomes_are_equivalent_for_simple_fanout_fanin_cachehit_and_partial_replay() {
    for shape in ["simple", "fan-out", "fan-in"] {
        let local = outcome(shape);
        let k8s = outcome(shape);
        assert!(
            equivalent_to_local(&local, &k8s),
            "shape must match: {shape}"
        );
    }

    let mut local_cache = outcome("cache-hit");
    local_cache.cache_hit_nodes = vec!["extract".to_string()];
    let mut k8s_cache = outcome("cache-hit");
    k8s_cache.cache_hit_nodes = vec!["extract".to_string()];
    assert!(equivalent_to_local(&local_cache, &k8s_cache));

    let mut local_replay = outcome("partial-replay");
    local_replay.replayed_nodes = vec!["train".to_string()];
    let mut k8s_replay = outcome("partial-replay");
    k8s_replay.replayed_nodes = vec!["train".to_string()];
    assert!(equivalent_to_local(&local_replay, &k8s_replay));
}

#[test]
fn k8s_failure_classification_covers_eviction_image_pull_and_pending_timeout() {
    let eviction = classify_k8s_failure("K8S_POD_EVICTED");
    assert_eq!(eviction.runtime_failure_kind, "infrastructure");
    assert!(eviction.retryable);

    let image_pull = classify_k8s_failure("K8S_IMAGE_PULL_BACKOFF");
    assert_eq!(image_pull.runtime_failure_kind, "configuration");
    assert!(!image_pull.retryable);

    let pending = classify_k8s_failure("K8S_POD_PENDING_TIMEOUT");
    assert_eq!(pending.runtime_failure_kind, "infrastructure");
    assert!(pending.retryable);
}

#[test]
fn k8s_secret_and_config_injection_is_strict() {
    let required = K8sInjectionRequest {
        required_secrets: vec!["db-password".to_string()],
        required_configs: vec!["runtime-config".to_string()],
    };
    let available = K8sInjectionAvailability {
        available_secrets: vec!["db-password".to_string()],
        available_configs: vec!["runtime-config".to_string()],
    };
    assert!(validate_k8s_injection(&required, &available).is_ok());

    let missing_secret = K8sInjectionAvailability {
        available_secrets: vec![],
        available_configs: vec!["runtime-config".to_string()],
    };
    assert!(validate_k8s_injection(&required, &missing_secret).is_err());
}

#[test]
fn stdout_stderr_and_artifact_collection_contracts_hold_for_success_and_failure() {
    let local = outcome("simple");
    let k8s = outcome("simple");
    assert!(outputs_logs_equivalent(&local, &k8s));

    assert_eq!(
        artifact_collection_state(3, 3),
        ArtifactCollectionState::Complete
    );
    assert_eq!(
        artifact_collection_state(3, 1),
        ArtifactCollectionState::Partial
    );
    assert_eq!(
        artifact_collection_state(3, 0),
        ArtifactCollectionState::Missing
    );
}

#[test]
fn workdir_volume_semantics_distinguish_emptydir_and_persistent_volume() {
    let empty_dir = workdir_semantics(WorkdirVolumeKind::EmptyDir);
    assert!(!empty_dir.survives_pod_restart);
    assert!(!empty_dir.survives_reschedule);

    let pvc = workdir_semantics(WorkdirVolumeKind::PersistentVolumeClaim);
    assert!(pvc.survives_pod_restart);
    assert!(pvc.survives_reschedule);
}

#[test]
fn k8s_terminal_event_reduction_is_deterministic_under_async_duplicate_watch_events() {
    let events = vec![
        K8sWatchEvent {
            node_id: "train".to_string(),
            phase: "Running".to_string(),
            observed_at_millis: 2,
            sequence: 2,
        },
        K8sWatchEvent {
            node_id: "train".to_string(),
            phase: "Succeeded".to_string(),
            observed_at_millis: 4,
            sequence: 4,
        },
        K8sWatchEvent {
            node_id: "train".to_string(),
            phase: "Succeeded".to_string(),
            observed_at_millis: 3,
            sequence: 3,
        },
        K8sWatchEvent {
            node_id: "extract".to_string(),
            phase: "Failed".to_string(),
            observed_at_millis: 5,
            sequence: 9,
        },
        K8sWatchEvent {
            node_id: "extract".to_string(),
            phase: "Failed".to_string(),
            observed_at_millis: 1,
            sequence: 8,
        },
    ];
    let reduced = canonical_k8s_terminal_events(&events);
    assert_eq!(reduced["train"].phase, "Succeeded");
    assert_eq!(reduced["train"].sequence, 4);
    assert_eq!(reduced["extract"].phase, "Failed");
    assert_eq!(reduced["extract"].sequence, 9);
}

#[test]
fn watcher_reconnect_reconciles_without_corrupting_terminal_state() {
    let initial = BTreeMap::from([(
        "extract".to_string(),
        K8sWatchEvent {
            node_id: "extract".to_string(),
            phase: "Succeeded".to_string(),
            observed_at_millis: 10,
            sequence: 10,
        },
    )]);
    let reconnect_events = vec![
        K8sWatchEvent {
            node_id: "extract".to_string(),
            phase: "Succeeded".to_string(),
            observed_at_millis: 8,
            sequence: 9,
        },
        K8sWatchEvent {
            node_id: "train".to_string(),
            phase: "Succeeded".to_string(),
            observed_at_millis: 11,
            sequence: 11,
        },
    ];
    let merged = reconcile_k8s_watch_stream(&initial, &reconnect_events);
    assert_eq!(merged["extract"].sequence, 10);
    assert_eq!(merged["train"].sequence, 11);
}

#[test]
fn node_selector_affinity_capabilities_are_declared() {
    let caps = k8s_capability_declaration();
    assert!(caps.supports_node_selector);
    assert!(caps.supports_node_affinity);
    assert!(caps.supports_pod_affinity);
}

#[test]
fn unsupported_kubernetes_only_fields_are_rejected_by_contract() {
    assert!(reject_unsupported_k8s_fields(&["hostNetwork".to_string()]).is_err());
    assert!(reject_unsupported_k8s_fields(&["runtimeClassName".to_string()]).is_err());
    assert!(reject_unsupported_k8s_fields(&["safeField".to_string()]).is_ok());
}
