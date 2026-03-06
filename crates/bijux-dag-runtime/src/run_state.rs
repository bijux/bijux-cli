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
        if !value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
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
    pub selected_nodes: Vec<String>,
    pub dependency_closure_enabled: bool,
    pub replay_source_run_id: Option<RunId>,
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
            total_nodes: self.counts.success + self.counts.failed + self.counts.skipped + self.counts.cached,
            success: self.counts.success,
            failed: self.counts.failed,
            skipped: self.counts.skipped,
            cached: self.counts.cached,
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
        Self {
            max_event_count_before_compaction: 10_000,
            keep_latest_attempts: 5,
        }
    }
}

pub fn validate_node_transition(transition: &NodeTransition) -> Result<(), String> {
    use NodeState as S;
    let allowed = matches!(
        (&transition.from, &transition.to),
        (S::Pending, S::Eligible)
            | (S::Eligible, S::Queued)
            | (S::Queued, S::Running)
            | (S::Running, S::Success)
            | (S::Running, S::Failed)
            | (S::Eligible, S::Skipped)
            | (S::Queued, S::Skipped)
            | (S::Eligible, S::Cached)
            | (S::Queued, S::Cached)
            | (S::Running, S::Cancelled)
    );
    if allowed {
        Ok(())
    } else {
        Err(format!(
            "illegal node transition: {:?} -> {:?}",
            transition.from, transition.to
        ))
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
            | (S::Running, S::Failed)
            | (S::Running, S::Succeeded)
    );
    if allowed {
        Ok(())
    } else {
        Err(format!(
            "illegal run transition: {:?} -> {:?}",
            transition.from, transition.to
        ))
    }
}
