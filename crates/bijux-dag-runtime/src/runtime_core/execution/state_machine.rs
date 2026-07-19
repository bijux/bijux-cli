//! Formal node and run state machine contracts.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeLifecycleState {
    Pending,
    Ready,
    Queued,
    Running,
    Succeeded,
    Failed,
    Cached,
    Skipped,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RunLifecycleState {
    Queued,
    Ready,
    Running,
    Succeeded,
    Failed,
    Cached,
    Skipped,
    Cancelled,
}

pub fn node_transition_allowed(from: NodeLifecycleState, to: NodeLifecycleState) -> bool {
    use NodeLifecycleState as S;
    matches!(
        (from, to),
        (S::Pending, S::Ready)
            | (S::Ready, S::Queued)
            | (S::Queued, S::Running)
            | (S::Running, S::Succeeded)
            | (S::Running, S::Failed)
            | (S::Ready, S::Cached)
            | (S::Queued, S::Cached)
            | (S::Pending, S::Skipped)
            | (S::Ready, S::Skipped)
            | (S::Queued, S::Skipped)
            | (S::Pending, S::Cancelled)
            | (S::Ready, S::Cancelled)
            | (S::Queued, S::Cancelled)
            | (S::Running, S::Cancelled)
            | (S::Pending, S::TimedOut)
            | (S::Ready, S::TimedOut)
            | (S::Queued, S::TimedOut)
            | (S::Running, S::TimedOut)
    )
}

pub fn run_transition_allowed(from: RunLifecycleState, to: RunLifecycleState) -> bool {
    use RunLifecycleState as S;
    matches!(
        (from, to),
        (S::Queued, S::Ready)
            | (S::Ready, S::Running)
            | (S::Running, S::Succeeded)
            | (S::Running, S::Failed)
            | (S::Running, S::Cached)
            | (S::Running, S::Skipped)
            | (S::Queued, S::Cancelled)
            | (S::Ready, S::Cancelled)
            | (S::Running, S::Cancelled)
    )
}

pub fn failure_propagation_is_deterministic(upstream_failed: bool, selected: bool) -> bool {
    if !selected {
        return false;
    }
    !upstream_failed
}
