use crate::execution_plan::ExecutionPlan;
use crate::scheduler_workload::{
    priority_class_weight, ScheduleOverrideAction, ScheduleOverrideRecord, ScheduleOverrideState,
    ScheduleOverrideStatus, StarvationPreventionPolicy, WeightedPriorityPolicy,
};
use crate::RuntimeConfig;
use bijux_dag_core::{materialize_graph_input_value, resources, Graph, GraphInputSpec};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerPolicy {
    pub max_parallelism: usize,
    pub cpu_budget: Option<u32>,
    pub memory_budget_mb: Option<u32>,
    pub gpu_device_budget: Option<u32>,
    #[serde(default)]
    pub named_resource_capacities: BTreeMap<String, u32>,
    pub fairness: SchedulerFairness,
    pub queue_isolation: QueueIsolationPolicy,
    pub bounded_executor_capacity: usize,
    pub prefer_throughput_scheduler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DependencyTriggerCondition {
    Success,
    Failure,
    AnyTerminal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerSpec {
    Manual,
    Cron { expression: String, timezone: String },
    Event { event_type: String, source: String },
    Dependency { dag_name: String, on_status: DependencyTriggerCondition },
    Signal { signal_name: String, payload_schema: Option<String> },
    Backfill(BackfillRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueIdentity {
    #[serde(default = "default_queue_name")]
    pub queue_name: String,
    pub tenant: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PriorityClass {
    Critical,
    High,
    Standard,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConcurrencyPolicyLayers {
    pub per_dag: Option<u32>,
    pub per_queue: Option<u32>,
    pub per_tenant: Option<u32>,
    pub per_node_group: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatchUpPolicy {
    pub enabled: bool,
    pub max_catch_up_runs: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackfillFailurePolicy {
    Continue,
    Pause,
    Cancel,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillRequest {
    pub window_start_unix_ms: u128,
    pub window_end_unix_ms: u128,
    pub partition_by: Option<String>,
    #[serde(default)]
    pub partition_keys: Vec<String>,
    pub max_parallelism: u32,
    pub failure_policy: BackfillFailurePolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackfillLifecycleStatus {
    Active,
    Paused,
    Cancelled,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackfillRunStatus {
    Queued,
    Submitted,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillRunRecord {
    pub requested_unix_ms: u128,
    pub partition_key: Option<String>,
    pub dedupe_key: String,
    pub run_id: String,
    #[serde(default = "default_backfill_attempt")]
    pub attempt: u32,
    #[serde(default)]
    pub previous_run_ids: Vec<String>,
    pub status: BackfillRunStatus,
    pub updated_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillAuditRecord {
    pub at_unix_ms: u128,
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackfillOperation {
    pub backfill_id: String,
    pub schedule_id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
    #[serde(default)]
    pub input_contract: BTreeMap<String, GraphInputSpec>,
    #[serde(default)]
    pub input_bindings: BTreeMap<String, ScheduleInputSource>,
    pub queue: QueueIdentity,
    pub priority: PriorityClass,
    pub request: BackfillRequest,
    pub lifecycle: BackfillLifecycleStatus,
    pub lifecycle_reason: Option<String>,
    pub updated_unix_ms: u128,
    #[serde(default)]
    pub audit: Vec<BackfillAuditRecord>,
    #[serde(default)]
    pub runs: Vec<BackfillRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillStatusUpdate {
    pub run_id: String,
    pub status: BackfillRunStatus,
    pub updated_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct BackfillStatusUpdateBatch {
    #[serde(default)]
    pub updates: Vec<BackfillStatusUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillAdvanceRequest {
    pub now_unix_ms: u128,
    pub pending_live_runs: usize,
    pub throttling_policy: crate::scheduler_workload::BackfillThrottlingPolicy,
    #[serde(default)]
    pub status_updates: Vec<BackfillStatusUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackfillAdvanceReport {
    pub operation: BackfillOperation,
    #[serde(default)]
    pub dispatched_requests: Vec<ExecutionSubmissionRequest>,
    pub active_runs: usize,
    pub queued_runs: usize,
    pub allowed_dispatches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillPartitionSummary {
    pub requested_unix_ms: u128,
    pub partition_key: Option<String>,
    pub status: BackfillRunStatus,
    pub attempt: u32,
    pub run_id: String,
    #[serde(default)]
    pub previous_run_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillOperationSummary {
    pub backfill_id: String,
    pub schedule_id: String,
    pub dag_name: String,
    pub lifecycle: BackfillLifecycleStatus,
    pub lifecycle_reason: Option<String>,
    pub total_runs: usize,
    pub queued_runs: usize,
    pub submitted_runs: usize,
    pub running_runs: usize,
    pub completed_runs: usize,
    pub failed_runs: usize,
    pub cancelled_runs: usize,
    pub total_retry_attempts: u32,
    #[serde(default)]
    pub partitions: Vec<BackfillPartitionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum ScheduleInputSource {
    RequestedUnixMs,
    ManualArgument { key: String },
    EventPayload { pointer: Option<String> },
    SignalPayload { pointer: Option<String> },
    DependencyUpstreamRunId,
    DependencyStatus,
    BackfillWindowStartUnixMs,
    BackfillWindowEndUnixMs,
    BackfillPartitionKey,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScheduleDefinition {
    pub id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
    #[serde(default)]
    pub input_contract: BTreeMap<String, GraphInputSpec>,
    #[serde(default)]
    pub input_bindings: BTreeMap<String, ScheduleInputSource>,
    pub trigger: TriggerSpec,
    pub queue: QueueIdentity,
    pub priority: PriorityClass,
    pub concurrency: ConcurrencyPolicyLayers,
    pub catch_up: CatchUpPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ScheduleRegistry {
    pub definitions: Vec<ScheduleDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ScheduleSubmissionStatus {
    Pending,
    Running,
    Completed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduledSubmission {
    pub schedule_id: String,
    pub run_id: String,
    pub created_unix_ms: u128,
    pub status: ScheduleSubmissionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleAuditRecord {
    pub schedule_id: String,
    pub evaluated_unix_ms: u128,
    pub decision: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleEventLineage {
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub occurred_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleDryRunPreview {
    pub schedule_id: String,
    pub next_fire_unix_ms: Option<u128>,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionSubmissionRequest {
    pub schedule_id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
    pub queue: QueueIdentity,
    pub priority: PriorityClass,
    #[serde(default)]
    pub graph_inputs: BTreeMap<String, Value>,
    pub requested_unix_ms: u128,
    pub run_id: String,
    pub trigger_kind: SubmissionTriggerKind,
    pub dedupe_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_lineage: Option<ScheduleEventLineage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SubmissionTriggerKind {
    Manual,
    Cron,
    Event,
    Dependency,
    Signal,
    Backfill,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManualSubmissionRequest {
    pub request_id: String,
    pub schedule_id: String,
    pub requested_unix_ms: u128,
    #[serde(default)]
    pub arguments: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub occurred_unix_ms: u128,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DependencyCompletionRecord {
    pub upstream_run_id: String,
    pub dag_name: String,
    pub status: String,
    pub finished_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignalRecord {
    pub signal_id: String,
    pub signal_name: String,
    pub occurred_unix_ms: u128,
    pub payload: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleEvaluationInputs {
    pub now_unix_ms: u128,
    #[serde(default)]
    pub manual_requests: Vec<ManualSubmissionRequest>,
    #[serde(default)]
    pub events: Vec<ScheduleEventRecord>,
    #[serde(default)]
    pub dependencies: Vec<DependencyCompletionRecord>,
    #[serde(default)]
    pub signals: Vec<SignalRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleSubmissionLedgerEntry {
    pub schedule_id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
    #[serde(default = "default_queue_identity")]
    pub queue: QueueIdentity,
    #[serde(default = "default_priority_class")]
    pub priority: PriorityClass,
    #[serde(default)]
    pub graph_inputs: BTreeMap<String, Value>,
    pub requested_unix_ms: u128,
    pub created_unix_ms: u128,
    pub run_id: String,
    pub trigger_kind: SubmissionTriggerKind,
    pub dedupe_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub event_lineage: Option<ScheduleEventLineage>,
    pub status: ScheduleSubmissionStatus,
    #[serde(default)]
    pub starvation_ticks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleSubmissionLedger {
    #[serde(default)]
    pub entries: Vec<ScheduleSubmissionLedgerEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleSubmissionStatusUpdate {
    pub run_id: String,
    pub status: ScheduleSubmissionStatus,
    pub updated_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleSubmissionStatusUpdateBatch {
    #[serde(default)]
    pub updates: Vec<ScheduleSubmissionStatusUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleQueueRunRecord {
    pub schedule_id: String,
    pub dag_name: String,
    pub run_id: String,
    pub priority: PriorityClass,
    pub status: ScheduleSubmissionStatus,
    pub starvation_ticks: u32,
    pub requested_unix_ms: u128,
    pub created_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulePriorityDispatchPolicy {
    #[serde(default)]
    pub weights: WeightedPriorityPolicy,
    pub starvation: StarvationPreventionPolicy,
}

impl Default for SchedulePriorityDispatchPolicy {
    fn default() -> Self {
        Self {
            weights: WeightedPriorityPolicy::default(),
            starvation: StarvationPreventionPolicy {
                max_ticks_without_dispatch: 3,
                priority_boost_after_ticks: 1,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleDispatchRecord {
    pub schedule_id: String,
    pub run_id: String,
    pub queue: QueueIdentity,
    pub priority: PriorityClass,
    pub starvation_ticks: u32,
    pub effective_weight: u32,
    pub starvation_guard_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleDispatchReport {
    #[serde(default)]
    pub dispatched_runs: Vec<ScheduleDispatchRecord>,
    #[serde(default)]
    pub deferred_runs: Vec<ScheduleDispatchRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleQueueTenantState {
    pub tenant: String,
    pub per_tenant_cap: Option<u32>,
    pub active_runs: usize,
    pub available_slots: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleQueueStateEntry {
    pub queue_name: String,
    pub per_queue_cap: u32,
    pub active_runs: usize,
    pub available_slots: usize,
    #[serde(default)]
    pub tenants: Vec<ScheduleQueueTenantState>,
    #[serde(default)]
    pub runs: Vec<ScheduleQueueRunRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleQueueState {
    #[serde(default)]
    pub queues: Vec<ScheduleQueueStateEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleEvaluationReport {
    #[serde(default)]
    pub generated_requests: Vec<ExecutionSubmissionRequest>,
    #[serde(default)]
    pub recorded_submissions: Vec<ScheduleSubmissionLedgerEntry>,
    #[serde(default)]
    pub duplicate_suppressions: Vec<ScheduleAuditRecord>,
    #[serde(default)]
    pub paused_suppressions: Vec<ScheduleAuditRecord>,
    #[serde(default)]
    pub queue_suppressions: Vec<ScheduleAuditRecord>,
    #[serde(default)]
    pub audits: Vec<ScheduleAuditRecord>,
}

impl Default for SchedulerPolicy {
    fn default() -> Self {
        Self {
            max_parallelism: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            named_resource_capacities: BTreeMap::new(),
            fairness: SchedulerFairness::Deterministic,
            queue_isolation: QueueIsolationPolicy::SingleQueue,
            bounded_executor_capacity: 64,
            prefer_throughput_scheduler: false,
        }
    }
}

fn default_queue_name() -> String {
    "default".to_string()
}

fn default_queue_identity() -> QueueIdentity {
    QueueIdentity { queue_name: default_queue_name(), tenant: None }
}

fn default_priority_class() -> PriorityClass {
    PriorityClass::Standard
}

fn default_backfill_attempt() -> u32 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SchedulerFairness {
    Deterministic,
    ThroughputPreferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueueIsolationPolicy {
    SingleQueue,
    GroupIsolated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FailurePropagationMode {
    FailFast,
    IsolateBranch,
    ContinueIndependent,
    QuorumLikeFuture,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDecision {
    pub ready_candidates: Vec<String>,
    pub batch: Vec<String>,
    pub blocked_by_budget: Vec<String>,
    pub blocked_reasons: BTreeMap<String, String>,
    pub decision_reason: String,
    pub tie_break_reason: Option<String>,
    pub timed_out: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub loop_index: u64,
    pub ready_queue_depth: usize,
    pub ready_queue: Vec<String>,
    pub inflight: Vec<String>,
    pub scheduled: Vec<String>,
    pub blocked_by_budget: Vec<String>,
    pub blocked_reasons: BTreeMap<String, String>,
    pub completed_statuses: BTreeMap<String, String>,
    #[serde(default = "default_checkpoint_decision_reason")]
    pub decision_reason: String,
    pub failure_propagation_mode: String,
    pub dependency_closure_enabled: bool,
    pub generated_unix_ms: u128,
}

fn default_checkpoint_decision_reason() -> String {
    "not_recorded".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerUnit {
    Node,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerModel {
    EventDriven,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerPriorityModel {
    StaticAbsent,
    StaticHints,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadyTieBreak {
    LexicographicNodeId,
    PriorityCpuMemoryFitThenNodeId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerContractProfile {
    pub canonical_unit: SchedulerUnit,
    pub model: SchedulerModel,
    pub priority_model: SchedulerPriorityModel,
    pub ready_tie_break: ReadyTieBreak,
}

pub fn scheduler_contract_profile() -> SchedulerContractProfile {
    SchedulerContractProfile {
        canonical_unit: SchedulerUnit::Node,
        model: SchedulerModel::EventDriven,
        priority_model: SchedulerPriorityModel::StaticHints,
        ready_tie_break: ReadyTieBreak::PriorityCpuMemoryFitThenNodeId,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SchedulerEventKind {
    NodeReady,
    NodeScheduled,
    NodeBlockedByBudget,
    NodeRetryQueued,
    NodeRetryRequeued,
    NodeCached,
    NodeSkipped,
    NodeFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerEvent {
    pub sequence: u64,
    pub kind: SchedulerEventKind,
    pub node_id: String,
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerState {
    indegree: BTreeMap<String, usize>,
    adjacency: BTreeMap<String, Vec<String>>,
    ready: ReadyQueue,
    retry_queue: BTreeSet<String>,
    completion_by_node: BTreeMap<String, String>,
    events: Vec<SchedulerEvent>,
    next_seq: u64,
}

impl SchedulerState {
    pub fn from_plan(plan: &ExecutionPlan) -> Self {
        let mut indegree = BTreeMap::new();
        for (k, v) in &plan.indegree {
            indegree.insert(k.clone(), *v);
        }
        let mut adjacency = BTreeMap::new();
        for (k, v) in &plan.adj {
            adjacency.insert(k.clone(), v.clone());
        }
        Self {
            ready: ReadyQueue::from_indegree(&plan.indegree),
            indegree,
            adjacency,
            retry_queue: BTreeSet::new(),
            completion_by_node: BTreeMap::new(),
            events: Vec::new(),
            next_seq: 1,
        }
    }

    pub fn ready_snapshot(&self) -> Vec<String> {
        self.ready.snapshot_sorted()
    }

    pub fn retry_snapshot(&self) -> Vec<String> {
        self.retry_queue.iter().cloned().collect()
    }

    pub fn events(&self) -> &[SchedulerEvent] {
        &self.events
    }

    pub fn complete_success(&mut self, node_id: &str) -> Vec<String> {
        self.mark_completion(node_id, "success")
    }

    pub fn complete_cached(&mut self, node_id: &str) -> Vec<String> {
        self.mark_event(SchedulerEventKind::NodeCached, node_id, None);
        self.mark_completion(node_id, "cached")
    }

    pub fn complete_skipped(&mut self, node_id: &str) -> Vec<String> {
        self.mark_event(SchedulerEventKind::NodeSkipped, node_id, None);
        self.mark_completion(node_id, "skipped")
    }

    pub fn complete_failed(&mut self, node_id: &str, mode: FailurePropagationMode) -> Vec<String> {
        self.mark_event(
            SchedulerEventKind::NodeFailed,
            node_id,
            Some(format!("mode={}", failure_mode_name(&mode))),
        );
        let _ = self.ready.take(node_id);
        self.retry_queue.remove(node_id);
        self.completion_by_node.insert(node_id.to_string(), "failed".to_string());
        if failure_allows_downstream_readiness(mode) {
            self.release_downstream(node_id)
        } else {
            Vec::new()
        }
    }

    pub fn queue_retry(&mut self, node_id: &str) {
        if self.retry_queue.insert(node_id.to_string()) {
            self.mark_event(SchedulerEventKind::NodeRetryQueued, node_id, None);
        }
    }

    pub fn requeue_retries(&mut self) {
        let pending = self.retry_queue.iter().cloned().collect::<Vec<_>>();
        for node_id in pending {
            self.retry_queue.remove(&node_id);
            self.ready.insert(node_id.clone());
            self.mark_event(SchedulerEventKind::NodeRetryRequeued, &node_id, None);
        }
    }

    pub fn mark_scheduled(&mut self, node_id: &str) {
        self.mark_event(SchedulerEventKind::NodeScheduled, node_id, None);
    }

    fn mark_completion(&mut self, node_id: &str, status: &str) -> Vec<String> {
        let _ = self.ready.take(node_id);
        self.retry_queue.remove(node_id);
        self.completion_by_node.insert(node_id.to_string(), status.to_string());
        self.release_downstream(node_id)
    }

    fn release_downstream(&mut self, node_id: &str) -> Vec<String> {
        let mut newly_ready = Vec::new();
        if let Some(children) = self.adjacency.get(node_id).cloned() {
            for child in children {
                if let Some(counter) = self.indegree.get_mut(&child) {
                    *counter = counter.saturating_sub(1);
                    if *counter == 0 {
                        self.ready.insert(child.clone());
                        self.mark_event(SchedulerEventKind::NodeReady, &child, None);
                        newly_ready.push(child);
                    }
                }
            }
        }
        newly_ready.sort();
        newly_ready.dedup();
        newly_ready
    }

    fn mark_event(&mut self, kind: SchedulerEventKind, node_id: &str, detail: Option<String>) {
        self.events.push(SchedulerEvent {
            sequence: self.next_seq,
            kind,
            node_id: node_id.to_string(),
            detail,
        });
        self.next_seq += 1;
    }
}

pub trait SchedulerEventHook: Send + Sync {
    fn on_node_eligible(&self, _node_id: &str) {}
    fn on_node_blocked_by_budget(&self, _node_id: &str) {}
    fn on_node_scheduled(&self, _node_id: &str) {}
}

#[derive(Default)]
pub struct NoopSchedulerEventHook;
impl SchedulerEventHook for NoopSchedulerEventHook {}

pub trait Scheduler {
    fn next_batch(
        &mut self,
        graph: &Graph,
        ready_queue: &mut ReadyQueue,
        options: &RuntimeConfig,
        started: Instant,
        cancellation_requested: bool,
    ) -> ScheduleDecision;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReadyQueue {
    ordered: BTreeSet<String>,
    queue: VecDeque<String>,
}

impl ReadyQueue {
    pub fn from_indegree(indegree: &HashMap<String, usize>) -> Self {
        let mut ordered = BTreeSet::new();
        let mut queue = VecDeque::new();
        let mut roots: Vec<_> = indegree
            .iter()
            .filter_map(|(id, &deg)| if deg == 0 { Some(id.clone()) } else { None })
            .collect();
        roots.sort();
        for id in roots {
            ordered.insert(id.clone());
            queue.push_back(id);
        }
        Self { ordered, queue }
    }

    pub fn empty() -> Self {
        Self { ordered: BTreeSet::new(), queue: VecDeque::new() }
    }

    pub fn insert(&mut self, id: String) {
        if self.ordered.insert(id.clone()) {
            self.queue.push_back(id);
        }
    }

    pub fn pop_deterministic(&mut self) -> Option<String> {
        let id = self.ordered.iter().next().cloned()?;
        self.ordered.remove(&id);
        self.queue.retain(|v| v != &id);
        Some(id)
    }

    pub fn pop_fifo(&mut self) -> Option<String> {
        while let Some(id) = self.queue.pop_front() {
            if self.ordered.remove(&id) {
                return Some(id);
            }
        }
        None
    }

    pub fn take(&mut self, id: &str) -> Option<String> {
        if !self.ordered.remove(id) {
            return None;
        }
        self.queue.retain(|queued| queued != id);
        Some(id.to_string())
    }

    pub fn is_empty(&self) -> bool {
        self.ordered.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ordered.len()
    }

    pub fn snapshot_sorted(&self) -> Vec<String> {
        self.ordered.iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
pub struct DependencyCounter {
    indegree: HashMap<String, usize>,
    adj: HashMap<String, Vec<String>>,
}

impl DependencyCounter {
    pub fn from_plan(plan: &ExecutionPlan) -> Self {
        Self { indegree: plan.indegree.clone(), adj: plan.adj.clone() }
    }

    pub fn indegree_map(&self) -> &HashMap<String, usize> {
        &self.indegree
    }

    pub fn mark_completed(&mut self, node_id: &str) -> Vec<String> {
        let mut newly_ready = Vec::new();
        if let Some(neighbors) = self.adj.get(node_id) {
            for n in neighbors {
                if let Some(d) = self.indegree.get_mut(n) {
                    *d = d.saturating_sub(1);
                    if *d == 0 {
                        newly_ready.push(n.clone());
                    }
                }
            }
        }
        newly_ready.sort();
        newly_ready
    }
}

pub struct DeterministicScheduler;

#[derive(Debug, Clone)]
struct ReadyCandidate {
    node_id: String,
    priority: u8,
    cpu: u32,
    memory_mb: u32,
    gpu_devices: u32,
    named_resources: BTreeMap<String, u32>,
}

fn preflight_decision(
    options: &RuntimeConfig,
    started: Instant,
    cancellation_requested: bool,
) -> Option<ScheduleDecision> {
    if cancellation_requested {
        return Some(ScheduleDecision {
            ready_candidates: Vec::new(),
            batch: Vec::new(),
            blocked_by_budget: Vec::new(),
            blocked_reasons: BTreeMap::new(),
            decision_reason: "cancelled".to_string(),
            tie_break_reason: None,
            timed_out: false,
            cancelled: true,
        });
    }
    if let Some(limit_ms) = options.run_timeout_ms {
        if started.elapsed() >= Duration::from_millis(limit_ms) {
            return Some(ScheduleDecision {
                ready_candidates: Vec::new(),
                batch: Vec::new(),
                blocked_by_budget: Vec::new(),
                blocked_reasons: BTreeMap::new(),
                decision_reason: "run_timeout".to_string(),
                tie_break_reason: None,
                timed_out: true,
                cancelled: false,
            });
        }
    }
    None
}

impl Scheduler for DeterministicScheduler {
    fn next_batch(
        &mut self,
        graph: &Graph,
        ready_queue: &mut ReadyQueue,
        options: &RuntimeConfig,
        started: Instant,
        cancellation_requested: bool,
    ) -> ScheduleDecision {
        if let Some(decision) = preflight_decision(options, started, cancellation_requested) {
            return decision;
        }
        let cpu_budget = options
            .scheduler_policy
            .cpu_budget
            .or(options.cpu_budget)
            .unwrap_or(options.jobs.max(1) as u32);
        let memory_budget_mb =
            options.scheduler_policy.memory_budget_mb.or(options.memory_budget_mb);
        let gpu_device_budget =
            options.scheduler_policy.gpu_device_budget.or(options.gpu_device_budget);
        let named_resource_capacities = effective_named_resource_capacities(options);
        let mut used_cpu = 0u32;
        let mut used_memory_mb = 0u32;
        let mut used_gpu_devices = 0u32;
        let mut used_named_resources = BTreeMap::<String, u32>::new();
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        let mut blocked_reasons = BTreeMap::new();
        let mut candidates = ready_queue
            .snapshot_sorted()
            .into_iter()
            .map(|node_id| ReadyCandidate {
                priority: node_priority(graph, &node_id),
                cpu: node_cpu(graph, &node_id),
                memory_mb: node_memory_mb(graph, &node_id),
                gpu_devices: node_gpu_devices(graph, &node_id),
                named_resources: node_named_resources(graph, &node_id),
                node_id,
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.cpu.cmp(&b.cpu))
                .then_with(|| a.memory_mb.cmp(&b.memory_mb))
                .then_with(|| a.gpu_devices.cmp(&b.gpu_devices))
                .then_with(|| a.node_id.cmp(&b.node_id))
        });
        let ready_candidates =
            candidates.iter().map(|candidate| candidate.node_id.clone()).collect::<Vec<_>>();
        for candidate in &candidates {
            if batch.len()
                >= options.scheduler_policy.max_parallelism.max(1).min(options.jobs.max(1))
            {
                blocked.push(candidate.node_id.clone());
                blocked_reasons
                    .insert(candidate.node_id.clone(), "blocked_by_parallelism".to_string());
                continue;
            }
            if used_cpu + candidate.cpu > cpu_budget {
                blocked.push(candidate.node_id.clone());
                blocked_reasons.insert(candidate.node_id.clone(), "blocked_by_cpu".to_string());
                continue;
            }
            if memory_budget_mb.is_some_and(|budget| used_memory_mb + candidate.memory_mb > budget)
            {
                blocked.push(candidate.node_id.clone());
                blocked_reasons.insert(candidate.node_id.clone(), "blocked_by_memory".to_string());
                continue;
            }
            if gpu_device_budget
                .is_some_and(|budget| used_gpu_devices + candidate.gpu_devices > budget)
            {
                blocked.push(candidate.node_id.clone());
                blocked_reasons.insert(candidate.node_id.clone(), "blocked_by_gpu".to_string());
                continue;
            }
            if let Some(resource_name) = first_exhausted_named_resource(
                &named_resource_capacities,
                &used_named_resources,
                &candidate.named_resources,
            ) {
                blocked.push(candidate.node_id.clone());
                blocked_reasons.insert(
                    candidate.node_id.clone(),
                    format!("blocked_by_named_resource:{resource_name}"),
                );
                continue;
            }
            used_cpu += candidate.cpu;
            used_memory_mb += candidate.memory_mb;
            used_gpu_devices += candidate.gpu_devices;
            reserve_named_resources(&mut used_named_resources, &candidate.named_resources);
            let _ = ready_queue.take(&candidate.node_id);
            batch.push(candidate.node_id.clone());
        }
        let mut decision_reason = "ready_batch".to_string();
        if batch.is_empty() {
            if let Some(candidate) = candidates.first() {
                let _ = ready_queue.take(&candidate.node_id);
                batch.push(candidate.node_id.clone());
                decision_reason = "forced_single_progress".to_string();
            }
        }
        ScheduleDecision {
            ready_candidates,
            batch,
            blocked_by_budget: blocked,
            blocked_reasons,
            decision_reason,
            tie_break_reason: Some("priority_cpu_memory_fit_then_node_id".to_string()),
            timed_out: false,
            cancelled: false,
        }
    }
}

pub struct ThroughputScheduler;

impl Scheduler for ThroughputScheduler {
    fn next_batch(
        &mut self,
        graph: &Graph,
        ready_queue: &mut ReadyQueue,
        options: &RuntimeConfig,
        started: Instant,
        cancellation_requested: bool,
    ) -> ScheduleDecision {
        if let Some(decision) = preflight_decision(options, started, cancellation_requested) {
            return decision;
        }
        let cpu_budget = options
            .scheduler_policy
            .cpu_budget
            .or(options.cpu_budget)
            .unwrap_or(options.jobs.max(1) as u32);
        let memory_budget_mb =
            options.scheduler_policy.memory_budget_mb.or(options.memory_budget_mb);
        let gpu_device_budget =
            options.scheduler_policy.gpu_device_budget.or(options.gpu_device_budget);
        let named_resource_capacities = effective_named_resource_capacities(options);
        let mut used_cpu = 0u32;
        let mut used_memory_mb = 0u32;
        let mut used_gpu_devices = 0u32;
        let mut used_named_resources = BTreeMap::<String, u32>::new();
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        let mut blocked_reasons = BTreeMap::new();
        let ready_candidates = ready_queue.snapshot_sorted();
        while !ready_queue.is_empty()
            && batch.len()
                < options.scheduler_policy.max_parallelism.max(1).min(options.jobs.max(1))
        {
            let id = match ready_queue.pop_fifo() {
                Some(v) => v,
                None => break,
            };
            let cpu = node_cpu(graph, &id);
            let memory_mb = node_memory_mb(graph, &id);
            let gpu_devices = node_gpu_devices(graph, &id);
            let named_resources = node_named_resources(graph, &id);
            if used_cpu + cpu > cpu_budget {
                blocked_reasons.insert(id.clone(), "blocked_by_cpu".to_string());
                blocked.push(id);
                continue;
            }
            if memory_budget_mb.is_some_and(|budget| used_memory_mb + memory_mb > budget) {
                blocked_reasons.insert(id.clone(), "blocked_by_memory".to_string());
                blocked.push(id);
                continue;
            }
            if gpu_device_budget.is_some_and(|budget| used_gpu_devices + gpu_devices > budget) {
                blocked_reasons.insert(id.clone(), "blocked_by_gpu".to_string());
                blocked.push(id);
                continue;
            }
            if let Some(resource_name) = first_exhausted_named_resource(
                &named_resource_capacities,
                &used_named_resources,
                &named_resources,
            ) {
                blocked_reasons
                    .insert(id.clone(), format!("blocked_by_named_resource:{resource_name}"));
                blocked.push(id);
                continue;
            }
            used_cpu += cpu;
            used_memory_mb += memory_mb;
            used_gpu_devices += gpu_devices;
            reserve_named_resources(&mut used_named_resources, &named_resources);
            batch.push(id);
        }
        for id in blocked.clone() {
            ready_queue.insert(id);
        }
        ScheduleDecision {
            ready_candidates,
            batch,
            blocked_by_budget: blocked,
            blocked_reasons,
            decision_reason: "fifo_throughput".to_string(),
            tie_break_reason: Some("fifo".to_string()),
            timed_out: false,
            cancelled: false,
        }
    }
}

pub fn build_scheduler(policy: &SchedulerPolicy) -> Box<dyn Scheduler + Send> {
    if policy.prefer_throughput_scheduler {
        Box::new(ThroughputScheduler)
    } else {
        Box::new(DeterministicScheduler)
    }
}

pub fn failure_allows_downstream_readiness(mode: FailurePropagationMode) -> bool {
    !matches!(mode, FailurePropagationMode::FailFast)
}

pub fn failure_mode_name(mode: &FailurePropagationMode) -> &'static str {
    match mode {
        FailurePropagationMode::FailFast => "fail_fast",
        FailurePropagationMode::IsolateBranch => "isolate_branch",
        FailurePropagationMode::ContinueIndependent => "continue_independent",
        FailurePropagationMode::QuorumLikeFuture => "quorum_like_future",
    }
}

pub fn scheduler_invariants_hold(state: &SchedulerState) -> bool {
    scheduler_invariant_violations(state).is_empty()
}

pub fn scheduler_invariant_violations(state: &SchedulerState) -> Vec<String> {
    let mut violations = Vec::new();
    let mut seen = BTreeSet::new();
    for event in &state.events {
        if !seen.insert(event.sequence) {
            violations.push(format!("duplicate scheduler event sequence {}", event.sequence));
        }
    }
    for node_id in &state.retry_queue {
        if state.ready.ordered.contains(node_id) {
            violations.push(format!("node {} is present in both ready and retry queues", node_id));
        }
    }
    for node_id in state.ready.snapshot_sorted() {
        if state.completion_by_node.contains_key(&node_id) {
            violations.push(format!("node {} is ready after reaching terminal state", node_id));
        }
    }
    violations
}

pub fn replay_scheduler_checkpoint(
    plan: &ExecutionPlan,
    checkpoint: &ExecutionCheckpoint,
) -> Result<SchedulerState, String> {
    let known_nodes = plan.nodes.iter().map(|node| node.id.as_str()).collect::<BTreeSet<_>>();
    for node_id in checkpoint
        .ready_queue
        .iter()
        .chain(checkpoint.inflight.iter())
        .chain(checkpoint.scheduled.iter())
        .chain(checkpoint.blocked_by_budget.iter())
        .chain(checkpoint.completed_statuses.keys())
    {
        if !known_nodes.contains(node_id.as_str()) {
            return Err(format!("checkpoint references unknown node '{}'", node_id));
        }
    }

    let mut state = SchedulerState::from_plan(plan);
    state.ready = ReadyQueue::empty();
    for node_id in &checkpoint.ready_queue {
        state.ready.insert(node_id.clone());
    }
    state.completion_by_node = checkpoint.completed_statuses.clone();
    let violations = scheduler_invariant_violations(&state);
    if violations.is_empty() {
        Ok(state)
    } else {
        Err(violations.join("; "))
    }
}

pub fn scheduler_debug_event_log(state: &SchedulerState) -> Vec<SchedulerEvent> {
    state.events().to_vec()
}

pub fn validate_cron_expression(expression: &str) -> Result<(), String> {
    crate::cron_calendar::validate_cron_expression(expression)
}

pub fn validate_schedule_registry(registry: &ScheduleRegistry) -> Result<(), String> {
    let mut ids = BTreeSet::new();
    for definition in &registry.definitions {
        if definition.id.is_empty() {
            return Err("schedule id must not be empty".to_string());
        }
        if !ids.insert(definition.id.clone()) {
            return Err(format!("duplicate schedule id '{}'", definition.id));
        }
        if let TriggerSpec::Cron { expression, timezone } = &definition.trigger {
            validate_cron_expression(expression)?;
            crate::cron_calendar::validate_cron_timezone(timezone)?;
        }
        validate_schedule_policy_combination(definition)?;
    }
    validate_queue_policy_consistency(registry)?;
    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct QueueCapacityPolicy {
    per_queue: u32,
    per_tenant: Option<u32>,
}

fn validate_queue_policy_consistency(registry: &ScheduleRegistry) -> Result<(), String> {
    let mut queue_caps = BTreeMap::<String, u32>::new();
    let mut tenant_caps = BTreeMap::<String, u32>::new();
    for definition in &registry.definitions {
        let queue_name = definition.queue.queue_name.clone();
        let per_queue = definition.concurrency.per_queue.expect("validated by schedule policy");
        match queue_caps.get(&queue_name) {
            Some(existing) if *existing != per_queue => {
                return Err(format!(
                    "queue '{}' has conflicting per_queue caps ({existing} vs {per_queue})",
                    queue_name
                ));
            }
            Some(_) => {}
            None => {
                queue_caps.insert(queue_name, per_queue);
            }
        }

        if let (Some(tenant), Some(per_tenant)) =
            (definition.queue.tenant.as_ref(), definition.concurrency.per_tenant)
        {
            match tenant_caps.get(tenant) {
                Some(existing) if *existing != per_tenant => {
                    return Err(format!(
                        "tenant '{}' has conflicting per_tenant caps ({existing} vs {per_tenant})",
                        tenant
                    ));
                }
                Some(_) => {}
                None => {
                    tenant_caps.insert(tenant.clone(), per_tenant);
                }
            }
        }
    }
    Ok(())
}

fn queue_capacity_policies(
    registry: &ScheduleRegistry,
) -> Result<BTreeMap<String, QueueCapacityPolicy>, String> {
    validate_queue_policy_consistency(registry)?;
    let mut policies = BTreeMap::<String, QueueCapacityPolicy>::new();
    for definition in &registry.definitions {
        let queue_name = definition.queue.queue_name.clone();
        let per_queue = definition.concurrency.per_queue.expect("validated by schedule policy");
        let per_tenant = definition.queue.tenant.as_ref().and(definition.concurrency.per_tenant);
        match policies.get(&queue_name) {
            Some(existing)
                if existing.per_queue != per_queue || existing.per_tenant != per_tenant =>
            {
                return Err(format!(
                    "queue '{}' has conflicting capacity policy declarations",
                    queue_name
                ));
            }
            Some(_) => {}
            None => {
                policies.insert(queue_name, QueueCapacityPolicy { per_queue, per_tenant });
            }
        }
    }
    Ok(policies)
}

fn submission_status_is_active(status: &ScheduleSubmissionStatus) -> bool {
    matches!(status, ScheduleSubmissionStatus::Pending | ScheduleSubmissionStatus::Running)
}

fn schedule_submission_status_can_transition(
    from: &ScheduleSubmissionStatus,
    to: &ScheduleSubmissionStatus,
) -> bool {
    matches!(
        (from, to),
        (ScheduleSubmissionStatus::Pending, ScheduleSubmissionStatus::Pending)
            | (ScheduleSubmissionStatus::Pending, ScheduleSubmissionStatus::Running)
            | (ScheduleSubmissionStatus::Pending, ScheduleSubmissionStatus::Completed)
            | (ScheduleSubmissionStatus::Running, ScheduleSubmissionStatus::Running)
            | (ScheduleSubmissionStatus::Running, ScheduleSubmissionStatus::Completed)
            | (ScheduleSubmissionStatus::Completed, ScheduleSubmissionStatus::Completed)
    )
}

fn latest_schedule_overrides(
    overrides: &ScheduleOverrideState,
) -> BTreeMap<String, ScheduleOverrideRecord> {
    let mut latest = BTreeMap::<String, ScheduleOverrideRecord>::new();
    for record in &overrides.records {
        match latest.get(&record.schedule_id) {
            Some(existing) if compare_schedule_override_precedence(existing, record).is_gt() => {}
            _ => {
                latest.insert(record.schedule_id.clone(), record.clone());
            }
        }
    }
    latest
}

fn compare_schedule_override_precedence(
    left: &ScheduleOverrideRecord,
    right: &ScheduleOverrideRecord,
) -> std::cmp::Ordering {
    left.created_unix_ms
        .cmp(&right.created_unix_ms)
        .then_with(|| left.operator.cmp(&right.operator))
        .then_with(|| match (&left.action, &right.action) {
            (ScheduleOverrideAction::Pause, ScheduleOverrideAction::Resume) => {
                std::cmp::Ordering::Less
            }
            (ScheduleOverrideAction::Resume, ScheduleOverrideAction::Pause) => {
                std::cmp::Ordering::Greater
            }
            _ => std::cmp::Ordering::Equal,
        })
}

fn schedule_is_paused(schedule_id: &str, overrides: &ScheduleOverrideState) -> bool {
    matches!(
        latest_schedule_overrides(overrides).get(schedule_id).map(|record| &record.action),
        Some(ScheduleOverrideAction::Pause)
    )
}

pub fn record_schedule_override(
    overrides: &mut ScheduleOverrideState,
    record: ScheduleOverrideRecord,
) -> Result<(), String> {
    if record.schedule_id.trim().is_empty() {
        return Err("schedule override must declare schedule_id".to_string());
    }
    if record.operator.trim().is_empty() {
        return Err(format!("schedule override '{}' must declare operator", record.schedule_id));
    }
    if record.reason.as_deref().is_some_and(|reason| reason.trim().is_empty()) {
        return Err(format!("schedule override '{}' reason must not be blank", record.schedule_id));
    }
    overrides.records.push(record);
    overrides.records.sort_by(|left, right| {
        left.schedule_id
            .cmp(&right.schedule_id)
            .then_with(|| compare_schedule_override_precedence(left, right))
    });
    Ok(())
}

pub fn pause_schedule(
    overrides: &mut ScheduleOverrideState,
    schedule_id: &str,
    operator: &str,
    at_unix_ms: u128,
    reason: Option<String>,
) -> Result<(), String> {
    record_schedule_override(
        overrides,
        ScheduleOverrideRecord {
            schedule_id: schedule_id.to_string(),
            operator: operator.to_string(),
            action: ScheduleOverrideAction::Pause,
            reason,
            created_unix_ms: at_unix_ms,
        },
    )
}

pub fn resume_schedule(
    overrides: &mut ScheduleOverrideState,
    schedule_id: &str,
    operator: &str,
    at_unix_ms: u128,
    reason: Option<String>,
) -> Result<(), String> {
    record_schedule_override(
        overrides,
        ScheduleOverrideRecord {
            schedule_id: schedule_id.to_string(),
            operator: operator.to_string(),
            action: ScheduleOverrideAction::Resume,
            reason,
            created_unix_ms: at_unix_ms,
        },
    )
}

pub fn build_schedule_override_status(
    registry: &ScheduleRegistry,
    overrides: &ScheduleOverrideState,
) -> Vec<ScheduleOverrideStatus> {
    let latest = latest_schedule_overrides(overrides);
    let mut statuses = registry
        .definitions
        .iter()
        .map(|definition| {
            let record = latest.get(&definition.id);
            ScheduleOverrideStatus {
                schedule_id: definition.id.clone(),
                paused: matches!(
                    record.map(|entry| &entry.action),
                    Some(ScheduleOverrideAction::Pause)
                ),
                operator: record.map(|entry| entry.operator.clone()),
                reason: record.and_then(|entry| entry.reason.clone()),
                updated_unix_ms: record.map(|entry| entry.created_unix_ms),
            }
        })
        .collect::<Vec<_>>();
    statuses.sort_by(|left, right| left.schedule_id.cmp(&right.schedule_id));
    statuses
}

fn starvation_guard_applies(starvation_ticks: u32, policy: &StarvationPreventionPolicy) -> bool {
    starvation_ticks >= policy.max_ticks_without_dispatch
}

fn starvation_boost(
    starvation_ticks: u32,
    weights: &WeightedPriorityPolicy,
    policy: &StarvationPreventionPolicy,
) -> u32 {
    if !starvation_guard_applies(starvation_ticks, policy) {
        return 0;
    }
    let boost_interval = policy.priority_boost_after_ticks.max(1);
    let overdue_ticks = starvation_ticks.saturating_sub(policy.max_ticks_without_dispatch);
    let boost_steps = 1 + (overdue_ticks / boost_interval);
    weights.critical_weight.saturating_mul(boost_steps)
}

fn effective_priority_weight(
    priority: &PriorityClass,
    starvation_ticks: u32,
    policy: &SchedulePriorityDispatchPolicy,
) -> u32 {
    priority_class_weight(Some(priority), &policy.weights).saturating_add(starvation_boost(
        starvation_ticks,
        &policy.weights,
        &policy.starvation,
    ))
}

pub fn validate_schedule_policy_combination(definition: &ScheduleDefinition) -> Result<(), String> {
    if definition.id.trim().is_empty() {
        return Err("schedule id must not be blank".to_string());
    }
    if definition.dag_name.trim().is_empty() {
        return Err(format!("schedule '{}' must declare dag_name", definition.id));
    }
    if definition.dag_version_policy.trim().is_empty() {
        return Err(format!("schedule '{}' must declare dag_version_policy", definition.id));
    }
    if definition.queue.queue_name.trim().is_empty() {
        return Err(format!("schedule '{}' must declare queue_name", definition.id));
    }
    if definition.queue.tenant.as_deref().is_some_and(|tenant| tenant.trim().is_empty()) {
        return Err(format!("schedule '{}' tenant must not be blank", definition.id));
    }
    for (name, value) in [
        ("per_dag", definition.concurrency.per_dag),
        ("per_queue", definition.concurrency.per_queue),
        ("per_tenant", definition.concurrency.per_tenant),
        ("per_node_group", definition.concurrency.per_node_group),
    ] {
        if value == Some(0) {
            return Err(format!(
                "schedule '{}' concurrency layer '{}' must be greater than zero",
                definition.id, name
            ));
        }
    }
    if definition.catch_up.enabled && definition.catch_up.max_catch_up_runs == 0 {
        return Err(format!(
            "schedule '{}' enables catch-up but max_catch_up_runs is zero",
            definition.id
        ));
    }
    if !definition.catch_up.enabled && definition.catch_up.max_catch_up_runs > 0 {
        return Err(format!(
            "schedule '{}' disables catch-up but leaves max_catch_up_runs non-zero",
            definition.id
        ));
    }
    if definition.catch_up.enabled && !matches!(definition.trigger, TriggerSpec::Cron { .. }) {
        return Err(format!(
            "schedule '{}' only supports catch-up on cron triggers",
            definition.id
        ));
    }
    if matches!(definition.trigger, TriggerSpec::Backfill(_)) {
        let TriggerSpec::Backfill(backfill) = &definition.trigger else {
            unreachable!();
        };
        if backfill.window_end_unix_ms < backfill.window_start_unix_ms {
            return Err(format!(
                "schedule '{}' backfill window_end_unix_ms must not precede window_start_unix_ms",
                definition.id
            ));
        }
        if backfill.max_parallelism == 0 {
            return Err(format!(
                "schedule '{}' backfill requires max_parallelism > 0",
                definition.id
            ));
        }
        if definition.concurrency.per_queue.is_none() {
            return Err(format!(
                "schedule '{}' backfill requires queue concurrency cap",
                definition.id
            ));
        }
        if definition.concurrency.per_queue.is_some_and(|cap| backfill.max_parallelism > cap) {
            return Err(format!(
                "schedule '{}' backfill max_parallelism exceeds queue concurrency cap",
                definition.id
            ));
        }
        if backfill.partition_by.as_ref().is_some_and(|value| value.trim().is_empty()) {
            return Err(format!(
                "schedule '{}' backfill partition_by must not be blank",
                definition.id
            ));
        }
        if !backfill.partition_keys.is_empty() && backfill.partition_by.is_none() {
            return Err(format!(
                "schedule '{}' backfill partition_keys require partition_by",
                definition.id
            ));
        }
        let mut seen_partition_keys = BTreeSet::new();
        for partition_key in &backfill.partition_keys {
            if partition_key.trim().is_empty() {
                return Err(format!(
                    "schedule '{}' backfill partition_keys must not contain blanks",
                    definition.id
                ));
            }
            if !seen_partition_keys.insert(partition_key) {
                return Err(format!(
                    "schedule '{}' backfill partition_keys must be unique",
                    definition.id
                ));
            }
        }
    } else if definition.concurrency.per_queue.is_none() {
        return Err(format!("schedule '{}' must declare queue concurrency cap", definition.id));
    }
    validate_schedule_input_contract(definition)?;
    Ok(())
}

pub fn dry_run_schedule(
    definition: &ScheduleDefinition,
    now_unix_ms: u128,
) -> ScheduleDryRunPreview {
    match &definition.trigger {
        TriggerSpec::Manual => ScheduleDryRunPreview {
            schedule_id: definition.id.clone(),
            next_fire_unix_ms: None,
            reason: "manual trigger has no automatic fire time".to_string(),
        },
        TriggerSpec::Cron { expression, timezone } => {
            match crate::cron_calendar::next_cron_fire_unix_ms(expression, timezone, now_unix_ms) {
                Ok(next_fire_unix_ms) => ScheduleDryRunPreview {
                    schedule_id: definition.id.clone(),
                    next_fire_unix_ms,
                    reason: format!("next cron fire resolved in timezone {timezone}"),
                },
                Err(error) => ScheduleDryRunPreview {
                    schedule_id: definition.id.clone(),
                    next_fire_unix_ms: None,
                    reason: error,
                },
            }
        }
        TriggerSpec::Event { .. } | TriggerSpec::Dependency { .. } | TriggerSpec::Signal { .. } => {
            ScheduleDryRunPreview {
                schedule_id: definition.id.clone(),
                next_fire_unix_ms: None,
                reason: "external trigger evaluated on signal arrival".to_string(),
            }
        }
        TriggerSpec::Backfill(backfill) => ScheduleDryRunPreview {
            schedule_id: definition.id.clone(),
            next_fire_unix_ms: Some(backfill.window_start_unix_ms),
            reason: "backfill preview points to window start".to_string(),
        },
    }
}

pub fn compile_submission_request(
    definition: &ScheduleDefinition,
    requested_unix_ms: u128,
) -> Result<ExecutionSubmissionRequest, String> {
    validate_schedule_policy_combination(definition)?;
    build_submission_request(
        definition,
        &compile_submission_candidate(definition, requested_unix_ms)?,
    )
}

#[derive(Debug, Clone)]
struct SubmissionCandidate {
    requested_unix_ms: u128,
    trigger_kind: SubmissionTriggerKind,
    dedupe_key: String,
    context: SubmissionContext,
}

#[derive(Debug, Clone)]
enum SubmissionContext {
    Manual { arguments: BTreeMap<String, Value> },
    Cron,
    Event { record: ScheduleEventRecord },
    Dependency { upstream_run_id: String, status: String },
    Signal { payload: Option<Value> },
    Backfill { window_start_unix_ms: u128, window_end_unix_ms: u128, partition_key: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DependencyCompletionOutcome {
    Success,
    Failure,
}

fn compile_submission_candidate(
    definition: &ScheduleDefinition,
    requested_unix_ms: u128,
) -> Result<SubmissionCandidate, String> {
    let candidate = match &definition.trigger {
        TriggerSpec::Manual => SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: format!("manual:{}:{}", definition.id, requested_unix_ms),
            context: SubmissionContext::Manual { arguments: BTreeMap::new() },
        },
        TriggerSpec::Cron { .. } => SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: format!("cron:{}:{}", definition.id, requested_unix_ms),
            context: SubmissionContext::Cron,
        },
        TriggerSpec::Event { .. } => SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Event,
            dedupe_key: format!("event:{}:compile", definition.id),
            context: SubmissionContext::Event {
                record: ScheduleEventRecord {
                    event_id: String::new(),
                    event_type: String::new(),
                    source: String::new(),
                    occurred_unix_ms: requested_unix_ms,
                    payload: None,
                },
            },
        },
        TriggerSpec::Dependency { .. } => SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Dependency,
            dedupe_key: format!("dependency:{}:compile", definition.id),
            context: SubmissionContext::Dependency {
                upstream_run_id: String::new(),
                status: String::new(),
            },
        },
        TriggerSpec::Signal { .. } => SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Signal,
            dedupe_key: format!("signal:{}:compile", definition.id),
            context: SubmissionContext::Signal { payload: None },
        },
        TriggerSpec::Backfill(_) => {
            return Err(format!(
                "schedule '{}' uses a backfill trigger; use schedule backfill plan instead",
                definition.id
            ));
        }
    };
    Ok(candidate)
}

pub fn compile_backfill_operation(
    definition: &ScheduleDefinition,
    backfill_id: Option<&str>,
    planned_unix_ms: u128,
) -> Result<BackfillOperation, String> {
    validate_schedule_policy_combination(definition)?;
    let TriggerSpec::Backfill(backfill) = &definition.trigger else {
        return Err(format!("schedule '{}' does not declare a backfill trigger", definition.id));
    };
    let backfill_id = backfill_id
        .map(str::to_string)
        .unwrap_or_else(|| deterministic_backfill_id(&definition.id, backfill));
    let runs = plan_backfill_runs(definition, backfill, planned_unix_ms);
    let mut operation = BackfillOperation {
        backfill_id,
        schedule_id: definition.id.clone(),
        dag_name: definition.dag_name.clone(),
        dag_version_policy: definition.dag_version_policy.clone(),
        input_contract: definition.input_contract.clone(),
        input_bindings: definition.input_bindings.clone(),
        queue: definition.queue.clone(),
        priority: definition.priority.clone(),
        request: backfill.clone(),
        lifecycle: BackfillLifecycleStatus::Active,
        lifecycle_reason: None,
        updated_unix_ms: planned_unix_ms,
        audit: vec![BackfillAuditRecord {
            at_unix_ms: planned_unix_ms,
            action: "planned".to_string(),
            detail: format!("planned {} backfill runs", runs.len()),
        }],
        runs,
    };
    refresh_backfill_completion(&mut operation, planned_unix_ms);
    Ok(operation)
}

pub fn pause_backfill_operation(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
    reason: Option<String>,
) -> Result<(), String> {
    match operation.lifecycle {
        BackfillLifecycleStatus::Active => {
            operation.lifecycle = BackfillLifecycleStatus::Paused;
            operation.lifecycle_reason.clone_from(&reason);
            operation.updated_unix_ms = at_unix_ms;
            record_backfill_audit(
                operation,
                at_unix_ms,
                "paused",
                reason.unwrap_or_else(|| "operator pause".to_string()),
            );
            Ok(())
        }
        BackfillLifecycleStatus::Paused => {
            Err(format!("backfill '{}' is already paused", operation.backfill_id))
        }
        BackfillLifecycleStatus::Cancelled => {
            Err(format!("backfill '{}' is cancelled and cannot be paused", operation.backfill_id))
        }
        BackfillLifecycleStatus::Completed => {
            Err(format!("backfill '{}' is completed and cannot be paused", operation.backfill_id))
        }
    }
}

pub fn resume_backfill_operation(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
) -> Result<(), String> {
    match operation.lifecycle {
        BackfillLifecycleStatus::Paused => {
            operation.lifecycle = BackfillLifecycleStatus::Active;
            operation.lifecycle_reason = None;
            operation.updated_unix_ms = at_unix_ms;
            record_backfill_audit(operation, at_unix_ms, "resumed", "operator resume".to_string());
            refresh_backfill_completion(operation, at_unix_ms);
            Ok(())
        }
        BackfillLifecycleStatus::Active => {
            Err(format!("backfill '{}' is already active", operation.backfill_id))
        }
        BackfillLifecycleStatus::Cancelled => {
            Err(format!("backfill '{}' is cancelled and cannot resume", operation.backfill_id))
        }
        BackfillLifecycleStatus::Completed => {
            Err(format!("backfill '{}' is completed and cannot resume", operation.backfill_id))
        }
    }
}

pub fn cancel_backfill_operation(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
    reason: Option<String>,
) -> Result<(), String> {
    match operation.lifecycle {
        BackfillLifecycleStatus::Completed => {
            Err(format!("backfill '{}' is completed and cannot cancel", operation.backfill_id))
        }
        BackfillLifecycleStatus::Cancelled => {
            Err(format!("backfill '{}' is already cancelled", operation.backfill_id))
        }
        BackfillLifecycleStatus::Active | BackfillLifecycleStatus::Paused => {
            for run in &mut operation.runs {
                if matches!(run.status, BackfillRunStatus::Queued) {
                    run.status = BackfillRunStatus::Cancelled;
                    run.updated_unix_ms = at_unix_ms;
                }
            }
            operation.lifecycle = BackfillLifecycleStatus::Cancelled;
            operation.lifecycle_reason.clone_from(&reason);
            operation.updated_unix_ms = at_unix_ms;
            record_backfill_audit(
                operation,
                at_unix_ms,
                "cancelled",
                reason.unwrap_or_else(|| "operator cancel".to_string()),
            );
            Ok(())
        }
    }
}

pub fn retry_failed_backfill_runs(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
) -> Result<usize, String> {
    if matches!(operation.lifecycle, BackfillLifecycleStatus::Cancelled) {
        return Err(format!(
            "backfill '{}' is cancelled and cannot retry failed runs",
            operation.backfill_id
        ));
    }

    let mut retried = 0usize;
    let mut audit_details = Vec::new();
    for run in &mut operation.runs {
        if run.status != BackfillRunStatus::Failed {
            continue;
        }
        let previous_run_id = run.run_id.clone();
        run.previous_run_ids.push(previous_run_id.clone());
        run.attempt = run.attempt.saturating_add(1);
        run.run_id = deterministic_backfill_retry_run_id(
            &operation.schedule_id,
            &run.dedupe_key,
            run.attempt,
        );
        run.status = BackfillRunStatus::Queued;
        run.updated_unix_ms = at_unix_ms;
        retried += 1;
        audit_details.push(format!(
            "queued retry attempt {} for partition run '{}' as '{}'",
            run.attempt, previous_run_id, run.run_id
        ));
    }

    if retried == 0 {
        return Ok(0);
    }

    operation.updated_unix_ms = at_unix_ms;
    operation.lifecycle_reason = None;
    if !matches!(operation.lifecycle, BackfillLifecycleStatus::Active) {
        operation.lifecycle = BackfillLifecycleStatus::Active;
    }
    for detail in audit_details {
        record_backfill_audit(operation, at_unix_ms, "retried_failed_run", detail);
    }
    Ok(retried)
}

pub fn summarize_backfill_operation(operation: &BackfillOperation) -> BackfillOperationSummary {
    let mut partitions = operation
        .runs
        .iter()
        .map(|run| BackfillPartitionSummary {
            requested_unix_ms: run.requested_unix_ms,
            partition_key: run.partition_key.clone(),
            status: run.status.clone(),
            attempt: run.attempt,
            run_id: run.run_id.clone(),
            previous_run_ids: run.previous_run_ids.clone(),
        })
        .collect::<Vec<_>>();
    partitions.sort_by(|left, right| {
        left.requested_unix_ms
            .cmp(&right.requested_unix_ms)
            .then_with(|| left.partition_key.cmp(&right.partition_key))
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    let mut queued_runs = 0usize;
    let mut submitted_runs = 0usize;
    let mut running_runs = 0usize;
    let mut completed_runs = 0usize;
    let mut failed_runs = 0usize;
    let mut cancelled_runs = 0usize;
    let mut total_retry_attempts = 0u32;
    for run in &operation.runs {
        total_retry_attempts = total_retry_attempts.saturating_add(run.attempt.saturating_sub(1));
        match run.status {
            BackfillRunStatus::Queued => queued_runs += 1,
            BackfillRunStatus::Submitted => submitted_runs += 1,
            BackfillRunStatus::Running => running_runs += 1,
            BackfillRunStatus::Completed => completed_runs += 1,
            BackfillRunStatus::Failed => failed_runs += 1,
            BackfillRunStatus::Cancelled => cancelled_runs += 1,
        }
    }

    BackfillOperationSummary {
        backfill_id: operation.backfill_id.clone(),
        schedule_id: operation.schedule_id.clone(),
        dag_name: operation.dag_name.clone(),
        lifecycle: operation.lifecycle.clone(),
        lifecycle_reason: operation.lifecycle_reason.clone(),
        total_runs: operation.runs.len(),
        queued_runs,
        submitted_runs,
        running_runs,
        completed_runs,
        failed_runs,
        cancelled_runs,
        total_retry_attempts,
        partitions,
    }
}

pub fn advance_backfill_operation(
    operation: &BackfillOperation,
    request: &BackfillAdvanceRequest,
) -> Result<BackfillAdvanceReport, String> {
    let mut operation = operation.clone();
    let failure_seen = apply_backfill_status_updates(&mut operation, &request.status_updates)?;
    if matches!(operation.lifecycle, BackfillLifecycleStatus::Active) {
        apply_backfill_failure_policy(&mut operation, request.now_unix_ms, failure_seen);
    }
    refresh_backfill_completion(&mut operation, request.now_unix_ms);

    let active_runs = operation
        .runs
        .iter()
        .filter(|run| {
            matches!(run.status, BackfillRunStatus::Submitted | BackfillRunStatus::Running)
        })
        .count();
    let queued_runs =
        operation.runs.iter().filter(|run| matches!(run.status, BackfillRunStatus::Queued)).count();
    if !matches!(operation.lifecycle, BackfillLifecycleStatus::Active) {
        return Ok(BackfillAdvanceReport {
            operation,
            dispatched_requests: Vec::new(),
            active_runs,
            queued_runs,
            allowed_dispatches: 0,
        });
    }

    let available_parallelism = operation.request.max_parallelism.max(1) as usize
        - active_runs.min(operation.request.max_parallelism.max(1) as usize);
    let throttled_dispatches = crate::scheduler_workload::apply_backfill_throttling(
        queued_runs,
        request.pending_live_runs,
        &request.throttling_policy,
    )
    .0;
    let allowed_dispatches = available_parallelism.min(throttled_dispatches);

    let mut dispatched_requests = Vec::new();
    let schedule_id = operation.schedule_id.clone();
    let dag_name = operation.dag_name.clone();
    let dag_version_policy = operation.dag_version_policy.clone();
    let queue = operation.queue.clone();
    let priority = operation.priority.clone();
    let input_contract = operation.input_contract.clone();
    let input_bindings = operation.input_bindings.clone();
    let window_start_unix_ms = operation.request.window_start_unix_ms;
    let window_end_unix_ms = operation.request.window_end_unix_ms;
    if allowed_dispatches > 0 {
        for run in operation
            .runs
            .iter_mut()
            .filter(|run| matches!(run.status, BackfillRunStatus::Queued))
            .take(allowed_dispatches)
        {
            run.status = BackfillRunStatus::Submitted;
            run.updated_unix_ms = request.now_unix_ms;
            dispatched_requests.push(build_backfill_submission_request(
                &schedule_id,
                &dag_name,
                &dag_version_policy,
                &queue,
                &priority,
                &input_contract,
                &input_bindings,
                window_start_unix_ms,
                window_end_unix_ms,
                run,
            )?);
        }
        record_backfill_audit(
            &mut operation,
            request.now_unix_ms,
            "advanced",
            format!("dispatched {} backfill runs", dispatched_requests.len()),
        );
    }
    refresh_backfill_completion(&mut operation, request.now_unix_ms);
    let active_runs = operation
        .runs
        .iter()
        .filter(|run| {
            matches!(run.status, BackfillRunStatus::Submitted | BackfillRunStatus::Running)
        })
        .count();
    let queued_runs =
        operation.runs.iter().filter(|run| matches!(run.status, BackfillRunStatus::Queued)).count();

    Ok(BackfillAdvanceReport {
        operation,
        dispatched_requests,
        active_runs,
        queued_runs,
        allowed_dispatches,
    })
}

pub fn evaluate_schedule_submissions(
    registry: &ScheduleRegistry,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
) -> ScheduleEvaluationReport {
    evaluate_schedule_submissions_with_overrides(
        registry,
        inputs,
        existing,
        &ScheduleOverrideState::default(),
    )
}

pub fn evaluate_schedule_submissions_with_overrides(
    registry: &ScheduleRegistry,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
    overrides: &ScheduleOverrideState,
) -> ScheduleEvaluationReport {
    let mut definitions = registry.definitions.iter().collect::<Vec<_>>();
    definitions.sort_by(|left, right| left.id.cmp(&right.id));

    let mut candidates = Vec::<ExecutionSubmissionRequest>::new();
    let mut audits = Vec::new();
    for definition in definitions {
        for candidate in candidate_submissions_for_definition(definition, inputs, existing) {
            match build_submission_request(definition, &candidate) {
                Ok(request) => candidates.push(request),
                Err(error) => audits.push(ScheduleAuditRecord {
                    schedule_id: definition.id.clone(),
                    evaluated_unix_ms: inputs.now_unix_ms,
                    decision: "mapping_rejected".to_string(),
                    reason: Some(error),
                }),
            }
        }
    }
    candidates.sort_by(|left, right| {
        left.requested_unix_ms
            .cmp(&right.requested_unix_ms)
            .then_with(|| left.schedule_id.cmp(&right.schedule_id))
            .then_with(|| left.dedupe_key.cmp(&right.dedupe_key))
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    let mut seen_keys =
        existing.entries.iter().map(|entry| entry.dedupe_key.clone()).collect::<BTreeSet<_>>();
    let mut generated_requests = Vec::new();
    let mut recorded_submissions = existing.entries.clone();
    let mut duplicate_suppressions = Vec::new();
    let mut paused_suppressions = Vec::new();
    let mut queue_suppressions = Vec::new();
    let queue_policies = match queue_capacity_policies(registry) {
        Ok(policies) => policies,
        Err(error) => {
            audits.push(ScheduleAuditRecord {
                schedule_id: "<registry>".to_string(),
                evaluated_unix_ms: inputs.now_unix_ms,
                decision: "queue_policy_invalid".to_string(),
                reason: Some(error),
            });
            return ScheduleEvaluationReport {
                generated_requests,
                recorded_submissions,
                duplicate_suppressions,
                paused_suppressions,
                queue_suppressions,
                audits,
            };
        }
    };
    let mut queue_active = existing
        .entries
        .iter()
        .filter(|entry| submission_status_is_active(&entry.status))
        .fold(BTreeMap::<String, usize>::new(), |mut counts, entry| {
            *counts.entry(entry.queue.queue_name.clone()).or_default() += 1;
            counts
        });
    let mut tenant_active = existing
        .entries
        .iter()
        .filter(|entry| submission_status_is_active(&entry.status))
        .filter_map(|entry| {
            entry
                .queue
                .tenant
                .clone()
                .map(|tenant| ((entry.queue.queue_name.clone(), tenant), 1usize))
        })
        .fold(BTreeMap::<(String, String), usize>::new(), |mut counts, (tenant, count)| {
            *counts.entry(tenant).or_default() += count;
            counts
        });

    for request in candidates {
        if schedule_is_paused(&request.schedule_id, overrides) {
            let audit = ScheduleAuditRecord {
                schedule_id: request.schedule_id.clone(),
                evaluated_unix_ms: inputs.now_unix_ms,
                decision: "paused_suppressed".to_string(),
                reason: Some("schedule is paused".to_string()),
            };
            paused_suppressions.push(audit.clone());
            audits.push(audit);
            continue;
        }
        if !seen_keys.insert(request.dedupe_key.clone()) {
            let audit = ScheduleAuditRecord {
                schedule_id: request.schedule_id.clone(),
                evaluated_unix_ms: inputs.now_unix_ms,
                decision: "duplicate_suppressed".to_string(),
                reason: Some(format!("dedupe_key={}", request.dedupe_key)),
            };
            duplicate_suppressions.push(audit.clone());
            audits.push(audit);
            continue;
        }

        let Some(policy) = queue_policies.get(&request.queue.queue_name) else {
            audits.push(ScheduleAuditRecord {
                schedule_id: request.schedule_id.clone(),
                evaluated_unix_ms: inputs.now_unix_ms,
                decision: "queue_policy_missing".to_string(),
                reason: Some(format!("queue '{}'", request.queue.queue_name)),
            });
            continue;
        };
        let queue_inflight =
            queue_active.get(&request.queue.queue_name).copied().unwrap_or_default();
        if queue_inflight >= policy.per_queue as usize {
            let audit = ScheduleAuditRecord {
                schedule_id: request.schedule_id.clone(),
                evaluated_unix_ms: inputs.now_unix_ms,
                decision: "queue_suppressed".to_string(),
                reason: Some(format!(
                    "queue '{}' is at capacity {}/{}",
                    request.queue.queue_name, queue_inflight, policy.per_queue
                )),
            };
            queue_suppressions.push(audit.clone());
            audits.push(audit);
            continue;
        }
        if let (Some(tenant), Some(per_tenant)) = (request.queue.tenant.as_ref(), policy.per_tenant)
        {
            let tenant_key = (request.queue.queue_name.clone(), tenant.clone());
            let tenant_inflight = tenant_active.get(&tenant_key).copied().unwrap_or_default();
            if tenant_inflight >= per_tenant as usize {
                let audit = ScheduleAuditRecord {
                    schedule_id: request.schedule_id.clone(),
                    evaluated_unix_ms: inputs.now_unix_ms,
                    decision: "queue_suppressed".to_string(),
                    reason: Some(format!(
                        "tenant '{}' is at capacity {}/{}",
                        tenant, tenant_inflight, per_tenant
                    )),
                };
                queue_suppressions.push(audit.clone());
                audits.push(audit);
                continue;
            }
            *tenant_active.entry(tenant_key).or_default() += 1;
        }
        *queue_active.entry(request.queue.queue_name.clone()).or_default() += 1;

        audits.push(ScheduleAuditRecord {
            schedule_id: request.schedule_id.clone(),
            evaluated_unix_ms: inputs.now_unix_ms,
            decision: "submitted".to_string(),
            reason: Some(format!(
                "trigger={} dedupe_key={}",
                submission_trigger_kind_name(&request.trigger_kind),
                request.dedupe_key
            )),
        });
        recorded_submissions
            .push(ScheduleSubmissionLedgerEntry::from_request(&request, inputs.now_unix_ms));
        generated_requests.push(request);
    }

    recorded_submissions.sort_by(|left, right| {
        left.created_unix_ms
            .cmp(&right.created_unix_ms)
            .then_with(|| left.schedule_id.cmp(&right.schedule_id))
            .then_with(|| left.run_id.cmp(&right.run_id))
            .then_with(|| left.dedupe_key.cmp(&right.dedupe_key))
    });

    ScheduleEvaluationReport {
        generated_requests,
        recorded_submissions,
        duplicate_suppressions,
        paused_suppressions,
        queue_suppressions,
        audits,
    }
}

impl ScheduleSubmissionLedgerEntry {
    fn from_request(request: &ExecutionSubmissionRequest, created_unix_ms: u128) -> Self {
        Self {
            schedule_id: request.schedule_id.clone(),
            dag_name: request.dag_name.clone(),
            dag_version_policy: request.dag_version_policy.clone(),
            queue: request.queue.clone(),
            priority: request.priority.clone(),
            graph_inputs: request.graph_inputs.clone(),
            requested_unix_ms: request.requested_unix_ms,
            created_unix_ms,
            run_id: request.run_id.clone(),
            trigger_kind: request.trigger_kind.clone(),
            dedupe_key: request.dedupe_key.clone(),
            event_lineage: request.event_lineage.clone(),
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }
    }
}

pub fn dispatch_schedule_queue_runs(
    ledger: &mut ScheduleSubmissionLedger,
    max_dispatches: usize,
    policy: &SchedulePriorityDispatchPolicy,
) -> ScheduleDispatchReport {
    #[derive(Debug, Clone)]
    struct RankedPendingSubmission {
        ledger_index: usize,
        effective_weight: u32,
        starvation_guard_applied: bool,
        starvation_ticks: u32,
        created_unix_ms: u128,
        schedule_id: String,
        run_id: String,
        queue: QueueIdentity,
        priority: PriorityClass,
    }

    let mut ranked = ledger
        .entries
        .iter()
        .enumerate()
        .filter(|(_, entry)| entry.status == ScheduleSubmissionStatus::Pending)
        .map(|(ledger_index, entry)| RankedPendingSubmission {
            ledger_index,
            effective_weight: effective_priority_weight(
                &entry.priority,
                entry.starvation_ticks,
                policy,
            ),
            starvation_guard_applied: starvation_guard_applies(
                entry.starvation_ticks,
                &policy.starvation,
            ),
            starvation_ticks: entry.starvation_ticks,
            created_unix_ms: entry.created_unix_ms,
            schedule_id: entry.schedule_id.clone(),
            run_id: entry.run_id.clone(),
            queue: entry.queue.clone(),
            priority: entry.priority.clone(),
        })
        .collect::<Vec<_>>();

    ranked.sort_by(|left, right| {
        right
            .effective_weight
            .cmp(&left.effective_weight)
            .then_with(|| right.starvation_ticks.cmp(&left.starvation_ticks))
            .then_with(|| left.created_unix_ms.cmp(&right.created_unix_ms))
            .then_with(|| left.schedule_id.cmp(&right.schedule_id))
            .then_with(|| left.run_id.cmp(&right.run_id))
    });

    let selected = max_dispatches.min(ranked.len());
    let mut dispatched_runs = Vec::with_capacity(selected);
    let mut deferred_runs = Vec::with_capacity(ranked.len().saturating_sub(selected));

    for (position, candidate) in ranked.into_iter().enumerate() {
        let entry = &mut ledger.entries[candidate.ledger_index];
        let record = ScheduleDispatchRecord {
            schedule_id: candidate.schedule_id,
            run_id: candidate.run_id,
            queue: candidate.queue,
            priority: candidate.priority,
            starvation_ticks: candidate.starvation_ticks,
            effective_weight: candidate.effective_weight,
            starvation_guard_applied: candidate.starvation_guard_applied,
        };
        if position < selected {
            entry.status = ScheduleSubmissionStatus::Running;
            entry.starvation_ticks = 0;
            dispatched_runs.push(record);
        } else {
            entry.starvation_ticks = entry.starvation_ticks.saturating_add(1);
            deferred_runs.push(record);
        }
    }

    ledger.entries.sort_by(|left, right| {
        left.created_unix_ms
            .cmp(&right.created_unix_ms)
            .then_with(|| left.schedule_id.cmp(&right.schedule_id))
            .then_with(|| left.run_id.cmp(&right.run_id))
            .then_with(|| left.dedupe_key.cmp(&right.dedupe_key))
    });

    ScheduleDispatchReport { dispatched_runs, deferred_runs }
}

pub fn apply_submission_status_updates(
    ledger: &mut ScheduleSubmissionLedger,
    updates: &[ScheduleSubmissionStatusUpdate],
) -> Result<(), String> {
    let mut ordered = updates.to_vec();
    ordered.sort_by(|left, right| {
        left.updated_unix_ms
            .cmp(&right.updated_unix_ms)
            .then_with(|| left.run_id.cmp(&right.run_id))
    });
    for update in ordered {
        let entry =
            ledger.entries.iter_mut().find(|entry| entry.run_id == update.run_id).ok_or_else(
                || format!("submission ledger is missing run_id '{}'", update.run_id),
            )?;
        if !schedule_submission_status_can_transition(&entry.status, &update.status) {
            return Err(format!(
                "submission '{}' cannot transition from {:?} to {:?}",
                update.run_id, entry.status, update.status
            ));
        }
        entry.status = update.status;
    }
    ledger.entries.sort_by(|left, right| {
        left.created_unix_ms
            .cmp(&right.created_unix_ms)
            .then_with(|| left.schedule_id.cmp(&right.schedule_id))
            .then_with(|| left.run_id.cmp(&right.run_id))
            .then_with(|| left.dedupe_key.cmp(&right.dedupe_key))
    });
    Ok(())
}

pub fn build_schedule_queue_state(
    registry: &ScheduleRegistry,
    ledger: &ScheduleSubmissionLedger,
) -> Result<ScheduleQueueState, String> {
    let policies = queue_capacity_policies(registry)?;
    let mut queue_runs = BTreeMap::<String, Vec<ScheduleQueueRunRecord>>::new();
    let mut queue_counts = BTreeMap::<String, usize>::new();
    let mut tenant_counts = BTreeMap::<(String, String), usize>::new();

    for entry in &ledger.entries {
        if !submission_status_is_active(&entry.status) {
            continue;
        }
        *queue_counts.entry(entry.queue.queue_name.clone()).or_default() += 1;
        if let Some(tenant) = entry.queue.tenant.as_ref() {
            *tenant_counts.entry((entry.queue.queue_name.clone(), tenant.clone())).or_default() +=
                1;
        }
        queue_runs.entry(entry.queue.queue_name.clone()).or_default().push(
            ScheduleQueueRunRecord {
                schedule_id: entry.schedule_id.clone(),
                dag_name: entry.dag_name.clone(),
                run_id: entry.run_id.clone(),
                priority: entry.priority.clone(),
                status: entry.status.clone(),
                starvation_ticks: entry.starvation_ticks,
                requested_unix_ms: entry.requested_unix_ms,
                created_unix_ms: entry.created_unix_ms,
            },
        );
    }

    let mut queues = policies
        .into_iter()
        .map(|(queue_name, policy)| {
            let mut runs = queue_runs.remove(&queue_name).unwrap_or_default();
            runs.sort_by(|left, right| {
                left.created_unix_ms
                    .cmp(&right.created_unix_ms)
                    .then_with(|| left.schedule_id.cmp(&right.schedule_id))
                    .then_with(|| left.run_id.cmp(&right.run_id))
            });
            let active_runs = queue_counts.get(&queue_name).copied().unwrap_or_default();
            let available_slots =
                policy.per_queue as usize - active_runs.min(policy.per_queue as usize);
            let mut tenants = tenant_counts
                .iter()
                .filter(|((candidate_queue, _), _)| candidate_queue == &queue_name)
                .map(|((_, tenant), active_runs)| ScheduleQueueTenantState {
                    tenant: tenant.clone(),
                    per_tenant_cap: policy.per_tenant,
                    active_runs: *active_runs,
                    available_slots: policy
                        .per_tenant
                        .map(|cap| cap as usize - (*active_runs).min(cap as usize)),
                })
                .collect::<Vec<_>>();
            tenants.sort_by(|left, right| left.tenant.cmp(&right.tenant));
            ScheduleQueueStateEntry {
                queue_name,
                per_queue_cap: policy.per_queue,
                active_runs,
                available_slots,
                tenants,
                runs,
            }
        })
        .collect::<Vec<_>>();
    queues.sort_by(|left, right| left.queue_name.cmp(&right.queue_name));
    Ok(ScheduleQueueState { queues })
}

fn candidate_submissions_for_definition(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
) -> Vec<SubmissionCandidate> {
    match &definition.trigger {
        TriggerSpec::Manual => manual_submission_candidates(definition, inputs),
        TriggerSpec::Cron { expression, timezone } => {
            cron_submission_candidates(definition, inputs, existing, expression, timezone)
        }
        TriggerSpec::Event { event_type, source } => {
            event_submission_candidates(definition, inputs, event_type, source)
        }
        TriggerSpec::Dependency { dag_name, on_status } => {
            dependency_submission_candidates(definition, inputs, dag_name, on_status)
        }
        TriggerSpec::Signal { signal_name, .. } => {
            signal_submission_candidates(definition, inputs, signal_name)
        }
        TriggerSpec::Backfill(backfill) => {
            backfill_submission_candidates(definition, inputs, backfill)
        }
    }
}

fn manual_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
) -> Vec<SubmissionCandidate> {
    let mut requests = inputs
        .manual_requests
        .iter()
        .filter(|request| request.schedule_id == definition.id)
        .cloned()
        .collect::<Vec<_>>();
    requests.sort_by(|left, right| {
        left.requested_unix_ms
            .cmp(&right.requested_unix_ms)
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    requests
        .into_iter()
        .map(|request| SubmissionCandidate {
            requested_unix_ms: request.requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: format!("manual:{}:{}", definition.id, request.request_id),
            context: SubmissionContext::Manual { arguments: request.arguments },
        })
        .collect()
}

fn cron_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
    expression: &str,
    timezone: &str,
) -> Vec<SubmissionCandidate> {
    let last_requested = existing
        .entries
        .iter()
        .filter(|entry| {
            entry.schedule_id == definition.id && entry.trigger_kind == SubmissionTriggerKind::Cron
        })
        .map(|entry| entry.requested_unix_ms)
        .max();

    let slots = if definition.catch_up.enabled {
        if let Some(last_requested) = last_requested {
            crate::cron_calendar::cron_fire_times_between(
                expression,
                timezone,
                last_requested,
                inputs.now_unix_ms,
                definition.catch_up.max_catch_up_runs.max(1) as usize,
            )
            .unwrap_or_default()
        } else if crate::cron_calendar::cron_matches_unix_ms(
            expression,
            timezone,
            inputs.now_unix_ms,
        )
        .unwrap_or(false)
        {
            vec![inputs.now_unix_ms]
        } else {
            Vec::new()
        }
    } else if crate::cron_calendar::cron_matches_unix_ms(expression, timezone, inputs.now_unix_ms)
        .unwrap_or(false)
        && last_requested != Some(inputs.now_unix_ms)
    {
        vec![inputs.now_unix_ms]
    } else {
        Vec::new()
    };

    slots
        .into_iter()
        .map(|requested_unix_ms| SubmissionCandidate {
            requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: format!("cron:{}:{}", definition.id, requested_unix_ms),
            context: SubmissionContext::Cron,
        })
        .collect()
}

fn event_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_type: &str,
    expected_source: &str,
) -> Vec<SubmissionCandidate> {
    let mut events = inputs
        .events
        .iter()
        .filter(|event| event.event_type == expected_type && event.source == expected_source)
        .cloned()
        .collect::<Vec<_>>();
    events.sort_by(|left, right| {
        left.occurred_unix_ms
            .cmp(&right.occurred_unix_ms)
            .then_with(|| left.event_id.cmp(&right.event_id))
    });
    events
        .into_iter()
        .map(|event| SubmissionCandidate {
            requested_unix_ms: event.occurred_unix_ms,
            trigger_kind: SubmissionTriggerKind::Event,
            dedupe_key: format!("event:{}:{}", definition.id, event.event_id),
            context: SubmissionContext::Event { record: event },
        })
        .collect()
}

fn dependency_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_dag_name: &str,
    expected_status: &DependencyTriggerCondition,
) -> Vec<SubmissionCandidate> {
    let mut completions = inputs
        .dependencies
        .iter()
        .filter(|completion| {
            completion.dag_name == expected_dag_name
                && dependency_trigger_condition_matches(expected_status, &completion.status)
        })
        .cloned()
        .collect::<Vec<_>>();
    completions.sort_by(|left, right| {
        left.finished_unix_ms
            .cmp(&right.finished_unix_ms)
            .then_with(|| left.upstream_run_id.cmp(&right.upstream_run_id))
            .then_with(|| left.status.cmp(&right.status))
    });
    completions
        .into_iter()
        .map(|completion| SubmissionCandidate {
            requested_unix_ms: completion.finished_unix_ms,
            trigger_kind: SubmissionTriggerKind::Dependency,
            dedupe_key: format!(
                "dependency:{}:{}:{}",
                definition.id,
                completion.upstream_run_id,
                dependency_trigger_condition_key(expected_status)
            ),
            context: SubmissionContext::Dependency {
                upstream_run_id: completion.upstream_run_id,
                status: completion.status,
            },
        })
        .collect()
}

fn signal_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_signal_name: &str,
) -> Vec<SubmissionCandidate> {
    let mut signals = inputs
        .signals
        .iter()
        .filter(|signal| signal.signal_name == expected_signal_name)
        .cloned()
        .collect::<Vec<_>>();
    signals.sort_by(|left, right| {
        left.occurred_unix_ms
            .cmp(&right.occurred_unix_ms)
            .then_with(|| left.signal_id.cmp(&right.signal_id))
    });
    signals
        .into_iter()
        .map(|signal| SubmissionCandidate {
            requested_unix_ms: signal.occurred_unix_ms,
            trigger_kind: SubmissionTriggerKind::Signal,
            dedupe_key: format!("signal:{}:{}", definition.id, signal.signal_id),
            context: SubmissionContext::Signal { payload: signal.payload },
        })
        .collect()
}

fn backfill_submission_candidates(
    definition: &ScheduleDefinition,
    _inputs: &ScheduleEvaluationInputs,
    backfill: &BackfillRequest,
) -> Vec<SubmissionCandidate> {
    plan_backfill_runs(definition, backfill, backfill.window_start_unix_ms)
        .into_iter()
        .take(backfill.max_parallelism.max(1) as usize)
        .map(|run| SubmissionCandidate {
            requested_unix_ms: run.requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Backfill,
            dedupe_key: run.dedupe_key,
            context: SubmissionContext::Backfill {
                window_start_unix_ms: backfill.window_start_unix_ms,
                window_end_unix_ms: backfill.window_end_unix_ms,
                partition_key: run.partition_key,
            },
        })
        .collect()
}

fn build_submission_request(
    definition: &ScheduleDefinition,
    candidate: &SubmissionCandidate,
) -> Result<ExecutionSubmissionRequest, String> {
    let graph_inputs = bind_schedule_graph_inputs(definition, candidate)?;
    Ok(ExecutionSubmissionRequest {
        schedule_id: definition.id.clone(),
        dag_name: definition.dag_name.clone(),
        dag_version_policy: definition.dag_version_policy.clone(),
        queue: definition.queue.clone(),
        priority: definition.priority.clone(),
        graph_inputs,
        requested_unix_ms: candidate.requested_unix_ms,
        run_id: deterministic_schedule_run_id(&definition.id, &candidate.dedupe_key),
        trigger_kind: candidate.trigger_kind.clone(),
        dedupe_key: candidate.dedupe_key.clone(),
        event_lineage: event_lineage(candidate),
    })
}

fn build_backfill_submission_request(
    schedule_id: &str,
    dag_name: &str,
    dag_version_policy: &str,
    queue: &QueueIdentity,
    priority: &PriorityClass,
    input_contract: &BTreeMap<String, GraphInputSpec>,
    input_bindings: &BTreeMap<String, ScheduleInputSource>,
    window_start_unix_ms: u128,
    window_end_unix_ms: u128,
    run: &BackfillRunRecord,
) -> Result<ExecutionSubmissionRequest, String> {
    let graph_inputs = bind_schedule_graph_inputs_with_contract(
        schedule_id,
        input_contract,
        input_bindings,
        &SubmissionCandidate {
            requested_unix_ms: run.requested_unix_ms,
            trigger_kind: SubmissionTriggerKind::Backfill,
            dedupe_key: run.dedupe_key.clone(),
            context: SubmissionContext::Backfill {
                window_start_unix_ms,
                window_end_unix_ms,
                partition_key: run.partition_key.clone(),
            },
        },
    )?;
    Ok(ExecutionSubmissionRequest {
        schedule_id: schedule_id.to_string(),
        dag_name: dag_name.to_string(),
        dag_version_policy: dag_version_policy.to_string(),
        queue: queue.clone(),
        priority: priority.clone(),
        graph_inputs,
        requested_unix_ms: run.requested_unix_ms,
        run_id: run.run_id.clone(),
        trigger_kind: SubmissionTriggerKind::Backfill,
        dedupe_key: run.dedupe_key.clone(),
        event_lineage: None,
    })
}

fn event_lineage(candidate: &SubmissionCandidate) -> Option<ScheduleEventLineage> {
    let SubmissionContext::Event { record } = &candidate.context else {
        return None;
    };
    Some(ScheduleEventLineage {
        event_id: record.event_id.clone(),
        event_type: record.event_type.clone(),
        source: record.source.clone(),
        occurred_unix_ms: record.occurred_unix_ms,
    })
}

fn bind_schedule_graph_inputs(
    definition: &ScheduleDefinition,
    candidate: &SubmissionCandidate,
) -> Result<BTreeMap<String, Value>, String> {
    bind_schedule_graph_inputs_with_contract(
        &definition.id,
        &definition.input_contract,
        &definition.input_bindings,
        candidate,
    )
}

fn bind_schedule_graph_inputs_with_contract(
    schedule_id: &str,
    input_contract: &BTreeMap<String, GraphInputSpec>,
    input_bindings: &BTreeMap<String, ScheduleInputSource>,
    candidate: &SubmissionCandidate,
) -> Result<BTreeMap<String, Value>, String> {
    let mut graph_inputs = BTreeMap::new();
    for (input_name, source) in input_bindings {
        let Some(spec) = input_contract.get(input_name) else {
            return Err(format!(
                "schedule '{schedule_id}' binds undeclared graph input '{input_name}'"
            ));
        };
        let raw_value = resolve_schedule_input_source(schedule_id, input_name, source, candidate)?;
        let normalized = materialize_graph_input_value(
            spec,
            &raw_value,
            &format!("/schedule/{schedule_id}/graph_inputs/{input_name}"),
        )
        .map_err(|error| format!("{}: {}", error.path, error.message))?;
        graph_inputs.insert(input_name.clone(), normalized);
    }

    for (input_name, spec) in input_contract {
        if graph_inputs.contains_key(input_name) {
            continue;
        }
        if let Some(default) = spec.effective_value() {
            let normalized = materialize_graph_input_value(
                spec,
                default,
                &format!("/schedule/{schedule_id}/graph_inputs/{input_name}"),
            )
            .map_err(|error| format!("{}: {}", error.path, error.message))?;
            graph_inputs.insert(input_name.clone(), normalized);
        } else if spec.required {
            return Err(format!(
                "schedule '{schedule_id}' could not bind required graph input '{input_name}'"
            ));
        }
    }
    Ok(graph_inputs)
}

fn resolve_schedule_input_source(
    schedule_id: &str,
    input_name: &str,
    source: &ScheduleInputSource,
    candidate: &SubmissionCandidate,
) -> Result<Value, String> {
    match source {
        ScheduleInputSource::RequestedUnixMs => Ok(json_u128(candidate.requested_unix_ms)),
        ScheduleInputSource::ManualArgument { key } => {
            let SubmissionContext::Manual { arguments } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from a manual argument on a non-manual trigger"
                ));
            };
            arguments.get(key).cloned().ok_or_else(|| {
                format!(
                    "schedule '{schedule_id}' missing manual argument '{key}' for graph input '{input_name}'"
                )
            })
        }
        ScheduleInputSource::EventPayload { pointer } => {
            let SubmissionContext::Event { record } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from event payload on a non-event trigger"
                ));
            };
            resolve_payload_binding(
                schedule_id,
                input_name,
                "event payload",
                record.payload.as_ref(),
                pointer.as_deref(),
            )
        }
        ScheduleInputSource::SignalPayload { pointer } => {
            let SubmissionContext::Signal { payload } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from signal payload on a non-signal trigger"
                ));
            };
            resolve_payload_binding(
                schedule_id,
                input_name,
                "signal payload",
                payload.as_ref(),
                pointer.as_deref(),
            )
        }
        ScheduleInputSource::DependencyUpstreamRunId => {
            let SubmissionContext::Dependency { upstream_run_id, .. } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from dependency metadata on a non-dependency trigger"
                ));
            };
            Ok(Value::String(upstream_run_id.clone()))
        }
        ScheduleInputSource::DependencyStatus => {
            let SubmissionContext::Dependency { status, .. } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from dependency metadata on a non-dependency trigger"
                ));
            };
            Ok(Value::String(normalize_schedule_status(status)))
        }
        ScheduleInputSource::BackfillWindowStartUnixMs => {
            let SubmissionContext::Backfill { window_start_unix_ms, .. } = &candidate.context
            else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from backfill metadata on a non-backfill trigger"
                ));
            };
            Ok(json_u128(*window_start_unix_ms))
        }
        ScheduleInputSource::BackfillWindowEndUnixMs => {
            let SubmissionContext::Backfill { window_end_unix_ms, .. } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from backfill metadata on a non-backfill trigger"
                ));
            };
            Ok(json_u128(*window_end_unix_ms))
        }
        ScheduleInputSource::BackfillPartitionKey => {
            let SubmissionContext::Backfill { partition_key, .. } = &candidate.context else {
                return Err(format!(
                    "schedule '{schedule_id}' binds graph input '{input_name}' from backfill metadata on a non-backfill trigger"
                ));
            };
            partition_key.clone().map(Value::String).ok_or_else(|| {
                format!(
                    "schedule '{schedule_id}' missing backfill partition key for graph input '{input_name}'"
                )
            })
        }
    }
}

fn resolve_payload_binding(
    schedule_id: &str,
    input_name: &str,
    payload_name: &str,
    payload: Option<&Value>,
    pointer: Option<&str>,
) -> Result<Value, String> {
    let Some(payload) = payload else {
        return Err(format!(
            "schedule '{schedule_id}' missing {payload_name} for graph input '{input_name}'"
        ));
    };
    let Some(pointer) = pointer else {
        return Ok(payload.clone());
    };
    payload.pointer(pointer).cloned().ok_or_else(|| {
        format!(
            "schedule '{schedule_id}' could not resolve {payload_name} pointer '{pointer}' for graph input '{input_name}'"
        )
    })
}

fn validate_schedule_input_contract(definition: &ScheduleDefinition) -> Result<(), String> {
    for input_name in definition.input_bindings.keys() {
        if !definition.input_contract.contains_key(input_name) {
            return Err(format!(
                "schedule '{}' binds undeclared graph input '{}'",
                definition.id, input_name
            ));
        }
    }
    for (input_name, source) in &definition.input_bindings {
        match source {
            ScheduleInputSource::RequestedUnixMs => {}
            ScheduleInputSource::ManualArgument { key } => {
                if key.trim().is_empty() {
                    return Err(format!(
                        "schedule '{}' manual argument binding for graph input '{}' must not be blank",
                        definition.id, input_name
                    ));
                }
                if !matches!(definition.trigger, TriggerSpec::Manual) {
                    return Err(format!(
                        "schedule '{}' uses manual argument binding for graph input '{}' on a non-manual trigger",
                        definition.id, input_name
                    ));
                }
            }
            ScheduleInputSource::EventPayload { pointer } => {
                if !matches!(definition.trigger, TriggerSpec::Event { .. }) {
                    return Err(format!(
                        "schedule '{}' uses event payload binding for graph input '{}' on a non-event trigger",
                        definition.id, input_name
                    ));
                }
                validate_payload_pointer(&definition.id, input_name, pointer.as_deref())?;
            }
            ScheduleInputSource::SignalPayload { pointer } => {
                if !matches!(definition.trigger, TriggerSpec::Signal { .. }) {
                    return Err(format!(
                        "schedule '{}' uses signal payload binding for graph input '{}' on a non-signal trigger",
                        definition.id, input_name
                    ));
                }
                validate_payload_pointer(&definition.id, input_name, pointer.as_deref())?;
            }
            ScheduleInputSource::DependencyUpstreamRunId
            | ScheduleInputSource::DependencyStatus => {
                if !matches!(definition.trigger, TriggerSpec::Dependency { .. }) {
                    return Err(format!(
                        "schedule '{}' uses dependency metadata binding for graph input '{}' on a non-dependency trigger",
                        definition.id, input_name
                    ));
                }
            }
            ScheduleInputSource::BackfillWindowStartUnixMs
            | ScheduleInputSource::BackfillWindowEndUnixMs
            | ScheduleInputSource::BackfillPartitionKey => {
                if !matches!(definition.trigger, TriggerSpec::Backfill(_)) {
                    return Err(format!(
                        "schedule '{}' uses backfill binding for graph input '{}' on a non-backfill trigger",
                        definition.id, input_name
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_payload_pointer(
    schedule_id: &str,
    input_name: &str,
    pointer: Option<&str>,
) -> Result<(), String> {
    let Some(pointer) = pointer else {
        return Ok(());
    };
    if pointer.is_empty() || pointer.starts_with('/') {
        Ok(())
    } else {
        Err(format!(
            "schedule '{}' payload pointer for graph input '{}' must be empty or start with '/'",
            schedule_id, input_name
        ))
    }
}

fn json_u128(value: u128) -> Value {
    serde_json::to_value(value).expect("u128 should serialize into json")
}

fn deterministic_schedule_run_id(schedule_id: &str, dedupe_key: &str) -> String {
    let slug =
        schedule_id
            .chars()
            .map(|character| {
                if character.is_ascii_alphanumeric() {
                    character.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string();
    let slug = if slug.is_empty() { "schedule".to_string() } else { slug };
    let digest = Sha256::digest(dedupe_key.as_bytes());
    let checksum = format!("{:x}", digest);
    format!("sched-{slug}-{}", &checksum[..12])
}

fn deterministic_backfill_id(schedule_id: &str, backfill: &BackfillRequest) -> String {
    let dedupe_key = format!(
        "backfill:{}:{}:{}:{}:{}",
        schedule_id,
        backfill.window_start_unix_ms,
        backfill.window_end_unix_ms,
        backfill.partition_by.as_deref().unwrap_or("none"),
        backfill.partition_keys.join(",")
    );
    deterministic_schedule_run_id(schedule_id, &dedupe_key)
}

fn deterministic_backfill_retry_run_id(
    schedule_id: &str,
    dedupe_key: &str,
    attempt: u32,
) -> String {
    deterministic_schedule_run_id(schedule_id, &format!("{dedupe_key}:attempt:{attempt}"))
}

fn plan_backfill_runs(
    definition: &ScheduleDefinition,
    backfill: &BackfillRequest,
    planned_unix_ms: u128,
) -> Vec<BackfillRunRecord> {
    let mut runs = Vec::new();
    let partition_keys = if backfill.partition_keys.is_empty() {
        vec![None]
    } else {
        backfill.partition_keys.iter().cloned().map(Some).collect::<Vec<_>>()
    };
    let mut requested_unix_ms = backfill.window_start_unix_ms;
    while requested_unix_ms <= backfill.window_end_unix_ms {
        for partition_key in &partition_keys {
            let dedupe_key = match partition_key {
                Some(partition_key) => {
                    format!("backfill:{}:{}:{}", definition.id, requested_unix_ms, partition_key)
                }
                None => format!("backfill:{}:{}", definition.id, requested_unix_ms),
            };
            runs.push(BackfillRunRecord {
                requested_unix_ms,
                partition_key: partition_key.clone(),
                run_id: deterministic_schedule_run_id(&definition.id, &dedupe_key),
                attempt: default_backfill_attempt(),
                previous_run_ids: Vec::new(),
                dedupe_key,
                status: BackfillRunStatus::Queued,
                updated_unix_ms: planned_unix_ms,
            });
        }
        requested_unix_ms = requested_unix_ms.saturating_add(60_000);
    }
    runs
}

fn apply_backfill_status_updates(
    operation: &mut BackfillOperation,
    updates: &[BackfillStatusUpdate],
) -> Result<bool, String> {
    let mut failure_seen = false;
    for update in updates {
        let Some(run) = operation.runs.iter_mut().find(|run| run.run_id == update.run_id) else {
            return Err(format!(
                "backfill '{}' does not contain run '{}'",
                operation.backfill_id, update.run_id
            ));
        };
        if !backfill_status_transition_allowed(&run.status, &update.status) {
            return Err(format!(
                "backfill '{}' cannot transition run '{}' from {:?} to {:?}",
                operation.backfill_id, update.run_id, run.status, update.status
            ));
        }
        if matches!(update.status, BackfillRunStatus::Failed) {
            failure_seen = true;
        }
        run.status = update.status.clone();
        run.updated_unix_ms = update.updated_unix_ms;
    }
    Ok(failure_seen)
}

fn backfill_status_transition_allowed(
    current: &BackfillRunStatus,
    next: &BackfillRunStatus,
) -> bool {
    if current == next {
        return true;
    }
    match current {
        BackfillRunStatus::Queued => matches!(next, BackfillRunStatus::Cancelled),
        BackfillRunStatus::Submitted => matches!(
            next,
            BackfillRunStatus::Running
                | BackfillRunStatus::Completed
                | BackfillRunStatus::Failed
                | BackfillRunStatus::Cancelled
        ),
        BackfillRunStatus::Running => matches!(
            next,
            BackfillRunStatus::Completed | BackfillRunStatus::Failed | BackfillRunStatus::Cancelled
        ),
        BackfillRunStatus::Completed | BackfillRunStatus::Failed | BackfillRunStatus::Cancelled => {
            false
        }
    }
}

fn apply_backfill_failure_policy(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
    failure_seen: bool,
) {
    if !failure_seen {
        return;
    }
    match operation.request.failure_policy {
        BackfillFailurePolicy::Continue => {
            record_backfill_audit(
                operation,
                at_unix_ms,
                "failure_observed",
                "failure policy kept backfill active".to_string(),
            );
        }
        BackfillFailurePolicy::Pause => {
            operation.lifecycle = BackfillLifecycleStatus::Paused;
            operation.lifecycle_reason = Some("paused after failed backfill run".to_string());
            operation.updated_unix_ms = at_unix_ms;
            record_backfill_audit(
                operation,
                at_unix_ms,
                "paused",
                "failure policy paused backfill after failed run".to_string(),
            );
        }
        BackfillFailurePolicy::Cancel => {
            for run in &mut operation.runs {
                if matches!(run.status, BackfillRunStatus::Queued) {
                    run.status = BackfillRunStatus::Cancelled;
                    run.updated_unix_ms = at_unix_ms;
                }
            }
            operation.lifecycle = BackfillLifecycleStatus::Cancelled;
            operation.lifecycle_reason = Some("cancelled after failed backfill run".to_string());
            operation.updated_unix_ms = at_unix_ms;
            record_backfill_audit(
                operation,
                at_unix_ms,
                "cancelled",
                "failure policy cancelled remaining backfill runs".to_string(),
            );
        }
    }
}

fn refresh_backfill_completion(operation: &mut BackfillOperation, at_unix_ms: u128) {
    if matches!(operation.lifecycle, BackfillLifecycleStatus::Cancelled) {
        return;
    }
    if operation.runs.iter().all(|run| {
        matches!(
            run.status,
            BackfillRunStatus::Completed | BackfillRunStatus::Failed | BackfillRunStatus::Cancelled
        )
    }) {
        operation.lifecycle = BackfillLifecycleStatus::Completed;
        operation.lifecycle_reason = None;
        operation.updated_unix_ms = at_unix_ms;
    }
}

fn record_backfill_audit(
    operation: &mut BackfillOperation,
    at_unix_ms: u128,
    action: &str,
    detail: String,
) {
    operation.audit.push(BackfillAuditRecord { at_unix_ms, action: action.to_string(), detail });
}

fn normalize_schedule_status(status: &str) -> String {
    status.trim().to_ascii_lowercase().replace(['-', ' '], "_")
}

fn classify_dependency_completion_status(status: &str) -> Option<DependencyCompletionOutcome> {
    match normalize_schedule_status(status).as_str() {
        "success" | "succeeded" => Some(DependencyCompletionOutcome::Success),
        "failed" | "failure" | "error" | "cancelled" | "canceled" | "timed_out" | "timeout" => {
            Some(DependencyCompletionOutcome::Failure)
        }
        _ => None,
    }
}

fn dependency_trigger_condition_matches(
    condition: &DependencyTriggerCondition,
    status: &str,
) -> bool {
    let Some(outcome) = classify_dependency_completion_status(status) else {
        return false;
    };
    match condition {
        DependencyTriggerCondition::Success => outcome == DependencyCompletionOutcome::Success,
        DependencyTriggerCondition::Failure => outcome == DependencyCompletionOutcome::Failure,
        DependencyTriggerCondition::AnyTerminal => true,
    }
}

fn dependency_trigger_condition_key(condition: &DependencyTriggerCondition) -> &'static str {
    match condition {
        DependencyTriggerCondition::Success => "success",
        DependencyTriggerCondition::Failure => "failure",
        DependencyTriggerCondition::AnyTerminal => "any_terminal",
    }
}

fn submission_trigger_kind_name(kind: &SubmissionTriggerKind) -> &'static str {
    match kind {
        SubmissionTriggerKind::Manual => "manual",
        SubmissionTriggerKind::Cron => "cron",
        SubmissionTriggerKind::Event => "event",
        SubmissionTriggerKind::Dependency => "dependency",
        SubmissionTriggerKind::Signal => "signal",
        SubmissionTriggerKind::Backfill => "backfill",
    }
}

pub fn deterministic_tick_order(
    mut submissions: Vec<ScheduledSubmission>,
) -> Vec<ScheduledSubmission> {
    submissions.sort_by(|a, b| {
        a.created_unix_ms
            .cmp(&b.created_unix_ms)
            .then_with(|| a.schedule_id.cmp(&b.schedule_id))
            .then_with(|| a.run_id.cmp(&b.run_id))
    });
    submissions
}

fn node_cpu(graph: &Graph, node_id: &str) -> u32 {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .and_then(|n| n.resources.as_ref().map(|r| r.cpu))
        .unwrap_or(1)
        .max(1)
}

fn node_memory_mb(graph: &Graph, node_id: &str) -> u32 {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .and_then(|n| n.resources.as_ref().map(|r| r.mem_mb))
        .unwrap_or(256)
        .max(1)
}

fn node_gpu_devices(graph: &Graph, node_id: &str) -> u32 {
    graph.nodes.iter().find(|n| n.id == node_id).map(resources::node_gpu_devices).unwrap_or(0)
}

fn node_named_resources(graph: &Graph, node_id: &str) -> BTreeMap<String, u32> {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(resources::node_named_resources)
        .unwrap_or_default()
}

fn effective_named_resource_capacities(options: &RuntimeConfig) -> BTreeMap<String, u32> {
    let mut capacities = options.named_resource_capacities.clone();
    for (name, amount) in &options.scheduler_policy.named_resource_capacities {
        capacities.insert(name.clone(), *amount);
    }
    capacities
}

fn first_exhausted_named_resource(
    capacities: &BTreeMap<String, u32>,
    used: &BTreeMap<String, u32>,
    requested: &BTreeMap<String, u32>,
) -> Option<String> {
    requested.iter().find_map(|(name, amount)| {
        let capacity = capacities.get(name).copied().unwrap_or_default();
        let in_use = used.get(name).copied().unwrap_or_default();
        (in_use.saturating_add(*amount) > capacity).then(|| name.clone())
    })
}

fn reserve_named_resources(used: &mut BTreeMap<String, u32>, requested: &BTreeMap<String, u32>) {
    for (name, amount) in requested {
        *used.entry(name.clone()).or_default() += *amount;
    }
}

fn node_priority(graph: &Graph, node_id: &str) -> u8 {
    let Some(node) = graph.nodes.iter().find(|node| node.id == node_id) else {
        return 1;
    };
    if node.tags.iter().any(|tag| tag == "critical" || tag == "priority:critical") {
        4
    } else if node.tags.iter().any(|tag| tag == "high" || tag == "priority:high") {
        3
    } else if node.tags.iter().any(|tag| tag == "low" || tag == "priority:low") {
        1
    } else {
        2
    }
}
