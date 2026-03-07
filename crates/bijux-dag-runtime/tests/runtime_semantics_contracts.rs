use bijux_dag_runtime::{
    append_audit_event, artifact_commit_guaranteed, artifact_lineage_complete,
    cache_entry_invalidated, cache_entry_valid, cancellation_is_terminal, classify_failure,
    dependency_resolution_is_complete, deterministic_schedule_order, fairness_is_satisfied,
    replay_equivalent, recovery_action_required, retry_allowed, run_manifest_valid,
    timeout_triggered, trace_event_count_by_category, CacheValidationInput,
    ManifestVerificationInput, ReadyNode, RecoveryInput, RetryPolicySemantics,
    RuntimeAuditEvent, RuntimeFailureClass,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn deterministic_scheduling_fairness_and_tie_break_are_stable() {
    let nodes = vec![
        ReadyNode { node_id: "b".to_string(), priority: 2, attempt: 1, ready_unix_ms: 1000 },
        ReadyNode { node_id: "a".to_string(), priority: 2, attempt: 1, ready_unix_ms: 1000 },
        ReadyNode { node_id: "c".to_string(), priority: 1, attempt: 1, ready_unix_ms: 999 },
    ];
    let starvation = BTreeMap::from([
        ("c".to_string(), 5_u32),
        ("a".to_string(), 0_u32),
        ("b".to_string(), 0_u32),
    ]);
    let ordered = deterministic_schedule_order(nodes, &starvation);
    assert_eq!(ordered[0].node_id, "c");
    assert_eq!(ordered[1].node_id, "a");
    assert_eq!(ordered[2].node_id, "b");
    assert!(fairness_is_satisfied(&ordered, 3, &starvation));
}

#[test]
fn retry_timeout_cancellation_dependency_and_artifact_commit_are_enforced() {
    let retry = RetryPolicySemantics { max_attempts: 3, initial_backoff_ms: 100, exponential: true };
    assert!(retry_allowed(2, &retry));
    assert!(!retry_allowed(3, &retry));
    assert!(timeout_triggered(10, 50, Some(20)));
    assert!(cancellation_is_terminal(true, true));
    let succeeded = BTreeSet::from(["extract".to_string(), "transform".to_string()]);
    assert!(dependency_resolution_is_complete(&["extract".to_string()], &succeeded));
    assert!(artifact_commit_guaranteed(true, true, true));
}

#[test]
fn cache_replay_manifest_recovery_lineage_and_failure_classification_are_consistent() {
    assert!(cache_entry_valid(&CacheValidationInput {
        fingerprint_matches: true,
        schema_matches: true,
        proof_present: true,
    }));
    assert!(cache_entry_invalidated(false, true, false));
    assert!(replay_equivalent("abc", "abc"));
    assert!(run_manifest_valid(&ManifestVerificationInput {
        has_run_header: true,
        has_trace_index: true,
        has_outputs_index: true,
        totals_consistent: true,
    }));
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: false,
        partial_artifacts_present: false,
    }));
    let lineage = BTreeMap::from([
        ("a/out".to_string(), "extract".to_string()),
        ("b/out".to_string(), "transform".to_string()),
    ]);
    assert!(artifact_lineage_complete(&["a/out".to_string(), "b/out".to_string()], &lineage));
    assert_eq!(
        classify_failure(false, false, false, true, false, false),
        RuntimeFailureClass::PolicyViolation
    );
}

#[test]
fn runtime_audit_and_trace_events_are_recorded_and_grouped() {
    let mut events = Vec::new();
    append_audit_event(
        &mut events,
        RuntimeAuditEvent {
            event_id: "evt-1".to_string(),
            run_id: "run-1".to_string(),
            node_id: Some("extract".to_string()),
            category: "execution".to_string(),
            details: BTreeMap::from([("status".to_string(), "ok".to_string())]),
        },
    );
    append_audit_event(
        &mut events,
        RuntimeAuditEvent {
            event_id: "evt-2".to_string(),
            run_id: "run-1".to_string(),
            node_id: Some("transform".to_string()),
            category: "failure".to_string(),
            details: BTreeMap::from([("class".to_string(), "timeout".to_string())]),
        },
    );
    let grouped = trace_event_count_by_category(&events);
    assert_eq!(grouped.get("execution"), Some(&1));
    assert_eq!(grouped.get("failure"), Some(&1));
}
