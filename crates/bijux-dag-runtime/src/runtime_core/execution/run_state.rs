use bijux_dag_artifacts::{NodeCounts, RunSummary};
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RunId(pub String);

impl RunId {
    pub fn parse(value: &str) -> Result<Self, String> {
        if value.is_empty() {
            return Err("run id must not be empty".to_string());
        }
        if !value.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_') {
            return Err("run id contains invalid characters".to_string());
        }
        Ok(Self(value.to_string()))
    }
}

impl fmt::Display for RunId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeState {
    Pending,
    Eligible,
    Queued,
    Running,
    Success,
    Failed,
    Skipped,
    Cached,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunState {
    Submitted,
    Planning,
    Running,
    Paused,
    Interrupted,
    Cancelling,
    Cancelled,
    TimedOut,
    Failed,
    Succeeded,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransitionCause {
    Submission,
    PlanningCompleted,
    SchedulerEligible,
    SchedulerQueued,
    ExecutionStarted,
    ExecutionSucceeded,
    ExecutionFailed,
    CachedReuse,
    PolicyDenied,
    DependencyFailed,
    SelectionFiltered,
    ExecutionAborted,
    CancelRequested,
    TimeoutExceeded,
    ReplayReused,
    ReplayReexecuted,
    ResumeRequested,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeTransition {
    pub from: NodeState,
    pub to: NodeState,
    pub cause: TransitionCause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunTransition {
    pub from: RunState,
    pub to: RunState,
    pub cause: TransitionCause,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunAttempt {
    pub attempt_index: u32,
    pub run_id: RunId,
    pub parent_run_id: Option<RunId>,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resume_summary: Option<ResumeSummary>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeFailureMode {
    RerunIncomplete,
    RejectIncomplete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResumeSummary {
    pub failure_mode: ResumeFailureMode,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reused_nodes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rerun_nodes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rejected_nodes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSnapshot {
    pub run_id: RunId,
    pub graph_snapshot_path: String,
    pub planner_config: String,
    pub scheduler_config: String,
    pub policy_config: String,
    pub provenance: String,
    pub submission_source: String,
    pub trigger_source: String,
    pub operator: String,
    pub labels: Vec<String>,
    pub parent_run_id: Option<RunId>,
    #[serde(default)]
    pub requested_selectors: Vec<String>,
    pub selected_nodes: Vec<String>,
    pub dependency_closure_enabled: bool,
    pub replay_source_run_id: Option<RunId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partial_rerun_contract: Option<PartialRerunContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialRerunContract {
    pub selected_nodes: Vec<String>,
    pub invalidated_downstream_nodes: Vec<String>,
    pub stale_downstream_reuse_forbidden: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplayNodeProvenance {
    pub node_id: String,
    pub action: ReplayNodeAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayNodeAction {
    Reexecuted,
    Reused,
    Skipped,
    Restored,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunSummaryV2 {
    pub run_id: RunId,
    pub state: RunState,
    pub counts: NodeCounts,
}

impl RunSummaryV2 {
    pub fn to_artifact_summary(&self) -> RunSummary {
        RunSummary {
            total_nodes: self.counts.success
                + self.counts.failed
                + self.counts.skipped
                + self.counts.cached
                + self.counts.cancelled,
            success: self.counts.success,
            failed: self.counts.failed,
            skipped: self.counts.skipped,
            cached: self.counts.cached,
            cancelled: self.counts.cancelled,
            promoted_outputs: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunComparison {
    pub semantic_differences: Vec<String>,
    pub incidental_differences: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunCompactionPolicy {
    pub max_event_count_before_compaction: usize,
    pub keep_latest_attempts: usize,
}

impl Default for RunCompactionPolicy {
    fn default() -> Self {
        Self { max_event_count_before_compaction: 10_000, keep_latest_attempts: 5 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionAuditEvent {
    pub invariant_id: String,
    pub entity: String,
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateConsistencyReport {
    pub valid: bool,
    pub violations: Vec<String>,
}

pub const INV_NODE_TRANSITION_PENDING_ELIGIBLE: &str = "INV-NODE-TRANSITION-001";
pub const INV_NODE_TRANSITION_ELIGIBLE_QUEUED: &str = "INV-NODE-TRANSITION-002";
pub const INV_NODE_TRANSITION_QUEUED_RUNNING: &str = "INV-NODE-TRANSITION-003";
pub const INV_NODE_TRANSITION_RUNNING_SUCCESS: &str = "INV-NODE-TERMINAL-001";
pub const INV_NODE_TRANSITION_RUNNING_FAILED: &str = "INV-NODE-TERMINAL-002";
pub const INV_NODE_TRANSITION_ELIGIBLE_SKIPPED: &str = "INV-NODE-TERMINAL-003";
pub const INV_NODE_TRANSITION_QUEUED_SKIPPED: &str = "INV-NODE-TERMINAL-004";
pub const INV_NODE_TRANSITION_ELIGIBLE_CACHED: &str = "INV-NODE-TERMINAL-005";
pub const INV_NODE_TRANSITION_QUEUED_CACHED: &str = "INV-NODE-TERMINAL-006";
pub const INV_NODE_TRANSITION_RUNNING_CANCELLED: &str = "INV-NODE-TERMINAL-007";
pub const INV_NODE_TRANSITION_QUEUED_FAILED: &str = "INV-NODE-TERMINAL-008";
pub const INV_NODE_TRANSITION_PENDING_CANCELLED: &str = "INV-NODE-TERMINAL-009";
pub const INV_NODE_TRANSITION_ELIGIBLE_CANCELLED: &str = "INV-NODE-TERMINAL-010";
pub const INV_NODE_TRANSITION_QUEUED_CANCELLED: &str = "INV-NODE-TERMINAL-011";
pub const INV_NODE_TRANSITION_PENDING_TIMED_OUT: &str = "INV-NODE-TERMINAL-012";
pub const INV_NODE_TRANSITION_ELIGIBLE_TIMED_OUT: &str = "INV-NODE-TERMINAL-013";
pub const INV_NODE_TRANSITION_QUEUED_TIMED_OUT: &str = "INV-NODE-TERMINAL-014";
pub const INV_NODE_TRANSITION_RUNNING_TIMED_OUT: &str = "INV-NODE-TERMINAL-015";
pub const INV_NODE_TRANSITION_PENDING_SKIPPED: &str = "INV-NODE-TERMINAL-016";
pub const INV_NODE_TERMINAL_NO_REVERT: &str = "INV-NODE-TERMINAL-REVERT-001";

pub const INV_RUN_TRANSITION_SUBMITTED_PLANNING: &str = "INV-RUN-TRANSITION-001";
pub const INV_RUN_TRANSITION_PLANNING_RUNNING: &str = "INV-RUN-TRANSITION-002";
pub const INV_RUN_TRANSITION_RUNNING_PAUSED: &str = "INV-RUN-TRANSITION-003";
pub const INV_RUN_TRANSITION_PAUSED_RUNNING: &str = "INV-RUN-TRANSITION-004";
pub const INV_RUN_TRANSITION_RUNNING_INTERRUPTED: &str = "INV-RUN-TRANSITION-005";
pub const INV_RUN_TRANSITION_INTERRUPTED_RUNNING: &str = "INV-RUN-TRANSITION-006";
pub const INV_RUN_TRANSITION_INTERRUPTED_CANCELLING: &str = "INV-RUN-TRANSITION-007";
pub const INV_RUN_TRANSITION_RUNNING_CANCELLING: &str = "INV-RUN-TRANSITION-008";
pub const INV_RUN_TRANSITION_CANCELLING_CANCELLED: &str = "INV-RUN-TERMINAL-001";
pub const INV_RUN_TRANSITION_RUNNING_TIMED_OUT: &str = "INV-RUN-TERMINAL-002";
pub const INV_RUN_TRANSITION_RUNNING_FAILED: &str = "INV-RUN-TERMINAL-003";
pub const INV_RUN_TRANSITION_RUNNING_SUCCEEDED: &str = "INV-RUN-TERMINAL-004";
pub const INV_RUN_FAILED_CAUSAL_FAILURE: &str = "INV-RUN-FAILED-CAUSAL-001";

pub fn node_transition_invariant_id(from: NodeState, to: NodeState) -> Option<&'static str> {
    use NodeState as S;
    match (from, to) {
        (S::Pending, S::Eligible) => Some(INV_NODE_TRANSITION_PENDING_ELIGIBLE),
        (S::Eligible, S::Queued) => Some(INV_NODE_TRANSITION_ELIGIBLE_QUEUED),
        (S::Queued, S::Running) => Some(INV_NODE_TRANSITION_QUEUED_RUNNING),
        (S::Running, S::Success) => Some(INV_NODE_TRANSITION_RUNNING_SUCCESS),
        (S::Running, S::Failed) => Some(INV_NODE_TRANSITION_RUNNING_FAILED),
        (S::Eligible, S::Skipped) => Some(INV_NODE_TRANSITION_ELIGIBLE_SKIPPED),
        (S::Queued, S::Skipped) => Some(INV_NODE_TRANSITION_QUEUED_SKIPPED),
        (S::Eligible, S::Cached) => Some(INV_NODE_TRANSITION_ELIGIBLE_CACHED),
        (S::Queued, S::Cached) => Some(INV_NODE_TRANSITION_QUEUED_CACHED),
        (S::Running, S::Cancelled) => Some(INV_NODE_TRANSITION_RUNNING_CANCELLED),
        (S::Queued, S::Failed) => Some(INV_NODE_TRANSITION_QUEUED_FAILED),
        (S::Pending, S::Skipped) => Some(INV_NODE_TRANSITION_PENDING_SKIPPED),
        (S::Pending, S::Cancelled) => Some(INV_NODE_TRANSITION_PENDING_CANCELLED),
        (S::Eligible, S::Cancelled) => Some(INV_NODE_TRANSITION_ELIGIBLE_CANCELLED),
        (S::Queued, S::Cancelled) => Some(INV_NODE_TRANSITION_QUEUED_CANCELLED),
        (S::Pending, S::TimedOut) => Some(INV_NODE_TRANSITION_PENDING_TIMED_OUT),
        (S::Eligible, S::TimedOut) => Some(INV_NODE_TRANSITION_ELIGIBLE_TIMED_OUT),
        (S::Queued, S::TimedOut) => Some(INV_NODE_TRANSITION_QUEUED_TIMED_OUT),
        (S::Running, S::TimedOut) => Some(INV_NODE_TRANSITION_RUNNING_TIMED_OUT),
        _ => None,
    }
}

pub fn run_transition_invariant_id(from: RunState, to: RunState) -> Option<&'static str> {
    use RunState as S;
    match (from, to) {
        (S::Submitted, S::Planning) => Some(INV_RUN_TRANSITION_SUBMITTED_PLANNING),
        (S::Planning, S::Running) => Some(INV_RUN_TRANSITION_PLANNING_RUNNING),
        (S::Running, S::Paused) => Some(INV_RUN_TRANSITION_RUNNING_PAUSED),
        (S::Paused, S::Running) => Some(INV_RUN_TRANSITION_PAUSED_RUNNING),
        (S::Running, S::Interrupted) => Some(INV_RUN_TRANSITION_RUNNING_INTERRUPTED),
        (S::Interrupted, S::Running) => Some(INV_RUN_TRANSITION_INTERRUPTED_RUNNING),
        (S::Interrupted, S::Cancelling) => Some(INV_RUN_TRANSITION_INTERRUPTED_CANCELLING),
        (S::Running, S::Cancelling) => Some(INV_RUN_TRANSITION_RUNNING_CANCELLING),
        (S::Cancelling, S::Cancelled) => Some(INV_RUN_TRANSITION_CANCELLING_CANCELLED),
        (S::Running, S::TimedOut) => Some(INV_RUN_TRANSITION_RUNNING_TIMED_OUT),
        (S::Running, S::Failed) => Some(INV_RUN_TRANSITION_RUNNING_FAILED),
        (S::Running, S::Succeeded) => Some(INV_RUN_TRANSITION_RUNNING_SUCCEEDED),
        _ => None,
    }
}

fn node_is_terminal(state: NodeState) -> bool {
    matches!(
        state,
        NodeState::Success
            | NodeState::Failed
            | NodeState::Skipped
            | NodeState::Cached
            | NodeState::Cancelled
            | NodeState::TimedOut
    )
}

pub fn validate_node_transition(transition: &NodeTransition) -> Result<(), String> {
    if node_is_terminal(transition.from.clone()) && transition.from != transition.to {
        return Err(format!(
            "{} illegal node transition from terminal state: {:?} -> {:?}",
            INV_NODE_TERMINAL_NO_REVERT, transition.from, transition.to
        ));
    }
    use NodeState as S;
    let allowed = matches!(
        (&transition.from, &transition.to),
        (S::Pending, S::Eligible)
            | (S::Eligible, S::Queued)
            | (S::Queued, S::Running)
            | (S::Running, S::Success)
            | (S::Running, S::Failed)
            | (S::Pending, S::Skipped)
            | (S::Eligible, S::Skipped)
            | (S::Queued, S::Skipped)
            | (S::Eligible, S::Cached)
            | (S::Queued, S::Cached)
            | (S::Queued, S::Failed)
            | (S::Pending, S::Cancelled)
            | (S::Eligible, S::Cancelled)
            | (S::Queued, S::Cancelled)
            | (S::Running, S::Cancelled)
            | (S::Pending, S::TimedOut)
            | (S::Eligible, S::TimedOut)
            | (S::Queued, S::TimedOut)
            | (S::Running, S::TimedOut)
    );
    if allowed {
        Ok(())
    } else {
        let inv = node_transition_invariant_id(transition.from.clone(), transition.to.clone())
            .unwrap_or("INV-NODE-TRANSITION-UNKNOWN");
        Err(format!("{inv} illegal node transition: {:?} -> {:?}", transition.from, transition.to))
    }
}

pub fn validate_run_transition(transition: &RunTransition) -> Result<(), String> {
    use RunState as S;
    let allowed = matches!(
        (&transition.from, &transition.to),
        (S::Submitted, S::Planning)
            | (S::Planning, S::Running)
            | (S::Running, S::Paused)
            | (S::Paused, S::Running)
            | (S::Running, S::Interrupted)
            | (S::Interrupted, S::Running)
            | (S::Interrupted, S::Cancelling)
            | (S::Running, S::Cancelling)
            | (S::Cancelling, S::Cancelled)
            | (S::Running, S::TimedOut)
            | (S::Running, S::Failed)
            | (S::Running, S::Succeeded)
    );
    if allowed {
        Ok(())
    } else {
        let inv = run_transition_invariant_id(transition.from.clone(), transition.to.clone())
            .unwrap_or("INV-RUN-TRANSITION-UNKNOWN");
        Err(format!("{inv} illegal run transition: {:?} -> {:?}", transition.from, transition.to))
    }
}

pub fn verify_post_run_state_consistency(
    run_state: RunState,
    node_states: &[NodeState],
    causal_failure_count: usize,
) -> StateConsistencyReport {
    let mut violations = Vec::new();
    if run_state == RunState::Cancelled && !node_states.iter().any(|s| *s == NodeState::Cancelled) {
        violations.push("cancelled run has no cancelled nodes".to_string());
    }
    if run_state == RunState::Failed && causal_failure_count == 0 {
        violations
            .push(format!("{} failed run has no causal failure", INV_RUN_FAILED_CAUSAL_FAILURE));
    }
    if matches!(
        run_state,
        RunState::Succeeded | RunState::Failed | RunState::Cancelled | RunState::TimedOut
    ) {
        let non_terminal = node_states.iter().any(|s| {
            !matches!(
                s,
                NodeState::Success
                    | NodeState::Failed
                    | NodeState::Skipped
                    | NodeState::Cached
                    | NodeState::Cancelled
                    | NodeState::TimedOut
            )
        });
        if non_terminal {
            violations.push("terminal run contains non-terminal node".to_string());
        }
    }
    StateConsistencyReport { valid: violations.is_empty(), violations }
}

pub fn imported_run_distinguishable(snapshot: &RunSnapshot) -> bool {
    snapshot.submission_source == "import" || snapshot.replay_source_run_id.is_some()
}

pub fn terminal_transition_audit_events(
    node_transitions: &[NodeTransition],
    run_transitions: &[RunTransition],
) -> Vec<TransitionAuditEvent> {
    let mut out = Vec::new();
    for transition in node_transitions {
        if node_is_terminal(transition.to.clone()) {
            out.push(TransitionAuditEvent {
                invariant_id: node_transition_invariant_id(
                    transition.from.clone(),
                    transition.to.clone(),
                )
                .unwrap_or("INV-NODE-TERMINAL-UNKNOWN")
                .to_string(),
                entity: "node".to_string(),
                from: format!("{:?}", transition.from),
                to: format!("{:?}", transition.to),
            });
        }
    }
    for transition in run_transitions {
        if matches!(
            transition.to,
            RunState::Succeeded | RunState::Failed | RunState::Cancelled | RunState::TimedOut
        ) {
            out.push(TransitionAuditEvent {
                invariant_id: run_transition_invariant_id(
                    transition.from.clone(),
                    transition.to.clone(),
                )
                .unwrap_or("INV-RUN-TERMINAL-UNKNOWN")
                .to_string(),
                entity: "run".to_string(),
                from: format!("{:?}", transition.from),
                to: format!("{:?}", transition.to),
            });
        }
    }
    out
}
