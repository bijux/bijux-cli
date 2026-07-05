use crate::run_state::{NodeState, RunState, RunSummaryV2};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunPauseMode {
    PauseQueuedOnly,
    PauseQueuedAndReady,
    PauseAllNewDispatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunPausePolicy {
    pub mode: RunPauseMode,
    pub preserve_running_nodes: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeControlMode {
    Unblocked,
    Paused,
    BlockedByOperator { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersistedRunSnapshotRef {
    pub run_id: String,
    pub snapshot_path: String,
    pub persisted_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerRecoveryAction {
    Reattach,
    Requeue,
    MarkFailed,
    Quarantine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerRecoveryRule {
    pub orphaned_node_state: NodeState,
    pub action: SchedulerRecoveryAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeHeartbeatPolicy {
    pub expected_interval_ms: u64,
    pub grace_missed_heartbeats: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StuckRunPolicy {
    pub max_without_progress_ms: u64,
    pub max_without_heartbeat_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InterruptionClass {
    CleanShutdown,
    ProcessCrash,
    WorkerLoss,
    BackendLoss,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResumePolicy {
    Reattach,
    VerifyAndContinue,
    RerunIncompleteNodes,
    FailSafeStop,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperatorRetryPolicy {
    pub max_manual_attempts: u32,
    pub require_reason: bool,
    pub requires_audit_record: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualInterventionRecord {
    pub run_id: String,
    pub node_id: Option<String>,
    pub operator: String,
    pub action: String,
    pub reason: String,
    pub recorded_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointResumeContract {
    pub node_id: String,
    pub checkpoint_id: String,
    pub supports_resume: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BranchRecoveryMode {
    FailFast,
    ContinueHealthyBranches,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DegradedExecutionPolicy {
    pub allow_without_remote_tracing: bool,
    pub allow_without_remote_metrics_sink: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunRepairOutcome {
    pub manifest_valid: bool,
    pub index_valid: bool,
    pub repaired_manifest: bool,
    pub repaired_index: bool,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConsistencyCheckReport {
    pub summary_matches_node_states: bool,
    pub all_success_nodes_have_artifacts: bool,
    pub mismatches: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunQuarantineRecord {
    pub run_id: String,
    pub reason: String,
    pub evidence: Vec<String>,
    pub recorded_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResilientLogRecord {
    pub node_id: String,
    pub primary_log_path: String,
    pub persisted_fallback_path: Option<String>,
    pub durable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecoveryFaultBoundary {
    Planner,
    Scheduler,
    Worker,
    ArtifactStore,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryFaultInjection {
    pub boundary: RecoveryFaultBoundary,
    pub fault_code: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoverySimulationScenario {
    pub scenario_id: String,
    pub injections: Vec<RecoveryFaultInjection>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecoveryAcceptanceSuite {
    pub suite_id: String,
    pub required_scenarios: Vec<String>,
    pub strict: bool,
}

pub fn evaluate_pause_state(
    policy: &RunPausePolicy,
    queued_count: usize,
    ready_count: usize,
    running_count: usize,
) -> BTreeMap<&'static str, bool> {
    let mut result = BTreeMap::new();
    result.insert("freeze_dispatch", matches!(policy.mode, RunPauseMode::PauseAllNewDispatch));
    result.insert(
        "freeze_ready_queue",
        matches!(
            policy.mode,
            RunPauseMode::PauseQueuedAndReady | RunPauseMode::PauseAllNewDispatch
        ),
    );
    result.insert("has_queued", queued_count > 0);
    result.insert("has_ready", ready_count > 0);
    result.insert("has_running", running_count > 0);
    result.insert("preserve_running_nodes", policy.preserve_running_nodes);
    result
}

pub fn detect_stuck_run(
    now_unix_ms: u128,
    last_progress_unix_ms: u128,
    last_heartbeat_unix_ms: u128,
    policy: &StuckRunPolicy,
) -> bool {
    let progress_gap = now_unix_ms.saturating_sub(last_progress_unix_ms);
    let heartbeat_gap = now_unix_ms.saturating_sub(last_heartbeat_unix_ms);
    progress_gap > policy.max_without_progress_ms as u128
        || heartbeat_gap > policy.max_without_heartbeat_ms as u128
}

pub fn reconcile_orphaned_node(rule: &SchedulerRecoveryRule) -> NodeState {
    match rule.action {
        SchedulerRecoveryAction::Reattach | SchedulerRecoveryAction::Requeue => NodeState::Queued,
        SchedulerRecoveryAction::MarkFailed => NodeState::Failed,
        SchedulerRecoveryAction::Quarantine => NodeState::Cancelled,
    }
}

pub fn validate_and_repair_run_metadata(
    manifest_exists: bool,
    index_exists: bool,
    allow_repair: bool,
) -> RunRepairOutcome {
    let mut notes = Vec::new();
    let mut repaired_manifest = false;
    let mut repaired_index = false;
    if !manifest_exists {
        notes.push("manifest missing".to_string());
        if allow_repair {
            repaired_manifest = true;
            notes.push("manifest synthesized from node records".to_string());
        }
    }
    if !index_exists {
        notes.push("metadata index missing".to_string());
        if allow_repair {
            repaired_index = true;
            notes.push("metadata index rebuilt from artifacts".to_string());
        }
    }
    RunRepairOutcome {
        manifest_valid: manifest_exists || repaired_manifest,
        index_valid: index_exists || repaired_index,
        repaired_manifest,
        repaired_index,
        notes,
    }
}

pub fn check_run_consistency(
    node_states: &[(String, NodeState)],
    artifact_nodes: &[String],
    summary: &RunSummaryV2,
) -> ConsistencyCheckReport {
    let mut mismatches = Vec::new();
    let mut success_count = 0u32;
    let mut failed_count = 0u32;
    let mut skipped_count = 0u32;
    let mut cached_count = 0u32;
    for (_, state) in node_states {
        match state {
            NodeState::Success => success_count += 1,
            NodeState::Failed => failed_count += 1,
            NodeState::Skipped => skipped_count += 1,
            NodeState::Cached => cached_count += 1,
            _ => {}
        }
    }
    if summary.counts.success != success_count {
        mismatches.push("summary success count mismatch".to_string());
    }
    if summary.counts.failed != failed_count {
        mismatches.push("summary failed count mismatch".to_string());
    }
    if summary.counts.skipped != skipped_count {
        mismatches.push("summary skipped count mismatch".to_string());
    }
    if summary.counts.cached != cached_count {
        mismatches.push("summary cached count mismatch".to_string());
    }
    let artifact_set = artifact_nodes.iter().cloned().collect::<std::collections::BTreeSet<_>>();
    let all_success_nodes_have_artifacts = node_states
        .iter()
        .filter(|(_, state)| *state == NodeState::Success)
        .all(|(node_id, _)| artifact_set.contains(node_id));
    if !all_success_nodes_have_artifacts {
        mismatches.push("some successful nodes are missing artifacts".to_string());
    }
    ConsistencyCheckReport {
        summary_matches_node_states: mismatches.iter().all(|msg| !msg.contains("summary")),
        all_success_nodes_have_artifacts,
        mismatches,
    }
}

pub fn should_quarantine_run(
    run_state: &RunState,
    consistency: &ConsistencyCheckReport,
) -> Option<String> {
    if matches!(run_state, RunState::Cancelled | RunState::Failed | RunState::TimedOut)
        && (!consistency.summary_matches_node_states
            || !consistency.all_success_nodes_have_artifacts)
    {
        return Some("inconsistent terminal run metadata".to_string());
    }
    None
}
