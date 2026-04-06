use crate::observability::{EventCategory, EventRecord, TimelineEntry, TimelineExport};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticsKind {
    Validation,
    RuntimeFailure,
    PolicyDenial,
    RecoveryAnomaly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureCauseCode {
    ValidationError,
    PlannerError,
    PolicyDenied,
    ExecutionError,
    Timeout,
    Cancellation,
    CacheCorruption,
    WorkerChurn,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DiagnosticRecord {
    pub kind: DiagnosticsKind,
    pub cause_code: FailureCauseCode,
    pub message: String,
    pub run_id: Option<String>,
    pub node_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventCorrelation {
    pub correlation_id: String,
    pub planner_event_id: Option<String>,
    pub scheduler_event_id: Option<String>,
    pub worker_event_id: Option<String>,
    pub artifact_event_id: Option<String>,
    pub audit_event_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReplaySpanLink {
    pub from_span_id: String,
    pub to_span_id: String,
    pub relationship: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimelineTextSummary {
    pub lines: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyOverlayNode {
    pub node_id: String,
    pub queued: bool,
    pub retries: u32,
    pub cache_status: String,
    pub artifact_reused: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TopologyOverlay {
    pub schema_version: String,
    pub nodes: Vec<TopologyOverlayNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MetricsExportFormat {
    JsonFile,
    StdoutJson,
    Otlp,
    PrometheusText,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlertRule {
    pub name: String,
    pub metric: String,
    pub threshold: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservabilityContractStatus {
    pub emits_events: bool,
    pub emits_metrics: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainRunReport {
    pub what_happened: Vec<String>,
    pub why_happened: Vec<String>,
    pub what_next: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainNodeReport {
    pub node_id: String,
    pub reason: String,
    pub state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainArtifactReport {
    pub artifact_id: String,
    pub produced_by: Option<String>,
    pub consumed_by: Vec<String>,
    pub reproducible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExplainScheduleReport {
    pub schedule_id: String,
    pub created_run: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionPolicy {
    pub hide_node_params: bool,
    pub hide_env_values: bool,
    pub hide_artifact_metadata_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplingPolicy {
    pub enabled: bool,
    pub max_spans_per_run: usize,
    pub max_events_per_run: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvestigationBundle {
    pub run_id: String,
    pub event_paths: Vec<String>,
    pub manifest_paths: Vec<String>,
    pub lineage_paths: Vec<String>,
    pub log_paths: Vec<String>,
    pub summary_paths: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftDetectionReport {
    pub dag_name: String,
    pub baseline_name: String,
    pub drift_findings: Vec<String>,
}

pub fn build_diagnostics(events: &[EventRecord]) -> Vec<DiagnosticRecord> {
    events
        .iter()
        .filter_map(|event| match event.category {
            EventCategory::Failure => Some(DiagnosticRecord {
                kind: DiagnosticsKind::RuntimeFailure,
                cause_code: FailureCauseCode::ExecutionError,
                message: event.name.clone(),
                run_id: event.run_id.clone(),
                node_id: event.node_id.clone(),
            }),
            EventCategory::Timeout => Some(DiagnosticRecord {
                kind: DiagnosticsKind::RuntimeFailure,
                cause_code: FailureCauseCode::Timeout,
                message: event.name.clone(),
                run_id: event.run_id.clone(),
                node_id: event.node_id.clone(),
            }),
            _ => None,
        })
        .collect()
}

pub fn root_cause_graph(events: &[EventRecord]) -> BTreeMap<String, Vec<String>> {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for event in events {
        if event.category == EventCategory::Failure {
            let node = event.node_id.clone().unwrap_or_else(|| "run".to_string());
            if let Some(upstream) = event.details.get("upstream").and_then(|v| v.as_str()) {
                map.entry(node).or_default().push(upstream.to_string());
            } else {
                map.entry(node).or_default();
            }
        }
    }
    for values in map.values_mut() {
        values.sort();
        values.dedup();
    }
    map
}

pub fn render_timeline_text(timeline: &TimelineExport) -> TimelineTextSummary {
    let mut entries = timeline.entries.clone();
    entries.sort_by(|a, b| a.unix_ms.cmp(&b.unix_ms));
    let lines = entries
        .into_iter()
        .map(|entry| format!("{} {} {}", entry.unix_ms, entry.category, entry.label))
        .collect();
    TimelineTextSummary { lines }
}

pub fn build_topology_overlay(entries: &[TimelineEntry]) -> TopologyOverlay {
    let mut nodes: BTreeMap<String, TopologyOverlayNode> = BTreeMap::new();
    for entry in entries {
        if let Some(node_id) = &entry.node_id {
            let node = nodes.entry(node_id.clone()).or_insert(TopologyOverlayNode {
                node_id: node_id.clone(),
                queued: false,
                retries: 0,
                cache_status: "miss".to_string(),
                artifact_reused: false,
            });
            if entry.category == "schedule" || entry.category == "dispatch" {
                node.queued = true;
            }
            if entry.category == "retry" {
                node.retries += 1;
            }
            if entry.label.contains("cache_hit") {
                node.cache_status = "hit".to_string();
                node.artifact_reused = true;
            }
        }
    }
    TopologyOverlay {
        schema_version: "v0.1".to_string(),
        nodes: nodes.into_values().collect(),
    }
}

pub fn observability_contract_status(
    events: &[EventRecord],
    metric_count: usize,
) -> ObservabilityContractStatus {
    ObservabilityContractStatus {
        emits_events: !events.is_empty(),
        emits_metrics: metric_count > 0,
    }
}

pub fn redact_event_details(event: &EventRecord, policy: &RedactionPolicy) -> EventRecord {
    let mut cloned = event.clone();
    if policy.hide_node_params {
        if let Some(obj) = cloned.details.as_object_mut() {
            obj.remove("params");
        }
    }
    if policy.hide_env_values {
        if let Some(obj) = cloned.details.as_object_mut() {
            obj.remove("env");
        }
    }
    if let Some(obj) = cloned.details.as_object_mut() {
        for key in &policy.hide_artifact_metadata_keys {
            obj.remove(key);
        }
    }
    cloned
}

pub fn sample_events(events: &[EventRecord], policy: &SamplingPolicy) -> Vec<EventRecord> {
    if !policy.enabled || events.len() <= policy.max_events_per_run {
        return events.to_vec();
    }
    let stride = (events.len() / policy.max_events_per_run.max(1)).max(1);
    events
        .iter()
        .enumerate()
        .filter_map(|(idx, event)| {
            if idx % stride == 0 {
                Some(event.clone())
            } else {
                None
            }
        })
        .take(policy.max_events_per_run)
        .collect()
}

pub fn build_investigation_bundle(run_id: &str, run_dir: &str) -> InvestigationBundle {
    InvestigationBundle {
        run_id: run_id.to_string(),
        event_paths: vec![format!("{run_dir}/observability.events.json")],
        manifest_paths: vec![format!("{run_dir}/manifest.json")],
        lineage_paths: vec![format!(
            "{run_dir}/observability.lineage-visualization.json"
        )],
        log_paths: vec![format!("{run_dir}/nodes/*/stderr.log")],
        summary_paths: vec![format!("{run_dir}/observability.root-causes.json")],
    }
}

pub fn detect_metric_drift(
    current: &BTreeMap<String, f64>,
    baseline: &BTreeMap<String, f64>,
    dag_name: &str,
    baseline_name: &str,
) -> DriftDetectionReport {
    let mut findings = BTreeSet::new();
    for (metric, value) in current {
        if let Some(base) = baseline.get(metric) {
            if (*value - *base).abs() > 0.20 * base.max(1.0) {
                findings.insert(format!("{metric} drifted from {base:.2} to {value:.2}"));
            }
        }
    }
    DriftDetectionReport {
        dag_name: dag_name.to_string(),
        baseline_name: baseline_name.to_string(),
        drift_findings: findings.into_iter().collect(),
    }
}
