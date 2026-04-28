use serde::{Deserialize, Serialize};
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
    "node_failed",
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
    pub required_event_field_gaps: Vec<String>,
    pub missing_required_names: Vec<String>,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimelineExport {
    pub schema_version: String,
    pub entries: Vec<TimelineEntry>,
}

pub fn reconstruct_timeline_from_events(events: &[EventRecord]) -> TimelineExport {
    TimelineExport {
        schema_version: "v0.1".to_string(),
        entries: events
            .iter()
            .map(|event| TimelineEntry {
                unix_ms: event.unix_ms,
                category: format!("{:?}", event.category).to_lowercase(),
                label: event.name.clone(),
                node_id: event.node_id.clone(),
            })
            .collect(),
    }
}

pub fn verify_event_log_completeness(
    events: &[EventRecord],
    timeline: Option<&TimelineExport>,
) -> EventLogCompletenessReport {
    let missing_required_names = validate_required_event_names(events);
    let required_event_field_gaps = events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| (!required_event_fields_present(event)).then_some((idx, event)))
        .map(|(idx, event)| format!("event[{idx}] missing required fields: {}", event.name))
        .collect::<Vec<_>>();
    let monotonic_timestamps = events.windows(2).all(|window| window[0].unix_ms <= window[1].unix_ms);
    let reconstructed = reconstruct_timeline_from_events(events);
    let timeline_matches_reconstruction = timeline
        .map(|candidate| candidate == &reconstructed)
        .unwrap_or(true);

    let mut gaps = Vec::new();
    if !missing_required_names.is_empty() {
        gaps.push(format!(
            "missing required runtime events: {}",
            missing_required_names.join(", ")
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
        required_event_field_gaps,
        missing_required_names,
        monotonic_timestamps,
        timeline_matches_reconstruction,
        gaps,
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
    let payload = serde_json::to_vec_pretty(timeline).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
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
        "node_scheduled" => EventCategory::Schedule,
        "node_dispatch" => EventCategory::Dispatch,
        "node_started" | "run_started" => EventCategory::Start,
        "node_attempt_started" | "node_attempt_finished" => EventCategory::Retry,
        "run_timeout" => EventCategory::Timeout,
        "cache_hit" => EventCategory::CacheHit,
        "cache_miss" => EventCategory::CacheMiss,
        "node_failed" | "policy_denied" => EventCategory::Failure,
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
