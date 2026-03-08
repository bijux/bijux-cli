use crate::execution_plan::ExecutionPlan;
use crate::RuntimeConfig;
use bijux_dag_core::Graph;
use serde::{Deserialize, Serialize};
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
    Cron {
        expression: String,
        timezone: String,
    },
    Event {
        event_type: String,
        source: String,
    },
    Dependency {
        dag_name: String,
        on_status: String,
    },
    Signal {
        signal_name: String,
        payload_schema: Option<String>,
    },
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
pub struct BackfillRequest {
    pub window_start_unix_ms: u128,
    pub window_end_unix_ms: u128,
    pub partition_by: Option<String>,
    pub max_parallelism: u32,
    pub failure_policy: String,
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
    pub batch: Vec<String>,
    pub blocked_by_budget: Vec<String>,
    pub timed_out: bool,
    pub cancelled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionCheckpoint {
    pub loop_index: u64,
    pub ready_queue_depth: usize,
    pub scheduled: Vec<String>,
    pub blocked_by_budget: Vec<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadyTieBreak {
    LexicographicNodeId,
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
        priority_model: SchedulerPriorityModel::StaticAbsent,
        ready_tie_break: ReadyTieBreak::LexicographicNodeId,
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
        self.completion_by_node
            .insert(node_id.to_string(), "failed".to_string());
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
        self.completion_by_node
            .insert(node_id.to_string(), status.to_string());
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
        Self {
            indegree: plan.indegree.clone(),
            adj: plan.adj.clone(),
        }
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

fn preflight_decision(
    options: &RuntimeConfig,
    started: Instant,
    cancellation_requested: bool,
) -> Option<ScheduleDecision> {
    if cancellation_requested {
        return Some(ScheduleDecision {
            batch: Vec::new(),
            blocked_by_budget: Vec::new(),
            timed_out: false,
            cancelled: true,
        });
    }
    if let Some(limit_ms) = options.run_timeout_ms {
        if started.elapsed() > Duration::from_millis(limit_ms) {
            return Some(ScheduleDecision {
                batch: Vec::new(),
                blocked_by_budget: Vec::new(),
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
        let cpu_budget = options.scheduler_policy.cpu_budget.or(options.cpu_budget).unwrap_or(
            options.jobs.max(1) as u32,
        );
        let mut used_cpu = 0u32;
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        let mut candidates = ready_queue.snapshot_sorted();
        for id in candidates.drain(..) {
            if batch.len()
                >= options
                    .scheduler_policy
                    .max_parallelism
                    .max(1)
                    .min(options.jobs.max(1))
            {
                blocked.push(id);
                continue;
            }
            let cpu = node_cpu(graph, &id);
            if used_cpu + cpu > cpu_budget {
                blocked.push(id);
                continue;
            }
            used_cpu += cpu;
            let _ = ready_queue.pop_deterministic();
            batch.push(id);
        }
        if batch.is_empty() {
            if let Some(id) = ready_queue.pop_deterministic() {
                batch.push(id);
            }
        }
        ScheduleDecision {
            batch,
            blocked_by_budget: blocked,
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
        let cpu_budget = options.scheduler_policy.cpu_budget.or(options.cpu_budget).unwrap_or(
            options.jobs.max(1) as u32,
        );
        let mut used_cpu = 0u32;
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        while !ready_queue.is_empty()
            && batch.len()
                < options
                    .scheduler_policy
                    .max_parallelism
                    .max(1)
                    .min(options.jobs.max(1))
        {
            let id = match ready_queue.pop_fifo() {
                Some(v) => v,
                None => break,
            };
            let cpu = node_cpu(graph, &id);
            if used_cpu + cpu > cpu_budget {
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
            batch,
            blocked_by_budget: blocked,
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
    let mut seen = BTreeSet::new();
    for event in &state.events {
        if !seen.insert(event.sequence) {
            return false;
        }
    }
    let retry_conflict = state
        .retry_queue
        .iter()
        .any(|id| state.ready.ordered.contains(id));
    !retry_conflict
}

pub fn scheduler_debug_event_log(state: &SchedulerState) -> Vec<SchedulerEvent> {
    state.events().to_vec()
}

pub fn validate_cron_expression(expression: &str) -> Result<(), String> {
    let fields: Vec<&str> = expression.split_whitespace().collect();
    if fields.len() != 5 {
        return Err("cron expression must have exactly five fields".to_string());
    }
    for field in fields {
        if field == "*" {
            continue;
        }
        if field.chars().all(|c| c.is_ascii_digit()) {
            continue;
        }
        return Err(format!("unsupported cron token '{field}'"));
    }
    Ok(())
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
        if let TriggerSpec::Cron { expression, .. } = &definition.trigger {
            validate_cron_expression(expression)?;
        }
        validate_schedule_policy_combination(definition)?;
    }
    Ok(())
}

pub fn validate_schedule_policy_combination(definition: &ScheduleDefinition) -> Result<(), String> {
    if definition.catch_up.enabled && definition.catch_up.max_catch_up_runs == 0 {
        return Err(format!(
            "schedule '{}' enables catch-up but max_catch_up_runs is zero",
            definition.id
        ));
    }
    if matches!(definition.trigger, TriggerSpec::Backfill(_)) {
        let TriggerSpec::Backfill(backfill) = &definition.trigger else {
            unreachable!();
        };
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
        TriggerSpec::Cron { expression, .. } => {
            if validate_cron_expression(expression).is_ok() {
                ScheduleDryRunPreview {
                    schedule_id: definition.id.clone(),
                    next_fire_unix_ms: Some(now_unix_ms + 60_000),
                    reason: "preview uses one-minute horizon for valid cron expression".to_string(),
                }
            } else {
                ScheduleDryRunPreview {
                    schedule_id: definition.id.clone(),
                    next_fire_unix_ms: None,
                    reason: "cron expression is invalid".to_string(),
                }
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
    ExecutionSubmissionRequest {
        schedule_id: definition.id.clone(),
        dag_name: definition.dag_name.clone(),
        dag_version_policy: definition.dag_version_policy.clone(),
        requested_unix_ms,
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
