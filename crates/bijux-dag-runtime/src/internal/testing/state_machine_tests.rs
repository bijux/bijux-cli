use crate::state_machine::{
    failure_propagation_is_deterministic, node_transition_allowed, run_transition_allowed,
    NodeLifecycleState, RunLifecycleState,
};

#[test]
fn node_state_machine_has_explicit_legal_edges() {
    use NodeLifecycleState as S;
    assert!(node_transition_allowed(S::Pending, S::Ready));
    assert!(node_transition_allowed(S::Ready, S::Queued));
    assert!(node_transition_allowed(S::Queued, S::Running));
    assert!(node_transition_allowed(S::Running, S::Succeeded));
    assert!(node_transition_allowed(S::Running, S::Failed));
    assert!(node_transition_allowed(S::Ready, S::Cached));
    assert!(node_transition_allowed(S::Queued, S::Cached));
    assert!(node_transition_allowed(S::Pending, S::Skipped));
    assert!(node_transition_allowed(S::Ready, S::Skipped));
    assert!(node_transition_allowed(S::Queued, S::Skipped));
    assert!(node_transition_allowed(S::Pending, S::TimedOut));
    assert!(node_transition_allowed(S::Ready, S::TimedOut));
    assert!(node_transition_allowed(S::Queued, S::TimedOut));
    assert!(node_transition_allowed(S::Running, S::TimedOut));
    assert!(node_transition_allowed(S::Running, S::Cancelled));
    assert!(!node_transition_allowed(S::Succeeded, S::Running));
}

#[test]
fn run_state_machine_has_explicit_legal_edges() {
    use RunLifecycleState as S;
    assert!(run_transition_allowed(S::Queued, S::Ready));
    assert!(run_transition_allowed(S::Ready, S::Running));
    assert!(run_transition_allowed(S::Running, S::Succeeded));
    assert!(run_transition_allowed(S::Running, S::Failed));
    assert!(run_transition_allowed(S::Running, S::Cancelled));
    assert!(!run_transition_allowed(S::Succeeded, S::Running));
}

#[test]
fn failure_propagation_rule_is_deterministic() {
    assert!(!failure_propagation_is_deterministic(true, true));
    assert!(failure_propagation_is_deterministic(false, true));
    assert!(!failure_propagation_is_deterministic(false, false));
}

#[test]
fn terminal_node_states_have_no_outgoing_transitions() {
    use NodeLifecycleState as S;
    let terminal = [S::Succeeded, S::Failed, S::Cached, S::Skipped, S::Cancelled, S::TimedOut];
    let all = [
        S::Pending,
        S::Ready,
        S::Queued,
        S::Running,
        S::Succeeded,
        S::Failed,
        S::Cached,
        S::Skipped,
        S::Cancelled,
        S::TimedOut,
    ];
    for from in terminal {
        for to in all {
            assert!(!node_transition_allowed(from, to));
        }
    }
}

#[test]
fn terminal_run_states_have_no_outgoing_transitions() {
    use RunLifecycleState as S;
    let terminal = [S::Succeeded, S::Failed, S::Cached, S::Skipped, S::Cancelled];
    let all = [
        S::Queued,
        S::Ready,
        S::Running,
        S::Succeeded,
        S::Failed,
        S::Cached,
        S::Skipped,
        S::Cancelled,
    ];
    for from in terminal {
        for to in all {
            assert!(!run_transition_allowed(from, to));
        }
    }
}
