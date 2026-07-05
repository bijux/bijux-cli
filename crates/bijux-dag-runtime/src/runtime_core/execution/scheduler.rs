use crate::execution_plan::ExecutionPlan;
use crate::RuntimeConfig;
use bijux_dag_core::Graph;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerPolicy {
    pub max_parallelism: usize,
    pub cpu_budget: Option<u32>,
    pub fairness: SchedulerFairness,
    pub queue_isolation: QueueIsolationPolicy,
    pub bounded_executor_capacity: usize,
    pub prefer_throughput_scheduler: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TriggerSpec {
    Manual,
    Cron { expression: String, timezone: String },
    Event { event_type: String, source: String },
    Dependency { dag_name: String, on_status: String },
    Signal { signal_name: String, payload_schema: Option<String> },
    Backfill(BackfillRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct QueueIdentity {
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
    pub status: BackfillRunStatus,
    pub updated_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillAuditRecord {
    pub at_unix_ms: u128,
    pub action: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillOperation {
    pub backfill_id: String,
    pub schedule_id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BackfillAdvanceReport {
    pub operation: BackfillOperation,
    #[serde(default)]
    pub dispatched_requests: Vec<ExecutionSubmissionRequest>,
    pub active_runs: usize,
    pub queued_runs: usize,
    pub allowed_dispatches: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleDefinition {
    pub id: String,
    pub dag_name: String,
    pub dag_version_policy: String,
    pub trigger: TriggerSpec,
    pub queue: QueueIdentity,
    pub priority: PriorityClass,
    pub concurrency: ConcurrencyPolicyLayers,
    pub catch_up: CatchUpPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
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
    pub requested_unix_ms: u128,
    pub run_id: String,
    pub trigger_kind: SubmissionTriggerKind,
    pub dedupe_key: String,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScheduleEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub source: String,
    pub occurred_unix_ms: u128,
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
    pub requested_unix_ms: u128,
    pub created_unix_ms: u128,
    pub run_id: String,
    pub trigger_kind: SubmissionTriggerKind,
    pub dedupe_key: String,
    pub status: ScheduleSubmissionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ScheduleSubmissionLedger {
    #[serde(default)]
    pub entries: Vec<ScheduleSubmissionLedgerEntry>,
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
    pub audits: Vec<ScheduleAuditRecord>,
}

impl Default for SchedulerPolicy {
    fn default() -> Self {
        Self {
            max_parallelism: 1,
            cpu_budget: None,
            fairness: SchedulerFairness::Deterministic,
            queue_isolation: QueueIsolationPolicy::SingleQueue,
            bounded_executor_capacity: 64,
            prefer_throughput_scheduler: false,
        }
    }
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
    pub failure_propagation_mode: String,
    pub dependency_closure_enabled: bool,
    pub generated_unix_ms: u128,
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
    PriorityCpuFitThenNodeId,
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
        ready_tie_break: ReadyTieBreak::PriorityCpuFitThenNodeId,
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
        let mut used_cpu = 0u32;
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        let mut blocked_reasons = BTreeMap::new();
        let mut candidates = ready_queue
            .snapshot_sorted()
            .into_iter()
            .map(|node_id| ReadyCandidate {
                priority: node_priority(graph, &node_id),
                cpu: node_cpu(graph, &node_id),
                node_id,
            })
            .collect::<Vec<_>>();
        candidates.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.cpu.cmp(&b.cpu))
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
            used_cpu += candidate.cpu;
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
            tie_break_reason: Some("priority_cpu_fit_then_node_id".to_string()),
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
        let mut used_cpu = 0u32;
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
            if used_cpu + cpu > cpu_budget {
                blocked_reasons.insert(id.clone(), "blocked_by_cpu".to_string());
                blocked.push(id);
                continue;
            }
            used_cpu += cpu;
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
    Ok(())
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
) -> ExecutionSubmissionRequest {
    build_submission_request(
        definition,
        requested_unix_ms,
        SubmissionTriggerKind::Manual,
        format!("manual:{}:{}", definition.id, requested_unix_ms),
    )
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
            operation.lifecycle_reason = reason.clone();
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
            operation.lifecycle_reason = reason.clone();
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
    if allowed_dispatches > 0 {
        for run in operation
            .runs
            .iter_mut()
            .filter(|run| matches!(run.status, BackfillRunStatus::Queued))
            .take(allowed_dispatches)
        {
            run.status = BackfillRunStatus::Submitted;
            run.updated_unix_ms = request.now_unix_ms;
            dispatched_requests.push(ExecutionSubmissionRequest {
                schedule_id: operation.schedule_id.clone(),
                dag_name: operation.dag_name.clone(),
                dag_version_policy: operation.dag_version_policy.clone(),
                requested_unix_ms: run.requested_unix_ms,
                run_id: run.run_id.clone(),
                trigger_kind: SubmissionTriggerKind::Backfill,
                dedupe_key: run.dedupe_key.clone(),
            });
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
    let mut definitions = registry.definitions.iter().collect::<Vec<_>>();
    definitions.sort_by(|left, right| left.id.cmp(&right.id));

    let mut candidates = Vec::<ExecutionSubmissionRequest>::new();
    for definition in definitions {
        candidates.extend(candidate_submissions_for_definition(definition, inputs, existing));
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
    let mut audits = Vec::new();

    for request in candidates {
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
        audits,
    }
}

impl ScheduleSubmissionLedgerEntry {
    fn from_request(request: &ExecutionSubmissionRequest, created_unix_ms: u128) -> Self {
        Self {
            schedule_id: request.schedule_id.clone(),
            dag_name: request.dag_name.clone(),
            dag_version_policy: request.dag_version_policy.clone(),
            requested_unix_ms: request.requested_unix_ms,
            created_unix_ms,
            run_id: request.run_id.clone(),
            trigger_kind: request.trigger_kind.clone(),
            dedupe_key: request.dedupe_key.clone(),
            status: ScheduleSubmissionStatus::Pending,
        }
    }
}

fn candidate_submissions_for_definition(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
) -> Vec<ExecutionSubmissionRequest> {
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
) -> Vec<ExecutionSubmissionRequest> {
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
        .map(|request| {
            build_submission_request(
                definition,
                request.requested_unix_ms,
                SubmissionTriggerKind::Manual,
                format!("manual:{}:{}", definition.id, request.request_id),
            )
        })
        .collect()
}

fn cron_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    existing: &ScheduleSubmissionLedger,
    expression: &str,
    timezone: &str,
) -> Vec<ExecutionSubmissionRequest> {
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
        .map(|requested_unix_ms| {
            build_submission_request(
                definition,
                requested_unix_ms,
                SubmissionTriggerKind::Cron,
                format!("cron:{}:{}", definition.id, requested_unix_ms),
            )
        })
        .collect()
}

fn event_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_type: &str,
    expected_source: &str,
) -> Vec<ExecutionSubmissionRequest> {
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
        .map(|event| {
            build_submission_request(
                definition,
                event.occurred_unix_ms,
                SubmissionTriggerKind::Event,
                format!("event:{}:{}", definition.id, event.event_id),
            )
        })
        .collect()
}

fn dependency_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_dag_name: &str,
    expected_status: &str,
) -> Vec<ExecutionSubmissionRequest> {
    let expected_status = normalize_schedule_status(expected_status);
    let mut completions = inputs
        .dependencies
        .iter()
        .filter(|completion| {
            completion.dag_name == expected_dag_name
                && normalize_schedule_status(&completion.status) == expected_status
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
        .map(|completion| {
            build_submission_request(
                definition,
                completion.finished_unix_ms,
                SubmissionTriggerKind::Dependency,
                format!(
                    "dependency:{}:{}:{}",
                    definition.id,
                    completion.upstream_run_id,
                    normalize_schedule_status(&completion.status)
                ),
            )
        })
        .collect()
}

fn signal_submission_candidates(
    definition: &ScheduleDefinition,
    inputs: &ScheduleEvaluationInputs,
    expected_signal_name: &str,
) -> Vec<ExecutionSubmissionRequest> {
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
        .map(|signal| {
            build_submission_request(
                definition,
                signal.occurred_unix_ms,
                SubmissionTriggerKind::Signal,
                format!("signal:{}:{}", definition.id, signal.signal_id),
            )
        })
        .collect()
}

fn backfill_submission_candidates(
    definition: &ScheduleDefinition,
    _inputs: &ScheduleEvaluationInputs,
    backfill: &BackfillRequest,
) -> Vec<ExecutionSubmissionRequest> {
    plan_backfill_runs(definition, backfill, backfill.window_start_unix_ms)
        .into_iter()
        .take(backfill.max_parallelism.max(1) as usize)
        .map(|run| ExecutionSubmissionRequest {
            schedule_id: definition.id.clone(),
            dag_name: definition.dag_name.clone(),
            dag_version_policy: definition.dag_version_policy.clone(),
            requested_unix_ms: run.requested_unix_ms,
            run_id: run.run_id,
            trigger_kind: SubmissionTriggerKind::Backfill,
            dedupe_key: run.dedupe_key,
        })
        .collect()
}

fn build_submission_request(
    definition: &ScheduleDefinition,
    requested_unix_ms: u128,
    trigger_kind: SubmissionTriggerKind,
    dedupe_key: String,
) -> ExecutionSubmissionRequest {
    ExecutionSubmissionRequest {
        schedule_id: definition.id.clone(),
        dag_name: definition.dag_name.clone(),
        dag_version_policy: definition.dag_version_policy.clone(),
        requested_unix_ms,
        run_id: deterministic_schedule_run_id(&definition.id, &dedupe_key),
        trigger_kind,
        dedupe_key,
    }
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
