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
    imported_run_distinguishable, terminal_transition_audit_events, validate_node_transition,
    validate_run_transition, verify_post_run_state_consistency, NodeState, NodeTransition,
    ResumeFailureMode, ResumeSummary, RunId, RunSnapshot, RunState, RunTransition, TransitionCause,
};

#[test]
fn terminal_nodes_never_revert_to_nonterminal_states() {
    let revert = NodeTransition {
        from: NodeState::Success,
        to: NodeState::Running,
        cause: TransitionCause::ExecutionStarted,
    };
    assert!(validate_node_transition(&revert).is_err());
}

#[test]
fn cached_and_skipped_states_are_terminal_and_unambiguous() {
    let cached_ok = NodeTransition {
        from: NodeState::Eligible,
        to: NodeState::Cached,
        cause: TransitionCause::CachedReuse,
    };
    assert!(validate_node_transition(&cached_ok).is_ok());

    let cached_revert = NodeTransition {
        from: NodeState::Cached,
        to: NodeState::Running,
        cause: TransitionCause::ExecutionStarted,
    };
    assert!(validate_node_transition(&cached_revert).is_err());

    let skipped_ok = NodeTransition {
        from: NodeState::Queued,
        to: NodeState::Skipped,
        cause: TransitionCause::SelectionFiltered,
    };
    assert!(validate_node_transition(&skipped_ok).is_ok());

    let timed_out_ok = NodeTransition {
        from: NodeState::Running,
        to: NodeState::TimedOut,
        cause: TransitionCause::TimeoutExceeded,
    };
    assert!(validate_node_transition(&timed_out_ok).is_ok());
}

#[test]
fn cancelled_and_failed_runs_must_be_coherent() {
    let cancelled = verify_post_run_state_consistency(
        RunState::Cancelled,
        &[NodeState::Cancelled, NodeState::Skipped],
        0,
    );
    assert!(cancelled.valid);

    let failed_without_cause =
        verify_post_run_state_consistency(RunState::Failed, &[NodeState::Failed], 0);
    assert!(!failed_without_cause.valid);

    let timed_out = verify_post_run_state_consistency(
        RunState::TimedOut,
        &[NodeState::TimedOut, NodeState::Success],
        0,
    );
    assert!(timed_out.valid);
}

#[test]
fn retry_attempts_keep_node_identity_but_change_attempt_identity() {
    let first = bijux_dag_runtime::RunAttempt {
        attempt_index: 1,
        run_id: RunId("run-1".to_string()),
        parent_run_id: None,
        reason: "initial".to_string(),
        resume_summary: None,
    };
    let second = bijux_dag_runtime::RunAttempt {
        attempt_index: 2,
        run_id: RunId("run-1".to_string()),
        parent_run_id: Some(RunId("run-1".to_string())),
        reason: "retry".to_string(),
        resume_summary: Some(ResumeSummary {
            failure_mode: ResumeFailureMode::RerunIncomplete,
            reused_nodes: vec!["extract".to_string()],
            rerun_nodes: vec!["publish".to_string()],
            rejected_nodes: Vec::new(),
        }),
    };
    assert_eq!(first.run_id, second.run_id);
    assert_ne!(first.attempt_index, second.attempt_index);
}

#[test]
fn imported_runs_are_distinguishable_from_native_runs() {
    let imported = RunSnapshot {
        run_id: RunId("run-imported".to_string()),
        graph_snapshot_path: "graph.snapshot.json".to_string(),
        planner_config: "planner".to_string(),
        scheduler_config: "scheduler".to_string(),
        policy_config: "policy".to_string(),
        provenance: "imported".to_string(),
        submission_source: "import".to_string(),
        trigger_source: "bundle".to_string(),
        operator: "system".to_string(),
        labels: vec![],
        parent_run_id: None,
        requested_selectors: vec![],
        selected_nodes: vec![],
        dependency_closure_enabled: true,
        replay_source_run_id: None,
        partial_rerun_contract: None,
    };
    assert!(imported_run_distinguishable(&imported));
}

#[test]
fn terminal_transition_audit_events_emit_for_terminal_paths() {
    let node_transitions = vec![NodeTransition {
        from: NodeState::Running,
        to: NodeState::Failed,
        cause: TransitionCause::ExecutionFailed,
    }];
    let run_transitions = vec![RunTransition {
        from: RunState::Running,
        to: RunState::TimedOut,
        cause: TransitionCause::TimeoutExceeded,
    }];
    let events = terminal_transition_audit_events(&node_transitions, &run_transitions);
    assert_eq!(events.len(), 2);
}

#[test]
fn state_machine_snapshot_fixture_represents_legal_evolution() {
    #[derive(serde::Deserialize)]
    struct TraceFixture {
        node_transitions: Vec<NodeTransition>,
        run_transitions: Vec<RunTransition>,
    }
    for fixture_path in [
        "tests/fixtures/state_machine/evolution_trace.json",
        "tests/fixtures/state_machine/cancellation_trace.json",
    ] {
        let raw = std::fs::read_to_string(fixture_path).expect("fixture must exist");
        let fixture: TraceFixture = serde_json::from_str(&raw).expect("fixture must parse");
        for transition in &fixture.node_transitions {
            assert!(validate_node_transition(transition).is_ok());
        }
        for transition in &fixture.run_transitions {
            assert!(validate_run_transition(transition).is_ok());
        }
    }
}
