use crate::scheduler::{
    BackfillRequest, PriorityClass, ScheduleDefinition, ScheduledSubmission, TriggerSpec,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DagCalendar {
    pub timezone: String,
    pub blackout_windows: Vec<BlackoutWindow>,
    pub holiday_policy: HolidayPolicy,
    pub suppress_by_environment: Vec<EnvironmentSuppression>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackoutWindow {
    pub start_unix_ms: u128,
    pub end_unix_ms: u128,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HolidayPolicy {
    pub region: String,
    pub holiday_dates: Vec<String>,
    pub suppress_runs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentSuppression {
    pub environment: String,
    pub suppressed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartitionBackfillOrchestration {
    pub request: BackfillRequest,
    pub partition_keys: Vec<String>,
    pub max_inflight_partitions: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackfillThrottlingPolicy {
    pub max_backfill_submissions_per_tick: u32,
    pub reserve_live_capacity_percent: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FairnessAlgorithm {
    RoundRobin,
    WeightedFairQueue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StarvationPreventionPolicy {
    pub max_ticks_without_dispatch: u32,
    pub priority_boost_after_ticks: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceClass {
    Interactive,
    Batch,
    Archival,
    Critical,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueAdmissionPolicy {
    pub min_free_cpu_percent: u8,
    pub min_free_memory_percent: u8,
    pub max_queue_depth: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunBatchPolicy {
    pub allow_grouping: bool,
    pub max_group_size: u32,
    pub require_same_dag: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConcurrencyScope {
    Dag,
    TaskGroup,
    Tenant,
    Queue,
    Global,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeightedPriorityPolicy {
    pub critical_weight: u32,
    pub high_weight: u32,
    pub standard_weight: u32,
    pub low_weight: u32,
}

impl Default for WeightedPriorityPolicy {
    fn default() -> Self {
        Self { critical_weight: 100, high_weight: 75, standard_weight: 50, low_weight: 25 }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyTriggerBufferPolicy {
    pub max_buffered_events: usize,
    pub dedup_window_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterializedRunPreview {
    pub schedule_id: String,
    pub next_run_unix_ms: Vec<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CronConflict {
    pub schedule_ids: Vec<String>,
    pub expression: String,
    pub timezone: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerDedupDecision {
    pub deduplicated: bool,
    pub dedup_key: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleSuppressionAnnotation {
    pub schedule_id: String,
    pub reason: String,
    pub until_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScheduleOverrideAction {
    Pause,
    Resume,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleOverrideRecord {
    pub schedule_id: String,
    pub operator: String,
    pub action: ScheduleOverrideAction,
    pub reason: Option<String>,
    pub created_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ScheduleOverrideState {
    #[serde(default)]
    pub records: Vec<ScheduleOverrideRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleOverrideStatus {
    pub schedule_id: String,
    pub paused: bool,
    pub operator: Option<String>,
    pub reason: Option<String>,
    pub updated_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlaPolicy {
    pub expected_start_within_ms: u64,
    pub expected_finish_within_ms: u64,
    pub max_latency_budget_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerSlaMetrics {
    pub missed_expected_start: u64,
    pub missed_expected_finish: u64,
    pub queue_saturation_count: u64,
    pub fairness_drift_count: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerAlertRule {
    pub name: String,
    pub threshold: u64,
    pub metric: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulingSimulationSuite {
    pub fixtures: Vec<String>,
    pub objective: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CrossSchedulerCompatibility {
    pub shared_submission_ordering: bool,
    pub shared_priority_semantics: bool,
    pub shared_dedup_semantics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerMaturityMatrix {
    pub local_only_ready: bool,
    pub durable_ready: bool,
    pub multi_queue_ready: bool,
    pub backfill_ready: bool,
    pub ha_ready: bool,
}

pub(crate) fn priority_class_weight(
    class: Option<&PriorityClass>,
    policy: &WeightedPriorityPolicy,
) -> u32 {
    match class {
        Some(PriorityClass::Critical) => policy.critical_weight,
        Some(PriorityClass::High) => policy.high_weight,
        Some(PriorityClass::Standard) => policy.standard_weight,
        Some(PriorityClass::Low) => policy.low_weight,
        None => 0,
    }
}

pub fn is_suppressed_by_calendar(
    calendar: &DagCalendar,
    environment: &str,
    now_unix_ms: u128,
) -> bool {
    let in_blackout = calendar
        .blackout_windows
        .iter()
        .any(|w| now_unix_ms >= w.start_unix_ms && now_unix_ms <= w.end_unix_ms);
    let env_suppressed = calendar
        .suppress_by_environment
        .iter()
        .any(|e| e.environment == environment && e.suppressed);
    in_blackout || env_suppressed
}

pub fn compute_partition_backfill_batches(
    orchestration: &PartitionBackfillOrchestration,
) -> Vec<Vec<String>> {
    let width = orchestration.max_inflight_partitions.max(1) as usize;
    orchestration.partition_keys.chunks(width).map(|chunk| chunk.to_vec()).collect()
}

pub fn apply_backfill_throttling(
    pending_backfill_runs: usize,
    pending_live_runs: usize,
    policy: &BackfillThrottlingPolicy,
) -> (usize, usize) {
    let reserve_live = (pending_live_runs * policy.reserve_live_capacity_percent as usize) / 100;
    let allowed_backfill = pending_backfill_runs
        .min(policy.max_backfill_submissions_per_tick as usize)
        .saturating_sub(reserve_live.min(pending_backfill_runs));
    (allowed_backfill, pending_live_runs)
}

pub fn weighted_priority_tie_break_order(
    mut submissions: Vec<ScheduledSubmission>,
    priorities: &BTreeMap<String, PriorityClass>,
    policy: &WeightedPriorityPolicy,
) -> Vec<ScheduledSubmission> {
    submissions.sort_by(|a, b| {
        let wa = priority_class_weight(priorities.get(&a.schedule_id), policy);
        let wb = priority_class_weight(priorities.get(&b.schedule_id), policy);
        wb.cmp(&wa)
            .then_with(|| a.created_unix_ms.cmp(&b.created_unix_ms))
            .then_with(|| a.schedule_id.cmp(&b.schedule_id))
            .then_with(|| a.run_id.cmp(&b.run_id))
    });
    submissions
}

pub fn materialize_next_runs(
    definition: &ScheduleDefinition,
    now_unix_ms: u128,
    n: usize,
) -> MaterializedRunPreview {
    let mut next = Vec::new();
    match definition.trigger {
        TriggerSpec::Cron { ref expression, ref timezone } => {
            if let Ok(runs) = crate::cron_calendar::materialize_next_cron_runs(
                expression,
                timezone,
                now_unix_ms,
                n,
            ) {
                next = runs;
            }
        }
        TriggerSpec::Backfill(ref b) => {
            let end = b.window_end_unix_ms.max(b.window_start_unix_ms);
            let mut cursor = b.window_start_unix_ms;
            while cursor <= end && next.len() < n.max(1) {
                next.push(cursor);
                cursor += 60_000;
            }
        }
        _ => {}
    }
    MaterializedRunPreview { schedule_id: definition.id.clone(), next_run_unix_ms: next }
}

pub fn detect_cron_conflicts(definitions: &[ScheduleDefinition]) -> Vec<CronConflict> {
    let mut grouped: BTreeMap<(String, String), Vec<String>> = BTreeMap::new();
    for d in definitions {
        if let TriggerSpec::Cron { expression, timezone } = &d.trigger {
            grouped.entry((expression.clone(), timezone.clone())).or_default().push(d.id.clone());
        }
    }
    grouped
        .into_iter()
        .filter(|(_, ids)| ids.len() > 1)
        .map(|((expression, timezone), mut ids)| {
            ids.sort();
            CronConflict { schedule_ids: ids, expression, timezone }
        })
        .collect()
}

pub fn deduplicate_trigger_events(keys: &[String]) -> Vec<TriggerDedupDecision> {
    let mut seen = BTreeSet::new();
    keys.iter()
        .map(|k| {
            if seen.insert(k.clone()) {
                TriggerDedupDecision {
                    deduplicated: false,
                    dedup_key: k.clone(),
                    reason: "first occurrence".to_string(),
                }
            } else {
                TriggerDedupDecision {
                    deduplicated: true,
                    dedup_key: k.clone(),
                    reason: "duplicate trigger key".to_string(),
                }
            }
        })
        .collect()
}

pub fn run_batches(queue: VecDeque<String>, policy: &RunBatchPolicy) -> Vec<Vec<String>> {
    if !policy.allow_grouping {
        return queue.into_iter().map(|id| vec![id]).collect();
    }
    let group_size = policy.max_group_size.max(1) as usize;
    let mut out = Vec::new();
    let mut chunk = Vec::new();
    for id in queue {
        chunk.push(id);
        if chunk.len() == group_size {
            out.push(chunk);
            chunk = Vec::new();
        }
    }
    if !chunk.is_empty() {
        out.push(chunk);
    }
    out
}

pub fn evaluate_sla_metrics(
    starts: &[(u128, u128)],
    finishes: &[(u128, u128)],
    queue_saturation_count: u64,
    fairness_drift_count: u64,
) -> SchedulerSlaMetrics {
    let missed_expected_start =
        starts.iter().filter(|(actual, expected)| actual > expected).count() as u64;
    let missed_expected_finish =
        finishes.iter().filter(|(actual, expected)| actual > expected).count() as u64;
    SchedulerSlaMetrics {
        missed_expected_start,
        missed_expected_finish,
        queue_saturation_count,
        fairness_drift_count,
    }
}
