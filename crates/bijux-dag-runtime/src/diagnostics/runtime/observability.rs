use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum EventCategory {
    Plan,
    Schedule,
    Dispatch,
    Start,
    Retry,
    Timeout,
    CacheHit,
    CacheMiss,
    Failure,
    Replay,
    Verify,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventRecord {
    pub category: EventCategory,
    pub name: String,
    pub unix_ms: u128,
    pub node_id: Option<String>,
    pub run_id: Option<String>,
    pub details: serde_json::Value,
}

pub const REQUIRED_RUNTIME_EVENT_NAMES: &[&str] = &[
    "run_started",
    "node_ready",
    "node_started",
    "node_attempt_started",
    "node_attempt_finished",
    "node_scheduled",
    "node_finished",
    "run_finished",
];

pub fn required_event_fields_present(event: &EventRecord) -> bool {
    !event.name.trim().is_empty()
        && event.unix_ms > 0
        && event.run_id.as_ref().map(|v| !v.is_empty()).unwrap_or(false)
}

pub fn validate_required_event_names(events: &[EventRecord]) -> Vec<String> {
    let mut missing = Vec::new();
    for name in REQUIRED_RUNTIME_EVENT_NAMES {
        if !events.iter().any(|event| event.name == *name) {
            missing.push((*name).to_string());
        }
    }
    missing
}

pub fn event_names_emitted_once(events: &[EventRecord], names: &[&str]) -> bool {
    names.iter().all(|name| events.iter().filter(|event| event.name == *name).count() == 1)
}

pub fn event_contains_sensitive_material(event: &EventRecord) -> bool {
    let serialized = event.details.to_string().to_lowercase();
    serialized.contains("secret") || serialized.contains("token") || serialized.contains("password")
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventLogCompletenessReport {
    pub complete: bool,
    pub required_names_present: bool,
    pub required_timeline_labels_present: bool,
    pub required_event_field_gaps: Vec<String>,
    pub missing_required_names: Vec<String>,
    pub missing_required_timeline_labels: Vec<String>,
    pub monotonic_timestamps: bool,
    pub timeline_matches_reconstruction: bool,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeMetrics {
    pub node_id: String,
    pub queue_delay_ms: u128,
    pub execution_time_ms: u128,
    pub retries: u32,
    pub output_bytes: u64,
    pub cache_status: String,
    pub effect_usage: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunMetrics {
    pub makespan_ms: u128,
    pub success_ratio: f64,
    pub parallelism_utilization: f64,
    pub cache_reuse_ratio: f64,
    pub artifact_volume_bytes: u64,
    pub planning_ms: u128,
    pub scheduling_wait_ms: u128,
    pub execution_ms: u128,
    pub trace_write_ms: u128,
    pub manifest_finalize_ms: u128,
    pub replay_compare_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulerMetrics {
    pub queue_depth: usize,
    pub ready_count: usize,
    pub running_count: usize,
    pub completed_count: usize,
    pub retry_count: u64,
    pub cache_hit_count: u64,
    pub cache_miss_count: u64,
    pub failure_count: u64,
    pub starvation_count: u64,
    pub dispatch_latency_ms: u128,
    pub concurrency_pressure: f64,
}

pub trait MetricsRegistry {
    fn record_node(&mut self, metrics: NodeMetrics);
    fn record_run(&mut self, metrics: RunMetrics);
    fn record_scheduler(&mut self, metrics: SchedulerMetrics);
}

#[derive(Default)]
pub struct InMemoryMetricsRegistry {
    pub node_metrics: Vec<NodeMetrics>,
    pub run_metrics: Vec<RunMetrics>,
    pub scheduler_metrics: Vec<SchedulerMetrics>,
}

impl MetricsRegistry for InMemoryMetricsRegistry {
    fn record_node(&mut self, metrics: NodeMetrics) {
        self.node_metrics.push(metrics);
    }
    fn record_run(&mut self, metrics: RunMetrics) {
        self.run_metrics.push(metrics);
    }
    fn record_scheduler(&mut self, metrics: SchedulerMetrics) {
        self.scheduler_metrics.push(metrics);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SpanKind {
    Run,
    Node,
    Adapter,
    Artifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TraceSpan {
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub kind: SpanKind,
    pub name: String,
    pub started_unix_ms: u128,
    pub finished_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineEntry {
    pub unix_ms: u128,
    pub category: String,
    pub label: String,
    pub node_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_event: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineExport {
    pub schema_version: String,
    pub entries: Vec<TimelineEntry>,
}

pub fn canonicalize_event_records(events: &[EventRecord]) -> Vec<EventRecord> {
    let mut ordered = events.to_vec();
    ordered.sort_by(compare_runtime_events);
    ordered
}

pub fn reconstruct_timeline_from_events(events: &[EventRecord]) -> TimelineExport {
    let ordered = canonicalize_event_records(events);
    TimelineExport {
        schema_version: "v0.1".to_string(),
        entries: ordered.iter().map(timeline_entry_from_event).collect(),
    }
}

pub fn verify_event_log_completeness(
    events: &[EventRecord],
    timeline: Option<&TimelineExport>,
) -> EventLogCompletenessReport {
    let missing_required_names = validate_required_event_names(events);
    let reconstructed = reconstruct_timeline_from_events(events);
    let timeline_for_validation = timeline.unwrap_or(&reconstructed);
    let missing_required_timeline_labels =
        validate_required_timeline_labels(events, timeline_for_validation);
    let required_event_field_gaps = events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (!required_event_fields_present(event)).then_some((idx, event)))
        .map(|(idx, event)| format!("event[{idx}] missing required fields: {}", event.name))
        .collect::<Vec<_>>();
    let monotonic_timestamps =
        events.windows(2).all(|window| window[0].unix_ms <= window[1].unix_ms);
    let timeline_matches_reconstruction =
        timeline.map(|candidate| candidate == &reconstructed).unwrap_or(true);

    let mut gaps = Vec::new();
    if !missing_required_names.is_empty() {
        gaps.push(format!(
            "missing required runtime events: {}",
            missing_required_names.join(", ")
        ));
    }
    if !missing_required_timeline_labels.is_empty() {
        gaps.push(format!(
            "missing required timeline labels: {}",
            missing_required_timeline_labels.join(", ")
        ));
    }
    gaps.extend(required_event_field_gaps.iter().cloned());
    if !monotonic_timestamps {
        gaps.push("event timestamps are not monotonic".to_string());
    }
    if timeline.is_some() && !timeline_matches_reconstruction {
        gaps.push("stored timeline does not match reconstructed event timeline".to_string());
    }

    EventLogCompletenessReport {
        complete: gaps.is_empty(),
        required_names_present: missing_required_names.is_empty(),
        required_timeline_labels_present: missing_required_timeline_labels.is_empty(),
        required_event_field_gaps,
        missing_required_names,
        missing_required_timeline_labels,
        monotonic_timestamps,
        timeline_matches_reconstruction,
        gaps,
    }
}

pub fn validate_required_timeline_labels(
    events: &[EventRecord],
    timeline: &TimelineExport,
) -> Vec<String> {
    let expected = required_timeline_labels(events);
    let present =
        timeline.entries.iter().map(|entry| entry.label.as_str()).collect::<BTreeSet<_>>();
    expected.into_iter().filter(|label| !present.contains(label.as_str())).collect()
}

fn required_timeline_labels(events: &[EventRecord]) -> BTreeSet<String> {
    events.iter().filter_map(required_timeline_label_for_event).collect()
}

fn required_timeline_label_for_event(event: &EventRecord) -> Option<String> {
    match event.name.as_str() {
        "run_started" => Some("run_started".to_string()),
        "node_ready" => Some("node_ready".to_string()),
        "node_scheduled" => Some("node_scheduled".to_string()),
        "node_started" => Some("node_started".to_string()),
        "node_finished" | "node_skipped" => Some(timeline_label(event)),
        "run_finished" => Some("run_completed".to_string()),
        _ => None,
    }
}

fn timeline_entry_from_event(event: &EventRecord) -> TimelineEntry {
    TimelineEntry {
        unix_ms: event.unix_ms,
        category: timeline_category(event),
        label: timeline_label(event),
        node_id: event.node_id.clone(),
        status: event_status(event),
        reason: event_reason(event),
        source_event: Some(event.name.clone()),
    }
}

fn timeline_category(event: &EventRecord) -> String {
    match event.name.as_str() {
        "run_started" | "run_finished" => "run".to_string(),
        "node_ready" => "ready".to_string(),
        "node_scheduled" => "schedule".to_string(),
        "node_started" | "node_attempt_started" => "start".to_string(),
        "node_attempt_finished" | "node_retry_scheduled" | "node_retry_exhausted" => {
            "retry".to_string()
        }
        "cache_hit" => "cache_hit".to_string(),
        "cache_miss" => "cache_miss".to_string(),
        "run_timeout" => "timeout".to_string(),
        "run_cancel_requested" => "cancel".to_string(),
        "node_blocked" => "blocked".to_string(),
        "node_skipped" | "node_finished" => timeline_terminal_category(event).to_string(),
        _ => format!("{:?}", event.category).to_lowercase(),
    }
}

fn timeline_terminal_category(event: &EventRecord) -> &'static str {
    match event_terminal_status(event).as_deref() {
        Some("failed") => "failure",
        Some("skipped") => "skip",
        Some("cached") => "cache_hit",
        Some("cancelled") => "cancel",
        _ => "complete",
    }
}

fn timeline_label(event: &EventRecord) -> String {
    match event.name.as_str() {
        "run_started" => "run_started".to_string(),
        "run_finished" => "run_completed".to_string(),
        "node_ready" => "node_ready".to_string(),
        "node_scheduled" => "node_scheduled".to_string(),
        "node_started" => "node_started".to_string(),
        "node_attempt_started" => "node_attempt_started".to_string(),
        "node_attempt_finished" => "node_attempt_finished".to_string(),
        "node_retry_scheduled" => "node_retry_scheduled".to_string(),
        "node_retry_exhausted" => "node_retry_exhausted".to_string(),
        "cache_hit" => "cache_hit".to_string(),
        "cache_miss" => "cache_miss".to_string(),
        "run_timeout" => "run_timed_out".to_string(),
        "run_cancel_requested" => "run_cancel_requested".to_string(),
        "node_blocked" => "node_blocked".to_string(),
        "node_skipped" | "node_finished" => match event_terminal_status(event).as_deref() {
            Some("failed") => "node_failed".to_string(),
            Some("skipped") => "node_skipped".to_string(),
            Some("cached") => "node_cached".to_string(),
            Some("cancelled") => "node_cancelled".to_string(),
            _ => "node_completed".to_string(),
        },
        _ => event.name.clone(),
    }
}

fn event_terminal_status(event: &EventRecord) -> Option<String> {
    if event.name == "node_skipped" {
        return Some(if event_reason(event).as_deref() == Some("cancelled") {
            "cancelled".to_string()
        } else {
            "skipped".to_string()
        });
    }
    event_status(event)
}

fn event_status(event: &EventRecord) -> Option<String> {
    event.details.get("status").and_then(|value| value.as_str()).map(ToString::to_string)
}

fn event_reason(event: &EventRecord) -> Option<String> {
    event.details.get("reason").and_then(|value| value.as_str()).map(ToString::to_string)
}

fn compare_runtime_events(left: &EventRecord, right: &EventRecord) -> Ordering {
    left.unix_ms
        .cmp(&right.unix_ms)
        .then_with(|| runtime_event_rank(left).cmp(&runtime_event_rank(right)))
        .then_with(|| left.node_id.cmp(&right.node_id))
        .then_with(|| left.name.cmp(&right.name))
}

fn runtime_event_rank(event: &EventRecord) -> u8 {
    match event.name.as_str() {
        "run_started" => 0,
        "plan_built" => 5,
        "node_ready" => 10,
        "scheduler_decision" => 15,
        "node_scheduled" => 20,
        "cache_hit" | "cache_miss" => 25,
        "node_started" => 30,
        "node_attempt_started" => 40,
        "node_attempt_finished" => 50,
        "node_retry_scheduled" => 60,
        "node_retry_exhausted" => 65,
        "run_cancel_requested" | "run_timeout" => 70,
        "node_blocked" => 75,
        "branch_decision_selected" => 80,
        "node_skipped" | "node_finished" => 90,
        "run_finished" => 255,
        _ => 200,
    }
}

pub trait EventSink: Send + Sync {
    fn write_event(&self, event: &EventRecord) -> Result<(), String>;
}

pub struct StdoutEventSink;

impl EventSink for StdoutEventSink {
    #[allow(clippy::print_stdout)]
    fn write_event(&self, event: &EventRecord) -> Result<(), String> {
        let line = serde_json::to_string(event).map_err(|err| err.to_string())?;
        println!("{line}");
        Ok(())
    }
}

pub struct FileEventSink {
    path: std::path::PathBuf,
}

impl FileEventSink {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self { path: path.as_ref().to_path_buf() }
    }
}

impl EventSink for FileEventSink {
    fn write_event(&self, event: &EventRecord) -> Result<(), String> {
        let line = serde_json::to_string(event).map_err(|err| err.to_string())?;
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|err| err.to_string())?;
        writeln!(file, "{line}").map_err(|err| err.to_string())
    }
}

pub struct RemoteCollectorSink;

impl EventSink for RemoteCollectorSink {
    fn write_event(&self, _event: &EventRecord) -> Result<(), String> {
        Ok(())
    }
}

pub fn write_timeline_export(
    path: impl AsRef<Path>,
    timeline: &TimelineExport,
) -> Result<(), String> {
    if let Some(parent) = path.as_ref().parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let payload = serialize_timeline_export(timeline)?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

pub fn serialize_timeline_export(timeline: &TimelineExport) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(timeline).map_err(|err| err.to_string())
}

pub fn summarize_failure_root_causes(events: &[EventRecord]) -> Vec<String> {
    let mut roots = Vec::new();
    for event in events {
        if event.category == EventCategory::Failure {
            if let Some(node_id) = event.node_id.as_ref() {
                let reason =
                    event.details.get("reason").and_then(|v| v.as_str()).unwrap_or("unspecified");
                roots.push(format!("{node_id}:{reason}"));
            }
        }
    }
    roots.sort();
    roots.dedup();
    roots
}

pub fn category_from_runtime_event_name(name: &str) -> EventCategory {
    match name {
        "plan_built" => EventCategory::Plan,
        "scheduler_decision" | "node_ready" | "node_scheduled" | "node_blocked" => {
            EventCategory::Schedule
        }
        "node_dispatch" => EventCategory::Dispatch,
        "node_started" | "run_started" | "run_finished" => EventCategory::Start,
        "node_attempt_started"
        | "node_attempt_finished"
        | "node_retry_scheduled"
        | "node_retry_exhausted" => EventCategory::Retry,
        "run_timeout" => EventCategory::Timeout,
        "cache_hit" => EventCategory::CacheHit,
        "cache_miss" => EventCategory::CacheMiss,
        "node_failed" | "node_finished" | "node_skipped" | "policy_denied" => {
            EventCategory::Failure
        }
        "run_cancel_requested" => EventCategory::Dispatch,
        "replay_reused" | "replay_reexecuted" => EventCategory::Replay,
        "verify_completed" => EventCategory::Verify,
        _ => EventCategory::Dispatch,
    }
}

pub fn current_process_memory_bytes() -> Option<u64> {
    #[cfg(target_os = "linux")]
    {
        let content = fs::read_to_string("/proc/self/statm").ok()?;
        let pages = content.split_whitespace().nth(1)?.parse::<u64>().ok()?;
        let page_size = 4096u64;
        return Some(pages.saturating_mul(page_size));
    }
    #[cfg(not(target_os = "linux"))]
    {
        None
    }
}
