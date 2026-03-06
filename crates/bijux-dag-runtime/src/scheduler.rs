use crate::execution_plan::ExecutionPlan;
use crate::RuntimeConfig;
use bijux_dag_core::Graph;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeSet, HashMap, VecDeque};
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

#[derive(Debug, Clone)]
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

impl Scheduler for DeterministicScheduler {
    fn next_batch(
        &mut self,
        graph: &Graph,
        ready_queue: &mut ReadyQueue,
        options: &RuntimeConfig,
        started: Instant,
        cancellation_requested: bool,
    ) -> ScheduleDecision {
        if cancellation_requested {
            return ScheduleDecision {
                batch: Vec::new(),
                blocked_by_budget: Vec::new(),
                timed_out: false,
                cancelled: true,
            };
        }
        if let Some(limit_ms) = options.run_timeout_ms {
            if started.elapsed() > Duration::from_millis(limit_ms) {
                return ScheduleDecision {
                    batch: Vec::new(),
                    blocked_by_budget: Vec::new(),
                    timed_out: true,
                    cancelled: false,
                };
            }
        }
        let cpu_budget = options
            .scheduler_policy
            .cpu_budget
            .or(options.cpu_budget)
            .unwrap_or(options.jobs.max(1) as u32);
        let mut used_cpu = 0u32;
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        let mut candidates = ready_queue.snapshot_sorted();
        for id in candidates.drain(..) {
            if batch.len() >= options.scheduler_policy.max_parallelism.max(1).min(options.jobs.max(1)) {
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
        if cancellation_requested {
            return ScheduleDecision {
                batch: Vec::new(),
                blocked_by_budget: Vec::new(),
                timed_out: false,
                cancelled: true,
            };
        }
        if let Some(limit_ms) = options.run_timeout_ms {
            if started.elapsed() > Duration::from_millis(limit_ms) {
                return ScheduleDecision {
                    batch: Vec::new(),
                    blocked_by_budget: Vec::new(),
                    timed_out: true,
                    cancelled: false,
                };
            }
        }
        let cpu_budget = options
            .scheduler_policy
            .cpu_budget
            .or(options.cpu_budget)
            .unwrap_or(options.jobs.max(1) as u32);
        let mut used_cpu = 0u32;
        let mut batch = Vec::new();
        let mut blocked = Vec::new();
        while !ready_queue.is_empty()
            && batch.len() < options.scheduler_policy.max_parallelism.max(1).min(options.jobs.max(1))
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

fn node_cpu(graph: &Graph, node_id: &str) -> u32 {
    graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .and_then(|n| n.resources.as_ref().map(|r| r.cpu))
        .unwrap_or(1)
        .max(1)
}
