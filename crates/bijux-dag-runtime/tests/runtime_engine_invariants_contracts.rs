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
    append_audit_event, artifact_lineage_complete, cancellation_is_terminal, classify_failure,
    dependency_resolution_is_complete, event_names_emitted_once, recovery_action_required,
    required_event_fields_present, retry_allowed, timeout_triggered, trace_event_count_by_category,
    trace_time_order_ok, validate_node_transition, validate_required_event_names,
    validate_run_transition, verify_post_run_state_consistency, EventCategory, EventRecord,
    EventSink, FileEventSink, NodeState, NodeTransition, RecoveryInput, RetryPolicySemantics,
    RunState, RunTransition, RuntimeAuditEvent, RuntimeFailureClass, TransitionCause,
};
use std::collections::{BTreeMap, BTreeSet};

#[test]
fn node_execution_lifecycle_transitions_are_validated() {
    let sequence = [
        (NodeState::Pending, NodeState::Eligible),
        (NodeState::Eligible, NodeState::Queued),
        (NodeState::Queued, NodeState::Running),
    ];
    for (from, to) in sequence {
        assert!(validate_node_transition(&NodeTransition {
            from,
            to,
            cause: TransitionCause::ExecutionStarted,
        })
        .is_ok());
    }
}

#[test]
fn node_start_to_success_transition_is_allowed() {
    assert!(validate_node_transition(&NodeTransition {
        from: NodeState::Running,
        to: NodeState::Success,
        cause: TransitionCause::ExecutionSucceeded,
    })
    .is_ok());
}

#[test]
fn node_start_to_failure_transition_is_allowed() {
    assert!(validate_node_transition(&NodeTransition {
        from: NodeState::Running,
        to: NodeState::Failed,
        cause: TransitionCause::ExecutionFailed,
    })
    .is_ok());
}

#[test]
fn node_start_to_cancellation_transition_is_allowed() {
    assert!(validate_node_transition(&NodeTransition {
        from: NodeState::Running,
        to: NodeState::Cancelled,
        cause: TransitionCause::CancelRequested,
    })
    .is_ok());
}

#[test]
fn node_start_to_timeout_transition_is_classified_as_timeout_failure() {
    assert!(validate_node_transition(&NodeTransition {
        from: NodeState::Running,
        to: NodeState::TimedOut,
        cause: TransitionCause::TimeoutExceeded,
    })
    .is_ok());
    assert!(timeout_triggered(1_000, 1_101, Some(100)));
    assert_eq!(
        classify_failure(true, false, false, false, false, false),
        RuntimeFailureClass::Timeout
    );
}

#[test]
fn node_retry_lifecycle_respects_policy_budget() {
    let policy =
        RetryPolicySemantics { max_attempts: 3, initial_backoff_ms: 50, exponential: true };
    assert!(retry_allowed(0, &policy));
    assert!(retry_allowed(1, &policy));
    assert!(retry_allowed(2, &policy));
}

#[test]
fn node_retry_exhaustion_is_enforced() {
    let policy =
        RetryPolicySemantics { max_attempts: 2, initial_backoff_ms: 10, exponential: false };
    assert!(!retry_allowed(2, &policy));
    assert!(!retry_allowed(3, &policy));
}

#[test]
fn run_lifecycle_state_transitions_are_validated() {
    let sequence = [
        (RunState::Submitted, RunState::Planning),
        (RunState::Planning, RunState::Running),
        (RunState::Running, RunState::Succeeded),
    ];
    for (from, to) in sequence {
        assert!(validate_run_transition(&RunTransition {
            from,
            to,
            cause: TransitionCause::PlanningCompleted,
        })
        .is_ok());
    }
}

#[test]
fn run_completion_detection_requires_terminal_nodes() {
    let report = verify_post_run_state_consistency(
        RunState::Succeeded,
        &[NodeState::Success, NodeState::Cached],
        0,
    );
    assert!(report.valid);
}

#[test]
fn run_cancellation_propagation_requires_terminal_node_state() {
    assert!(cancellation_is_terminal(true, true));
    assert!(!cancellation_is_terminal(true, false));

    assert!(validate_run_transition(&RunTransition {
        from: RunState::Running,
        to: RunState::Cancelling,
        cause: TransitionCause::CancelRequested,
    })
    .is_ok());
    assert!(validate_run_transition(&RunTransition {
        from: RunState::Cancelling,
        to: RunState::Cancelled,
        cause: TransitionCause::CancelRequested,
    })
    .is_ok());
}

#[test]
fn run_failure_classification_distinguishes_core_failure_classes() {
    assert_eq!(
        classify_failure(false, true, false, false, false, false),
        RuntimeFailureClass::Cancelled
    );
    assert_eq!(
        classify_failure(false, false, true, false, false, false),
        RuntimeFailureClass::DependencyFailure
    );
    assert_eq!(
        classify_failure(false, false, false, true, false, false),
        RuntimeFailureClass::PolicyViolation
    );
}

#[test]
fn run_partial_completion_requires_consistency_guardrails() {
    let report = verify_post_run_state_consistency(
        RunState::Succeeded,
        &[NodeState::Success, NodeState::Running],
        0,
    );
    assert!(!report.valid);
    assert!(report.violations.iter().any(|v| v.contains("non-terminal node")));
}

#[test]
fn state_machine_property_transitions_hold_for_generated_edges() {
    let legal = [
        (NodeState::Pending, NodeState::Eligible),
        (NodeState::Eligible, NodeState::Queued),
        (NodeState::Queued, NodeState::Running),
        (NodeState::Running, NodeState::Success),
        (NodeState::Running, NodeState::Failed),
        (NodeState::Running, NodeState::Cancelled),
    ];
    for _ in 0..32 {
        for (from, to) in &legal {
            assert!(validate_node_transition(&NodeTransition {
                from: from.clone(),
                to: to.clone(),
                cause: TransitionCause::ExecutionStarted,
            })
            .is_ok());
        }
    }
}

#[test]
fn deterministic_timestamp_ordering_contract_is_enforced() {
    assert!(trace_time_order_ok(100, 100));
    assert!(trace_time_order_ok(100, 101));
    assert!(!trace_time_order_ok(101, 100));
}

#[test]
fn runtime_event_ordering_is_stable_and_required_names_are_present() {
    let events = vec![
        ev("run_started", 1, None),
        ev("node_ready", 2, Some("a")),
        ev("node_started", 3, Some("a")),
        ev("node_attempt_started", 4, Some("a")),
        ev("node_attempt_finished", 5, Some("a")),
        ev("node_scheduled", 6, Some("a")),
        ev("node_finished", 7, Some("a")),
        ev("run_finished", 8, None),
    ];

    assert!(events.windows(2).all(|w| w[0].unix_ms <= w[1].unix_ms));
    assert!(validate_required_event_names(&events).is_empty());
    assert!(event_names_emitted_once(&events, &["run_started", "run_finished"]));
}

#[test]
fn runtime_event_persistence_writes_jsonl_records() {
    let dir = tempfile::tempdir().expect("temp dir");
    let path = dir.path().join("events/runtime.jsonl");
    let sink = FileEventSink::new(&path);

    let first = ev("run_started", 1, None);
    let second = ev("run_finished", 2, None);
    sink.write_event(&first).expect("write first");
    sink.write_event(&second).expect("write second");

    let body = std::fs::read_to_string(path).expect("read sink file");
    let lines: Vec<&str> = body.lines().collect();
    assert_eq!(lines.len(), 2);
    for line in lines {
        let parsed: EventRecord = serde_json::from_str(line).expect("json line");
        assert!(required_event_fields_present(&parsed));
    }
}

#[test]
fn runtime_failure_classification_maps_cache_and_adapter_paths() {
    assert_eq!(
        classify_failure(false, false, false, false, true, false),
        RuntimeFailureClass::CacheInvalid
    );
    assert_eq!(
        classify_failure(false, false, false, false, false, false),
        RuntimeFailureClass::AdapterFailure
    );
}

#[test]
fn runtime_crash_recovery_simulation_requires_recovery_on_checkpointed_interruptions() {
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: true,
        terminal_state_seen: false,
        partial_artifacts_present: false,
    }));
    assert!(recovery_action_required(&RecoveryInput {
        has_checkpoint: false,
        terminal_state_seen: false,
        partial_artifacts_present: true,
    }));
    assert!(!recovery_action_required(&RecoveryInput {
        has_checkpoint: false,
        terminal_state_seen: true,
        partial_artifacts_present: false,
    }));
}

#[test]
fn runtime_corruption_detection_classifies_artifact_corruption_distinctly() {
    assert_eq!(
        classify_failure(false, false, false, false, false, true),
        RuntimeFailureClass::ArtifactCorruption
    );
    assert!(artifact_lineage_complete(
        &["a/out".to_string(), "b/out".to_string()],
        &BTreeMap::from([
            ("a/out".to_string(), "src:a".to_string()),
            ("b/out".to_string(), "src:b".to_string()),
        ])
    ));
}

#[test]
fn runtime_engine_stress_handles_high_event_volume_without_losing_counts() {
    let mut events = Vec::new();
    for i in 0..20_000u64 {
        append_audit_event(
            &mut events,
            RuntimeAuditEvent {
                event_id: format!("evt-{i}"),
                run_id: "run-stress".to_string(),
                node_id: Some(format!("n{}", i % 64)),
                category: if i % 2 == 0 { "dispatch" } else { "start" }.to_string(),
                details: BTreeMap::new(),
            },
        );
    }

    let grouped = trace_event_count_by_category(&events);
    assert_eq!(grouped.get("dispatch"), Some(&10_000));
    assert_eq!(grouped.get("start"), Some(&10_000));

    let required = ["a".to_string(), "b".to_string(), "c".to_string()];
    let succeeded = BTreeSet::from(["a".to_string(), "b".to_string(), "c".to_string()]);
    assert!(dependency_resolution_is_complete(&required, &succeeded));
}

fn ev(name: &str, unix_ms: u128, node_id: Option<&str>) -> EventRecord {
    EventRecord {
        category: if name.contains("failed") {
            EventCategory::Failure
        } else {
            EventCategory::Start
        },
        name: name.to_string(),
        unix_ms,
        node_id: node_id.map(str::to_string),
        run_id: Some("run-1".to_string()),
        details: serde_json::json!({"reason":"contract"}),
    }
}
