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
    validate_node_transition, validate_run_transition, NodeState, NodeTransition, RunState,
    RunTransition, TransitionCause,
};

#[test]
fn accepts_legal_node_transitions() {
    let legal = [
        (NodeState::Pending, NodeState::Eligible),
        (NodeState::Eligible, NodeState::Queued),
        (NodeState::Queued, NodeState::Running),
        (NodeState::Running, NodeState::Success),
        (NodeState::Running, NodeState::Failed),
        (NodeState::Eligible, NodeState::Skipped),
        (NodeState::Queued, NodeState::Skipped),
        (NodeState::Eligible, NodeState::Cached),
        (NodeState::Queued, NodeState::Cached),
        (NodeState::Running, NodeState::Cancelled),
    ];
    for (from, to) in legal {
        let transition = NodeTransition { from, to, cause: TransitionCause::SchedulerQueued };
        assert!(validate_node_transition(&transition).is_ok());
    }
}

#[test]
fn rejects_illegal_node_transitions() {
    let illegal = [
        (NodeState::Pending, NodeState::Running),
        (NodeState::Success, NodeState::Failed),
        (NodeState::Failed, NodeState::Success),
        (NodeState::Skipped, NodeState::Running),
    ];
    for (from, to) in illegal {
        let transition = NodeTransition { from, to, cause: TransitionCause::ExecutionStarted };
        assert!(validate_node_transition(&transition).is_err());
    }
}

#[test]
fn accepts_legal_run_transitions() {
    let legal = [
        (RunState::Submitted, RunState::Planning),
        (RunState::Planning, RunState::Running),
        (RunState::Running, RunState::Paused),
        (RunState::Paused, RunState::Running),
        (RunState::Running, RunState::Interrupted),
        (RunState::Interrupted, RunState::Running),
        (RunState::Interrupted, RunState::Cancelling),
        (RunState::Running, RunState::Cancelling),
        (RunState::Cancelling, RunState::Cancelled),
        (RunState::Running, RunState::TimedOut),
        (RunState::Running, RunState::Failed),
        (RunState::Running, RunState::Succeeded),
    ];
    for (from, to) in legal {
        let transition = RunTransition { from, to, cause: TransitionCause::PlanningCompleted };
        assert!(validate_run_transition(&transition).is_ok());
    }
}

#[test]
fn rejects_illegal_run_transitions() {
    let illegal = [
        (RunState::Submitted, RunState::Running),
        (RunState::Paused, RunState::Succeeded),
        (RunState::Succeeded, RunState::Running),
        (RunState::Cancelled, RunState::Running),
    ];
    for (from, to) in illegal {
        let transition = RunTransition { from, to, cause: TransitionCause::ExecutionFailed };
        assert!(validate_run_transition(&transition).is_err());
    }
}
