use crate::routes::run_lookup::RunWorkspacePaths;
use crate::routes::selector_grammar::{SelectorExpression, SelectorField};
use bijux_dag_artifacts::FailureInfo;
use bijux_dag_runtime::{ExecutionCheckpoint, TimelineExport};
use serde::Serialize;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const RUN_HISTORY_INDEX_FILE: &str = ".bijux-run-history-index.json";

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub(crate) struct RunTimelineQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub node: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub since_unix_ms: Option<u128>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub until_unix_ms: Option<u128>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TimelineEventView {
    unix_ms: Option<u128>,
    category: String,
    label: String,
    node_id: Option<String>,
    status: Option<String>,
    reason: Option<String>,
    source_event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct TimelineQueryReport {
    source: String,
    filters: RunTimelineQuery,
    total_event_count: usize,
    matched_event_count: usize,
    events: Vec<TimelineEventView>,
}

pub fn resolve_run_dir(root: &Path, run_id: &str) -> PathBuf {
    RunWorkspacePaths::for_run(root, run_id)
        .map(|paths| paths.preferred_read_path())
        .unwrap_or_else(|_| root.join(run_id))
}

pub fn list_runs(root: &Path) -> Result<Vec<String>, std::io::Error> {
    let mut runs = Vec::new();
    if !root.exists() {
        return Ok(runs);
    }
    for entry in fs::read_dir(root)? {
        let entry = entry?;
        if !entry.path().is_dir() {
            continue;
        }
        runs.push(entry.file_name().to_string_lossy().to_string());
    }
    runs.sort();
    Ok(runs)
}

fn read_json(path: &Path) -> Result<Value, std::io::Error> {
    let payload = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&payload).unwrap_or(Value::Null))
}

fn inspect_integrity_state(run_dir: &Path, manifest: &Value) -> &'static str {
    if !run_dir.join("manifest.json").exists()
        || snapshot_path(run_dir).is_none()
        || output_inventory_path(run_dir).is_none()
    {
        return "incomplete";
    }
    if manifest.is_null() {
        return "corrupt";
    }
    let supported = ["run-dir/v0.1", "run/v0.1"];
    if let Some(version) = manifest.get("run_dir_format").and_then(Value::as_str) {
        if !supported.contains(&version) {
            return "unsupported";
        }
    }
    "healthy"
}

pub fn inspect_summary(run_dir: &Path) -> Result<Value, std::io::Error> {
    let manifest = read_json(&run_dir.join("manifest.json"))?;
    let integrity_state = inspect_integrity_state(run_dir, &manifest);
    let traces = read_node_traces(run_dir)?;
    let (retry_total, failed_nodes, failed_classes, failed_node_reasons, cache_hits) =
        traces.iter().fold(
            (
                0usize,
                Vec::<String>::new(),
                std::collections::BTreeSet::<String>::new(),
                Vec::<Value>::new(),
                0usize,
            ),
            |mut acc, (node_id, trace)| {
                if trace.get("status").and_then(Value::as_str) == Some("failed") {
                    acc.1.push(node_id.clone());
                    if let Some(class) = trace_failure_class(trace) {
                        acc.2.insert(class);
                    }
                    if let Some(reason) = trace_failure_reason(node_id, trace) {
                        acc.3.push(reason);
                    }
                }
                if let Some(attempt) = trace.get("attempt").and_then(Value::as_u64) {
                    if attempt > 1 {
                        acc.0 += (attempt - 1) as usize;
                    }
                }
                if trace.get("cache_hit").and_then(Value::as_bool) == Some(true) {
                    acc.4 += 1;
                }
                acc
            },
        );
    let artifact_count = read_outputs_count(run_dir);
    Ok(json!({
        "run_id": manifest.get("run_id").cloned().unwrap_or(Value::Null),
        "status": manifest.get("status").cloned().unwrap_or(Value::Null),
        "submission_source": manifest
            .get("run_metadata")
            .and_then(|m| m.get("submission_source"))
            .cloned()
            .or_else(|| manifest.get("submission_source").cloned())
            .unwrap_or_else(|| json!("manual")),
        "graph_fingerprint": manifest.get("graph_fingerprint").cloned().unwrap_or(Value::Null),
        "timing_ms": {
            "started": manifest.get("started_unix_ms").cloned().unwrap_or(Value::Null),
            "finished": manifest.get("finished_unix_ms").cloned().unwrap_or(Value::Null)
        },
        "node_counts": manifest.get("node_counts").cloned().unwrap_or(Value::Null),
        "retry_count": retry_total,
        "cache_hits": cache_hits,
        "artifact_count": artifact_count,
        "failed_nodes": failed_nodes,
        "failure_classes": failed_classes.into_iter().collect::<Vec<_>>(),
        "failed_node_reasons": failed_node_reasons,
        "run_summary": manifest.get("run_summary").cloned().unwrap_or(Value::Null),
        "integrity_state": integrity_state
    }))
}

pub fn run_completion_summary(run_dir: &Path) -> Result<Value, std::io::Error> {
    let summary = inspect_summary(run_dir)?;
    let run_id = summary.get("run_id").and_then(Value::as_str).unwrap_or("unknown");
    let root =
        run_dir.parent().map(|path| path.display().to_string()).unwrap_or_else(|| ".".to_string());
    let duration_ms = summary
        .get("timing_ms")
        .and_then(Value::as_object)
        .and_then(|timing| {
            Some((timing.get("started")?.as_u64()?, timing.get("finished")?.as_u64()?))
        })
        .and_then(|(started, finished)| finished.checked_sub(started));
    let promoted_artifact_count = summary
        .get("run_summary")
        .and_then(|value| value.get("promoted_outputs"))
        .and_then(Value::as_array)
        .map_or(0, std::vec::Vec::len);
    let failed_node_reasons =
        summary.get("failed_node_reasons").cloned().unwrap_or_else(|| Value::Array(Vec::new()));
    let status = summary.get("status").and_then(Value::as_str).unwrap_or("unknown");
    let integrity_state =
        summary.get("integrity_state").and_then(Value::as_str).unwrap_or("unknown");
    let suggested_next_action = suggested_next_action(run_id, &root, status, integrity_state);

    Ok(json!({
        "run_id": summary.get("run_id").cloned().unwrap_or(Value::Null),
        "status": summary.get("status").cloned().unwrap_or(Value::Null),
        "origin": summary.get("submission_source").cloned().unwrap_or(Value::Null),
        "integrity_state": summary.get("integrity_state").cloned().unwrap_or(Value::Null),
        "duration_ms": duration_ms,
        "node_counts": summary.get("node_counts").cloned().unwrap_or(Value::Null),
        "failed_node_reasons": failed_node_reasons,
        "cache_hits": summary.get("cache_hits").cloned().unwrap_or(Value::Null),
        "promoted_artifact_count": promoted_artifact_count,
        "artifact_count": summary.get("artifact_count").cloned().unwrap_or(Value::Null),
        "suggested_next_action": suggested_next_action,
    }))
}

pub fn explain_run_id(root: &Path, run_id: &str) -> Result<Value, std::io::Error> {
    let run_dir = resolve_run_dir(root, run_id);
    let manifest = read_json(&run_dir.join("manifest.json"))?;
    let run_metadata = manifest.get("run_metadata").cloned().unwrap_or_else(|| json!({}));
    Ok(json!({
        "run_id": manifest.get("run_id").cloned().unwrap_or(json!(run_id)),
        "run_dir": run_dir.display().to_string(),
        "exists": run_dir.exists(),
        "manifest_exists": run_dir.join("manifest.json").exists(),
        "created_unix_ms": manifest.get("created_unix_ms").cloned().unwrap_or(Value::Null),
        "started_unix_ms": manifest.get("started_unix_ms").cloned().unwrap_or(Value::Null),
        "finished_unix_ms": manifest.get("finished_unix_ms").cloned().unwrap_or(Value::Null),
        "submission_source": run_metadata.get("submission_source").cloned().unwrap_or(Value::Null),
        "trigger_source": run_metadata.get("trigger_source").cloned().unwrap_or(Value::Null),
        "parent_run_id": run_metadata.get("parent_run_id").cloned().unwrap_or(Value::Null),
        "source_run_id": run_metadata.get("source_run_id").cloned().unwrap_or(Value::Null),
        "immutability_contract": "run directories are immutable after finalization; aliases must not mutate historical content"
    }))
}

pub fn run_scheduler_checkpoint(run_dir: &Path) -> Result<Value, std::io::Error> {
    let manifest = read_json(&run_dir.join("manifest.json")).unwrap_or(Value::Null);
    let checkpoint_path = run_dir.join("scheduler.checkpoint.json");
    let run_id = manifest
        .get("run_id")
        .and_then(Value::as_str)
        .map(ToOwned::to_owned)
        .or_else(|| inferred_run_id(run_dir))
        .unwrap_or_else(|| "unknown-run".to_string());

    if !checkpoint_path.exists() {
        return Ok(json!({
            "run_id": run_id,
            "run_dir": run_dir.display().to_string(),
            "checkpoint_path": checkpoint_path.display().to_string(),
            "inspection_state": "absent",
            "checkpoint_present": false,
            "detail": "scheduler checkpoint was not retained for this run"
        }));
    }

    let payload = fs::read_to_string(&checkpoint_path)?;
    let checkpoint: ExecutionCheckpoint = match serde_json::from_str(&payload) {
        Ok(checkpoint) => checkpoint,
        Err(error) => {
            return Ok(json!({
                "run_id": run_id,
                "run_dir": run_dir.display().to_string(),
                "checkpoint_path": checkpoint_path.display().to_string(),
                "inspection_state": "corrupt",
                "checkpoint_present": true,
                "detail": "scheduler checkpoint could not be parsed",
                "parse_error": error.to_string(),
            }));
        }
    };

    Ok(json!({
        "run_id": run_id,
        "run_dir": run_dir.display().to_string(),
        "checkpoint_path": checkpoint_path.display().to_string(),
        "inspection_state": "present",
        "checkpoint_present": true,
        "loop_index": checkpoint.loop_index,
        "ready_queue_depth": checkpoint.ready_queue_depth,
        "ready_queue": checkpoint.ready_queue,
        "scheduled_batch": checkpoint.scheduled,
        "inflight_nodes": checkpoint.inflight,
        "resource_blocked_nodes": checkpoint.blocked_by_budget,
        "blocked_reasons": checkpoint.blocked_reasons,
        "completed_statuses": checkpoint.completed_statuses,
        "decision_reason": checkpoint.decision_reason,
        "failure_propagation_mode": checkpoint.failure_propagation_mode,
        "dependency_closure_enabled": checkpoint.dependency_closure_enabled,
        "generated_unix_ms": checkpoint.generated_unix_ms,
    }))
}

pub fn runs_history(root: &Path) -> Result<Value, std::io::Error> {
    runs_history_query_with_selectors(root, None, None, None, None)
}

pub fn runs_history_query(
    root: &Path,
    status_filter: Option<&str>,
    source_filter: Option<&str>,
    pagination: Option<(usize, usize)>,
) -> Result<Value, std::io::Error> {
    runs_history_query_with_filters(root, status_filter, source_filter, None, pagination, None)
}

pub(crate) fn runs_history_query_with_selectors(
    root: &Path,
    status_filter: Option<&str>,
    source_filter: Option<&str>,
    pagination: Option<(usize, usize)>,
    selectors: Option<&[SelectorExpression]>,
) -> Result<Value, std::io::Error> {
    runs_history_query_with_filters(root, status_filter, source_filter, None, pagination, selectors)
}

pub(crate) fn runs_history_query_with_filters(
    root: &Path,
    status_filter: Option<&str>,
    source_filter: Option<&str>,
    graph_filter: Option<&str>,
    pagination: Option<(usize, usize)>,
    selectors: Option<&[SelectorExpression]>,
) -> Result<Value, std::io::Error> {
    let mut rows = load_index_rows(root).unwrap_or(build_history_rows(root)?);

    if let Some(filter_status) = status_filter {
        rows.retain(|row| row.get("status").and_then(Value::as_str) == Some(filter_status));
    }
    if let Some(filter_source) = source_filter {
        rows.retain(|row| {
            row.get("submission_source").and_then(Value::as_str) == Some(filter_source)
        });
    }
    if let Some(filter_graph) = graph_filter {
        rows.retain(|row| row_matches_graph(row, filter_graph));
    }
    if let Some(selector_filters) = selectors {
        rows.retain(|row| {
            selector_filters.iter().all(|selector| selector_matches_row(selector, row))
        });
    }

    let (offset, limit) = pagination.unwrap_or((0, rows.len()));
    let bounded_limit = limit.max(1);
    let total = rows.len();
    let start = offset.min(total);
    let end = (start + bounded_limit).min(total);
    let window = rows[start..end].to_vec();

    if pagination.is_some() {
        return Ok(json!({
            "runs": window,
            "page": {
                "offset": start,
                "limit": bounded_limit,
                "total": total
            }
        }));
    }
    Ok(json!({ "runs": window }))
}

fn build_history_rows(root: &Path) -> Result<Vec<Value>, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut rows = Vec::new();
    for run_id in run_ids {
        let run_dir = resolve_run_dir(root, &run_id);
        rows.push(build_history_row(root, &run_dir, &run_id)?);
    }
    let child_map = build_history_child_map(&rows);
    for row in &mut rows {
        let Some(object) = row.as_object_mut() else {
            continue;
        };
        let run_id = object.get("run_id").and_then(Value::as_str).unwrap_or_default();
        let child_run_ids = child_map.get(run_id).cloned().unwrap_or_default();
        object.insert(
            "lineage".to_string(),
            json!({
                "parent_run_id": object.get("parent_run_id").cloned().unwrap_or(Value::Null),
                "source_run_id": object.get("source_run_id").cloned().unwrap_or(Value::Null),
                "child_run_ids": child_run_ids,
            }),
        );
    }
    rows.sort_by(|left, right| {
        let right_created = right.get("created_unix_ms").and_then(Value::as_u64).unwrap_or(0);
        let left_created = left.get("created_unix_ms").and_then(Value::as_u64).unwrap_or(0);
        right_created.cmp(&left_created).then_with(|| {
            left.get("run_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .cmp(right.get("run_id").and_then(Value::as_str).unwrap_or_default())
        })
    });
    Ok(rows)
}

fn build_history_row(
    root: &Path,
    run_dir: &Path,
    default_run_id: &str,
) -> Result<Value, std::io::Error> {
    let manifest = read_json(&run_dir.join("manifest.json")).ok();
    let graph_snapshot = snapshot_path(run_dir).and_then(|path| read_json(&path).ok());
    let runtime_snapshot = read_json(&run_dir.join("run.snapshot.json")).ok();
    let metadata = manifest
        .as_ref()
        .and_then(|value| value.get("run_metadata"))
        .cloned()
        .unwrap_or_else(|| json!({}));
    let labels = metadata
        .get("labels")
        .and_then(Value::as_array)
        .cloned()
        .or_else(|| {
            runtime_snapshot
                .as_ref()
                .and_then(|value| value.get("labels"))
                .and_then(Value::as_array)
                .cloned()
        })
        .unwrap_or_default();
    let marker_present = run_dir.join(".run-incomplete.json").exists();
    let finished_present = manifest
        .as_ref()
        .and_then(|value| value.get("finished_unix_ms"))
        .is_some_and(|value| !value.is_null());
    let lifecycle_state = if marker_present && !finished_present { "active" } else { "historical" };
    let graph_name = graph_snapshot.as_ref().and_then(history_graph_name).unwrap_or(Value::Null);
    let graph_fingerprint = manifest
        .as_ref()
        .and_then(|value| value.get("graph_fingerprint").cloned())
        .or_else(|| graph_snapshot.as_ref().and_then(history_graph_fingerprint))
        .unwrap_or(Value::Null);
    let run_id = manifest
        .as_ref()
        .and_then(|value| value.get("run_id").cloned())
        .or_else(|| runtime_snapshot.as_ref().and_then(|value| value.get("run_id").cloned()))
        .unwrap_or_else(|| json!(default_run_id));
    let status = manifest
        .as_ref()
        .and_then(|value| value.get("status").cloned())
        .or_else(|| (marker_present || runtime_snapshot.is_some()).then_some(json!("running")))
        .unwrap_or(Value::Null);
    let created_unix_ms = manifest
        .as_ref()
        .and_then(|value| value.get("created_unix_ms").cloned())
        .unwrap_or(Value::Null);
    let submission_source = metadata
        .get("submission_source")
        .cloned()
        .or_else(|| {
            runtime_snapshot.as_ref().and_then(|value| value.get("submission_source").cloned())
        })
        .unwrap_or(Value::Null);
    let trigger_source = metadata
        .get("trigger_source")
        .cloned()
        .or_else(|| {
            runtime_snapshot.as_ref().and_then(|value| value.get("trigger_source").cloned())
        })
        .unwrap_or(Value::Null);
    let parent_run_id = metadata
        .get("parent_run_id")
        .cloned()
        .or_else(|| runtime_snapshot.as_ref().and_then(|value| value.get("parent_run_id").cloned()))
        .unwrap_or(Value::Null);
    let source_run_id = metadata
        .get("source_run_id")
        .cloned()
        .or_else(|| {
            runtime_snapshot.as_ref().and_then(|value| value.get("replay_source_run_id").cloned())
        })
        .unwrap_or(Value::Null);
    let run_dir_display = run_dir.strip_prefix(root).unwrap_or(run_dir).display().to_string();
    let output_location_display = run_dir
        .strip_prefix(root)
        .map(|relative| relative.join("outputs"))
        .unwrap_or_else(|_| run_dir.join("outputs"))
        .display()
        .to_string();

    Ok(json!({
        "run_id": run_id,
        "status": status,
        "lifecycle_state": lifecycle_state,
        "created_unix_ms": created_unix_ms,
        "graph_name": graph_name,
        "graph_fingerprint": graph_fingerprint,
        "parent_run_id": parent_run_id,
        "source_run_id": source_run_id,
        "submission_source": submission_source,
        "trigger_source": trigger_source,
        "run_dir": run_dir_display,
        "output_location": output_location_display,
        "labels": labels
    }))
}

fn build_history_child_map(rows: &[Value]) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut children = std::collections::BTreeMap::<String, Vec<String>>::new();
    for row in rows {
        let Some(child_run_id) = row.get("run_id").and_then(Value::as_str) else {
            continue;
        };
        let Some(parent_run_id) = row.get("parent_run_id").and_then(Value::as_str) else {
            continue;
        };
        children.entry(parent_run_id.to_string()).or_default().push(child_run_id.to_string());
    }
    for child_ids in children.values_mut() {
        child_ids.sort();
    }
    children
}

fn history_graph_name(snapshot: &Value) -> Option<Value> {
    snapshot
        .get("graph")
        .and_then(|value| value.get("meta"))
        .and_then(|value| value.get("name"))
        .cloned()
        .or_else(|| snapshot.get("meta").and_then(|value| value.get("name")).cloned())
}

fn history_graph_fingerprint(snapshot: &Value) -> Option<Value> {
    snapshot.get("graph_fingerprint").cloned()
}

fn row_matches_graph(row: &Value, filter_graph: &str) -> bool {
    row.get("graph_name").and_then(Value::as_str) == Some(filter_graph)
        || row.get("graph_fingerprint").and_then(Value::as_str) == Some(filter_graph)
}

fn load_index_rows(root: &Path) -> Option<Vec<Value>> {
    let path = root.join(RUN_HISTORY_INDEX_FILE);
    let value = read_json(&path).ok()?;
    if value.get("schema").and_then(Value::as_str) != Some("runs-index/v0.1") {
        return None;
    }
    value.get("runs")?.as_array().cloned()
}

pub fn write_run_history_index(root: &Path) -> Result<PathBuf, std::io::Error> {
    fs::create_dir_all(root)?;
    let rows = build_history_rows(root)?;
    let generated_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|value| value.as_millis() as u64)
        .unwrap_or(0);
    let payload = json!({
        "schema": "runs-index/v0.1",
        "generated_unix_ms": generated_unix_ms,
        "runs": rows
    });
    let path = root.join(RUN_HISTORY_INDEX_FILE);
    fs::write(&path, serde_json::to_vec_pretty(&payload)?)?;
    Ok(path)
}

fn selector_matches_row(selector: &SelectorExpression, row: &Value) -> bool {
    match selector.field {
        SelectorField::Run => {
            row.get("run_id").and_then(Value::as_str) == Some(selector.value.as_str())
        }
        SelectorField::Graph => row_matches_graph(row, selector.value.as_str()),
        SelectorField::State => {
            row.get("status").and_then(Value::as_str) == Some(selector.value.as_str())
        }
        SelectorField::Tag => row
            .get("labels")
            .and_then(Value::as_array)
            .map(|labels| {
                labels.iter().any(|label| label.as_str() == Some(selector.value.as_str()))
            })
            .unwrap_or(false),
        SelectorField::Id | SelectorField::IdPrefix => row
            .get("run_id")
            .and_then(Value::as_str)
            .map(|run_id| run_id.starts_with(selector.value.as_str()))
            .unwrap_or(false),
        SelectorField::Node
        | SelectorField::NodePrefix
        | SelectorField::Artifact
        | SelectorField::Branch
        | SelectorField::Attempt
        | SelectorField::Kind => false,
    }
}

pub fn run_tree(run_dir: &Path) -> Result<Value, std::io::Error> {
    let snapshot = read_json(&run_dir.join("snapshot.json"))
        .or_else(|_| read_json(&run_dir.join("run.snapshot.json")))?;
    let nodes = snapshot
        .get("graph")
        .and_then(|g| g.get("nodes"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let edges = snapshot
        .get("graph")
        .and_then(|g| g.get("edges"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let traces = read_node_traces(run_dir)?;
    let mut items = Vec::new();
    for node in nodes {
        let node_id = node.get("id").and_then(Value::as_str).unwrap_or("unknown").to_string();
        let parents: Vec<String> = edges
            .iter()
            .filter_map(|e| {
                if e.get("to").and_then(|to| to.get("node_id")).and_then(Value::as_str)
                    == Some(node_id.as_str())
                {
                    e.get("from")
                        .and_then(|from| from.get("node_id"))
                        .and_then(Value::as_str)
                        .map(|s| s.to_string())
                } else {
                    None
                }
            })
            .collect();
        let status = traces
            .get(&node_id)
            .and_then(|t| t.get("status"))
            .and_then(Value::as_str)
            .unwrap_or("unknown");
        items.push(json!({"node_id": node_id, "parents": parents, "status": status}));
    }
    Ok(json!({"nodes": items}))
}

pub(crate) fn run_timeline_with_query(
    run_dir: &Path,
    query: &RunTimelineQuery,
) -> Result<Value, std::io::Error> {
    if let Ok(payload) = fs::read_to_string(run_dir.join("observability.timeline.json")) {
        if let Ok(mut timeline) = serde_json::from_str::<TimelineExport>(&payload) {
            timeline.entries.sort_by_key(|entry| entry.unix_ms);
            let events = timeline
                .entries
                .into_iter()
                .map(|entry| TimelineEventView {
                    unix_ms: Some(entry.unix_ms),
                    category: entry.category,
                    label: entry.label,
                    node_id: entry.node_id,
                    status: entry.status,
                    reason: entry.reason,
                    source_event: entry.source_event.unwrap_or_else(|| "unknown".to_string()),
                })
                .collect::<Vec<_>>();
            return timeline_query_report("observability_timeline", events, query);
        }
    }

    let traces = read_node_traces(run_dir)?;
    let mut events = Vec::new();
    for (node_id, trace) in traces {
        let start = trace.get("started_unix_ms").and_then(Value::as_u64);
        let finish = trace.get("finished_unix_ms").and_then(Value::as_u64);
        let status = trace.get("status").and_then(Value::as_str).unwrap_or("unknown");
        let cache_hit = trace.get("cache_hit").and_then(Value::as_bool).unwrap_or(false);
        let label = if cache_hit || status == "cached" {
            "node_cached"
        } else {
            match status {
                "failed" => "node_failed",
                "skipped" => "node_skipped",
                "cancelled" => "node_cancelled",
                "success" => "node_completed",
                _ => "node_completed",
            }
        };
        let category = if cache_hit || status == "cached" {
            "cache_hit"
        } else {
            match status {
                "failed" => "failure",
                "skipped" => "skip",
                "cancelled" => "cancel",
                _ => "complete",
            }
        };
        events.push(TimelineEventView {
            unix_ms: finish.map(u128::from).or_else(|| start.map(u128::from)),
            category: category.to_string(),
            label: label.to_string(),
            node_id: Some(node_id),
            status: Some(status.to_string()),
            reason: trace_timeline_reason(&trace),
            source_event: "trace_projection".to_string(),
        });
    }
    events.sort_by_key(|event| event.unix_ms.unwrap_or(0));
    timeline_query_report("node_traces", events, query)
}

pub fn run_timeline(run_dir: &Path) -> Result<Value, std::io::Error> {
    run_timeline_with_query(run_dir, &RunTimelineQuery::default())
}

fn timeline_query_report(
    source: &str,
    events: Vec<TimelineEventView>,
    query: &RunTimelineQuery,
) -> Result<Value, std::io::Error> {
    let total_event_count = events.len();
    let filtered_events = events
        .into_iter()
        .filter(|event| timeline_event_matches_query(event, query))
        .collect::<Vec<_>>();
    let report = TimelineQueryReport {
        source: source.to_string(),
        filters: query.clone(),
        total_event_count,
        matched_event_count: filtered_events.len(),
        events: filtered_events,
    };
    serde_json::to_value(report)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))
}

fn timeline_event_matches_query(event: &TimelineEventView, query: &RunTimelineQuery) -> bool {
    if let Some(node_filter) = query.node.as_deref() {
        if event.node_id.as_deref() != Some(node_filter) {
            return false;
        }
    }
    if let Some(event_filter) = query.event.as_ref() {
        let expected = event_filter.to_ascii_lowercase();
        let label = event.label.to_ascii_lowercase();
        let category = event.category.to_ascii_lowercase();
        let source_event = event.source_event.to_ascii_lowercase();
        if label != expected && category != expected && source_event != expected {
            return false;
        }
    }
    if let Some(since_unix_ms) = query.since_unix_ms {
        if event.unix_ms.is_none_or(|unix_ms| unix_ms < since_unix_ms) {
            return false;
        }
    }
    if let Some(until_unix_ms) = query.until_unix_ms {
        if event.unix_ms.is_none_or(|unix_ms| unix_ms > until_unix_ms) {
            return false;
        }
    }
    true
}

fn trace_timeline_reason(trace: &Value) -> Option<String> {
    trace
        .get("skip_reason")
        .and_then(|value| value.get("reason"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| trace.get("transition_cause").and_then(Value::as_str).map(ToString::to_string))
        .or_else(|| {
            trace
                .get("failure")
                .and_then(|value| value.get("code"))
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

pub fn doctor_run(run_dir: &Path) -> Value {
    let mut findings = Vec::new();
    if !run_dir.join("manifest.json").exists() {
        findings.push("missing manifest.json".to_string());
    }
    if snapshot_path(run_dir).is_none() {
        findings.push("missing graph snapshot".to_string());
    }
    if output_inventory_path(run_dir).is_none() {
        findings.push("missing outputs index".to_string());
    }
    let manifest = read_json(&run_dir.join("manifest.json")).unwrap_or(Value::Null);
    let expected_trace_nodes = manifest
        .get("node_counts")
        .and_then(Value::as_object)
        .map(|counts| counts.values().filter_map(Value::as_u64).sum::<u64>() as usize)
        .unwrap_or(0);
    let observed_trace_nodes = read_node_traces(run_dir).map(|traces| traces.len()).unwrap_or(0);
    if expected_trace_nodes > 0 && observed_trace_nodes == 0 {
        findings.push(
            "missing node traces referenced by manifest node_counts (expected non-zero traces)"
                .to_string(),
        );
    }
    json!({
        "status": if findings.is_empty() { "ok" } else { "corrupt" },
        "findings": findings
    })
}

pub fn runs_summary(root: &Path) -> Result<Value, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut statuses = std::collections::BTreeMap::<String, usize>::new();
    let mut total_retries = 0usize;
    let mut total_cache_hits = 0usize;
    let mut total_artifacts = 0usize;
    let mut replay_equivalent_runs = 0usize;
    let mut failed_run_count = 0usize;
    for run_id in &run_ids {
        let summary = inspect_summary(&resolve_run_dir(root, run_id))?;
        let status = summary.get("status").and_then(Value::as_str).unwrap_or("unknown").to_string();
        *statuses.entry(status).or_insert(0) += 1;
        if summary.get("status").and_then(Value::as_str) == Some("failed") {
            failed_run_count += 1;
        }
        if summary.get("retry_count").and_then(Value::as_u64).unwrap_or(0) == 0 {
            replay_equivalent_runs += 1;
        }
        total_retries += summary.get("retry_count").and_then(Value::as_u64).unwrap_or(0) as usize;
        total_cache_hits += summary.get("cache_hits").and_then(Value::as_u64).unwrap_or(0) as usize;
        total_artifacts +=
            summary.get("artifact_count").and_then(Value::as_u64).unwrap_or(0) as usize;
    }
    let run_count = run_ids.len();
    let exact_reports = json!({
        "failure_distribution": statuses,
        "cache_usefulness": {
            "total_cache_hits": total_cache_hits,
            "average_cache_hits_per_run": if run_count == 0 { 0.0 } else { total_cache_hits as f64 / run_count as f64 }
        },
        "replay_equivalence": {
            "replay_equivalent_runs": replay_equivalent_runs,
            "run_count": run_count
        },
        "determinism": {
            "failed_runs": failed_run_count,
            "success_runs": run_count.saturating_sub(failed_run_count)
        }
    });
    Ok(json!({
        "runs": run_count,
        "total_retries": total_retries,
        "total_cache_hits": total_cache_hits,
        "total_artifacts": total_artifacts
        ,"reports": exact_reports
    }))
}

pub fn runs_trend(root: &Path) -> Result<Value, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut points = Vec::new();
    for run_id in run_ids {
        let summary = inspect_summary(&resolve_run_dir(root, &run_id))?;
        points.push(json!({
            "run_id": run_id,
            "retry_count": summary.get("retry_count").cloned().unwrap_or(Value::Null),
            "cache_hits": summary.get("cache_hits").cloned().unwrap_or(Value::Null),
            "artifact_count": summary.get("artifact_count").cloned().unwrap_or(Value::Null),
            "status": summary.get("status").cloned().unwrap_or(Value::Null)
        }));
    }
    Ok(json!({"series": points}))
}

pub fn runs_failures(root: &Path) -> Result<Value, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut counts = std::collections::BTreeMap::<String, usize>::new();
    for run_id in run_ids {
        let run_dir = resolve_run_dir(root, &run_id);
        let traces = read_node_traces(&run_dir)?;
        for (_node_id, trace) in traces {
            if trace.get("status").and_then(Value::as_str) == Some("failed") {
                let kind = trace
                    .get("failure")
                    .and_then(|f| f.get("kind"))
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();
                *counts.entry(kind).or_insert(0) += 1;
            }
        }
    }
    Ok(json!({"failure_distribution": counts}))
}

pub fn runs_flakes(root: &Path) -> Result<Value, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut by_graph = std::collections::BTreeMap::<String, Vec<String>>::new();
    for run_id in &run_ids {
        let summary = inspect_summary(&resolve_run_dir(root, run_id))?;
        let graph = summary
            .get("graph_fingerprint")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let status = summary.get("status").and_then(Value::as_str).unwrap_or("unknown").to_string();
        by_graph.entry(graph).or_default().push(status);
    }
    let mut flaky = Vec::new();
    for (graph, statuses) in by_graph {
        let uniq: std::collections::BTreeSet<_> = statuses.iter().collect();
        if uniq.len() > 1 {
            flaky.push(json!({"graph_fingerprint": graph, "statuses": statuses}));
        }
    }
    Ok(json!({"flakes": flaky}))
}

pub fn render_run_summary(summary: &Value) -> String {
    let origin = match summary.get("submission_source").and_then(Value::as_str).unwrap_or("manual")
    {
        "import" => "imported",
        _ => "native",
    };
    let failure_classes = summary
        .get("failure_classes")
        .and_then(Value::as_array)
        .map(|classes| classes.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(","))
        .filter(|classes| !classes.is_empty());
    let status = summary.get("status").unwrap_or(&Value::Null);
    let status_rendered = match failure_classes {
        Some(classes) => format!("{status} [{classes}]"),
        None => status.to_string(),
    };
    format!(
        "note: human output is operator-facing; use --json for stable automation\nrun_id: {}\nstatus: {}\norigin: {}\nintegrity_state: {}\nretry_count: {}\ncache_hits: {}\nartifact_count: {}",
        summary.get("run_id").unwrap_or(&Value::Null),
        status_rendered,
        origin,
        summary.get("integrity_state").unwrap_or(&Value::Null),
        summary.get("retry_count").unwrap_or(&Value::Null),
        summary.get("cache_hits").unwrap_or(&Value::Null),
        summary.get("artifact_count").unwrap_or(&Value::Null),
    )
}

pub fn format_inspect_human(summary: &Value) -> String {
    render_run_summary(summary)
}

pub fn format_show_human(summary: &Value) -> String {
    format!(
        "{}\ntiming_ms: {}",
        render_run_summary(summary),
        summary.get("timing_ms").unwrap_or(&Value::Null),
    )
}

pub fn format_run_completion_human(summary: &Value) -> String {
    let next_action = summary
        .get("suggested_next_action")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default();
    let failed_nodes =
        summary.get("failed_node_reasons").and_then(Value::as_array).cloned().unwrap_or_default();
    let failed_render = if failed_nodes.is_empty() {
        "[]".to_string()
    } else {
        serde_json::to_string(&failed_nodes).unwrap_or_else(|_| "[]".to_string())
    };
    format!(
        "run_summary_status: {}\nrun_summary_duration_ms: {}\nrun_summary_node_counts: {}\nrun_summary_failed_nodes: {}\nrun_summary_cache_hits: {}\nrun_summary_artifact_count: {}\nrun_summary_promoted_artifact_count: {}\nrun_summary_next_action: {}\nrun_summary_next_command: {}",
        summary.get("status").unwrap_or(&Value::Null),
        summary.get("duration_ms").unwrap_or(&Value::Null),
        summary.get("node_counts").unwrap_or(&Value::Null),
        failed_render,
        summary.get("cache_hits").unwrap_or(&Value::Null),
        summary.get("artifact_count").unwrap_or(&Value::Null),
        summary.get("promoted_artifact_count").unwrap_or(&Value::Null),
        next_action.get("reason").cloned().unwrap_or(Value::Null),
        next_action.get("command").cloned().unwrap_or(Value::Null),
    )
}

fn inferred_run_id(run_dir: &Path) -> Option<String> {
    run_dir
        .file_name()
        .and_then(|value| value.to_str())
        .map(|value| value.trim_start_matches("run.tmp-").trim_start_matches("run-").to_string())
}

fn output_inventory_path(run_dir: &Path) -> Option<PathBuf> {
    let nested = run_dir.join("outputs").join("index.json");
    if nested.exists() {
        return Some(nested);
    }
    let legacy = run_dir.join("outputs.index.json");
    if legacy.exists() {
        return Some(legacy);
    }
    None
}

fn snapshot_path(run_dir: &Path) -> Option<PathBuf> {
    let current = run_dir.join("graph.snapshot.json");
    if current.exists() {
        return Some(current);
    }
    let legacy = run_dir.join("snapshot.json");
    if legacy.exists() {
        return Some(legacy);
    }
    None
}

fn read_outputs_count(run_dir: &Path) -> usize {
    let Some(path) = output_inventory_path(run_dir) else {
        return 0;
    };
    let payload = fs::read_to_string(path).ok();
    payload
        .and_then(|p| serde_json::from_str::<Value>(&p).ok())
        .and_then(|v| {
            v.get("files")
                .and_then(Value::as_array)
                .map(|arr| arr.len())
                .or_else(|| v.get("outputs").and_then(Value::as_array).map(|arr| arr.len()))
        })
        .unwrap_or(0)
}

fn suggested_next_action(run_id: &str, root: &str, status: &str, integrity_state: &str) -> Value {
    if integrity_state != "healthy" {
        return json!({
            "action": "doctor-run",
            "reason": "run artifacts are incomplete or corrupt and should be checked before reuse",
            "command": format!("bijux-dag runs doctor {run_id} --root {root}")
        });
    }
    match status {
        "failed" => json!({
            "action": "explain-failure",
            "reason": "failed nodes should be classified before rerun or replay decisions",
            "command": format!("bijux-dag runs explain-failure {run_id} --root {root}")
        }),
        "cancelled" => json!({
            "action": "inspect-run",
            "reason": "cancelled runs should be inspected before resume or replay",
            "command": format!("bijux-dag runs inspect {run_id} --root {root}")
        }),
        "success" | "cached" => json!({
            "action": "inspect-run",
            "reason": "successful runs should be inspected before replay, diff, or promotion",
            "command": format!("bijux-dag runs inspect {run_id} --root {root}")
        }),
        _ => json!({
            "action": "inspect-run",
            "reason": "inspect the finalized run evidence before taking follow-up actions",
            "command": format!("bijux-dag runs inspect {run_id} --root {root}")
        }),
    }
}

fn trace_failure_reason(node_id: &str, trace: &Value) -> Option<Value> {
    let failure = trace.get("failure")?.clone();
    let parsed: FailureInfo = serde_json::from_value(failure).ok()?;
    Some(json!({
        "node_id": node_id,
        "class": parsed.operator_class().as_str(),
        "code": parsed.code,
        "message": parsed.message,
    }))
}

fn trace_failure_class(trace: &Value) -> Option<String> {
    let failure = trace.get("failure")?.clone();
    let parsed: FailureInfo = serde_json::from_value(failure).ok()?;
    Some(parsed.operator_class().as_str().to_string())
}

fn read_node_traces(
    run_dir: &Path,
) -> Result<std::collections::BTreeMap<String, Value>, std::io::Error> {
    let mut map = std::collections::BTreeMap::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(map);
    }
    for entry in fs::read_dir(nodes_dir)? {
        let entry = entry?;
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        if !trace_path.exists() {
            continue;
        }
        let payload = fs::read_to_string(trace_path)?;
        let trace: Value = serde_json::from_str(&payload).unwrap_or(Value::Null);
        map.insert(node_id, trace);
    }
    Ok(map)
}

#[cfg(test)]
mod tests {
    use super::{
        run_completion_summary, run_timeline_with_query, runs_history_query_with_filters,
        runs_history_query_with_selectors, RunTimelineQuery,
    };
    use crate::routes::selector_grammar::parse_selector_expressions;
    use serde_json::json;

    fn write_manifest(root: &std::path::Path, run_id: &str, source: &str, labels: &[&str]) {
        let run_dir = root.join(run_id);
        std::fs::create_dir_all(&run_dir).expect("mkdir");
        std::fs::write(
            run_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": run_id,
                "status": "success",
                "created_unix_ms": 1,
                "run_metadata": {
                    "submission_source": source,
                    "trigger_source": "cli",
                    "labels": labels
                }
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
    }

    fn write_timeline_fixture(root: &std::path::Path, run_id: &str) -> std::path::PathBuf {
        let run_dir = root.join(run_id);
        std::fs::create_dir_all(run_dir.join("nodes").join("worker")).expect("mkdir worker");
        std::fs::write(
            run_dir.join("observability.timeline.json"),
            serde_json::to_vec_pretty(&json!({
                "schema_version": "v0.1",
                "entries": [
                    {
                        "unix_ms": 100u64,
                        "category": "run",
                        "label": "run_started",
                        "node_id": null,
                        "source_event": "run_started"
                    },
                    {
                        "unix_ms": 110u64,
                        "category": "ready",
                        "label": "node_ready",
                        "node_id": "worker",
                        "source_event": "node_ready"
                    },
                    {
                        "unix_ms": 130u64,
                        "category": "failure",
                        "label": "node_failed",
                        "node_id": "worker",
                        "status": "failed",
                        "reason": "execution_failed",
                        "source_event": "node_finished"
                    },
                    {
                        "unix_ms": 140u64,
                        "category": "run",
                        "label": "run_completed",
                        "node_id": null,
                        "source_event": "run_finished"
                    }
                ]
            }))
            .expect("timeline"),
        )
        .expect("write timeline");
        run_dir
    }

    fn write_trace_only_timeline_fixture(
        root: &std::path::Path,
        run_id: &str,
    ) -> std::path::PathBuf {
        let run_dir = root.join(run_id);
        std::fs::create_dir_all(run_dir.join("nodes").join("worker")).expect("mkdir worker");
        std::fs::write(
            run_dir.join("nodes").join("worker").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "status": "failed",
                "started_unix_ms": 200u64,
                "finished_unix_ms": 250u64,
                "attempt": 1,
                "transition_cause": "ExecutionFailed",
                "failure": {
                    "code": "EXEC_FAIL",
                    "kind": "Execution",
                    "message": "tool failed"
                }
            }))
            .expect("trace"),
        )
        .expect("write trace");
        run_dir
    }

    #[test]
    fn history_query_applies_selector_filters_with_pagination() {
        let temp = tempfile::tempdir().expect("tmp");
        write_manifest(temp.path(), "run-a", "manual", &["etl"]);
        write_manifest(temp.path(), "run-b", "imported", &["etl", "imported"]);
        let selectors =
            parse_selector_expressions(&["run:run-b".to_string(), "tag:etl".to_string()])
                .expect("selectors");
        let report = runs_history_query_with_selectors(
            temp.path(),
            Some("success"),
            Some("imported"),
            Some((0, 10)),
            Some(selectors.as_slice()),
        )
        .expect("history");
        let runs = report["runs"].as_array().expect("runs");
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0]["run_id"], "run-b");
        assert_eq!(report["page"]["total"], 1);
    }

    #[test]
    fn timeline_query_filters_persisted_timeline_by_node_event_and_time() {
        let temp = tempfile::tempdir().expect("tmp");
        let run_dir = write_timeline_fixture(temp.path(), "run-filtered");
        let report = run_timeline_with_query(
            &run_dir,
            &RunTimelineQuery {
                node: Some("worker".to_string()),
                event: Some("node_failed".to_string()),
                since_unix_ms: Some(120),
                until_unix_ms: Some(135),
            },
        )
        .expect("timeline");

        assert_eq!(report["source"], "observability_timeline");
        assert_eq!(report["total_event_count"], 4);
        assert_eq!(report["matched_event_count"], 1);
        let events = report["events"].as_array().expect("events");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["label"], "node_failed");
        assert_eq!(events[0]["reason"], "execution_failed");
    }

    #[test]
    fn timeline_query_matches_source_event_names_for_persisted_timelines() {
        let temp = tempfile::tempdir().expect("tmp");
        let run_dir = write_timeline_fixture(temp.path(), "run-source-match");
        let report = run_timeline_with_query(
            &run_dir,
            &RunTimelineQuery {
                event: Some("run_finished".to_string()),
                ..RunTimelineQuery::default()
            },
        )
        .expect("timeline");

        assert_eq!(report["matched_event_count"], 1);
        let events = report["events"].as_array().expect("events");
        assert_eq!(events[0]["label"], "run_completed");
        assert_eq!(events[0]["source_event"], "run_finished");
    }

    #[test]
    fn timeline_query_falls_back_to_trace_projection_with_reason() {
        let temp = tempfile::tempdir().expect("tmp");
        let run_dir = write_trace_only_timeline_fixture(temp.path(), "run-trace-only");
        let report = run_timeline_with_query(
            &run_dir,
            &RunTimelineQuery {
                event: Some("node_failed".to_string()),
                ..RunTimelineQuery::default()
            },
        )
        .expect("timeline");

        assert_eq!(report["source"], "node_traces");
        assert_eq!(report["matched_event_count"], 1);
        let events = report["events"].as_array().expect("events");
        assert_eq!(events[0]["unix_ms"], 250);
        assert_eq!(events[0]["reason"], "ExecutionFailed");
        assert_eq!(events[0]["source_event"], "trace_projection");
    }

    #[test]
    fn history_query_filters_by_graph_and_marks_active_runs() {
        let temp = tempfile::tempdir().expect("tmp");
        let active = temp.path().join("run-active");
        std::fs::create_dir_all(active.join("outputs")).expect("mkdir outputs");
        std::fs::write(
            active.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": "run-active",
                "status": "running",
                "created_unix_ms": 20,
                "started_unix_ms": 21,
                "graph_fingerprint": "graph-train",
                "run_metadata": {
                    "submission_source": "manual",
                    "trigger_source": "cli",
                    "parent_run_id": "run-parent",
                    "source_run_id": "run-parent",
                    "labels": ["train"]
                }
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        std::fs::write(
            active.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph_fingerprint": "graph-train",
                "graph": {"meta": {"name": "training-pipeline"}, "nodes": [], "edges": []}
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        std::fs::write(active.join("outputs").join("index.json"), "{\"files\":[]}")
            .expect("outputs");
        std::fs::write(active.join(".run-incomplete.json"), r#"{"reason":"run not finalized"}"#)
            .expect("incomplete marker");

        let historical = temp.path().join("run-parent");
        std::fs::create_dir_all(historical.join("outputs")).expect("mkdir outputs");
        std::fs::write(
            historical.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": "run-parent",
                "status": "success",
                "created_unix_ms": 10,
                "started_unix_ms": 11,
                "finished_unix_ms": 12,
                "graph_fingerprint": "graph-train",
                "run_metadata": {
                    "submission_source": "manual",
                    "trigger_source": "cli",
                    "labels": ["train"]
                }
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        std::fs::write(
            historical.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph_fingerprint": "graph-train",
                "graph": {"meta": {"name": "training-pipeline"}, "nodes": [], "edges": []}
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        std::fs::write(historical.join("outputs").join("index.json"), "{\"files\":[]}")
            .expect("outputs");

        let report = runs_history_query_with_filters(
            temp.path(),
            Some("running"),
            None,
            Some("training-pipeline"),
            None,
            None,
        )
        .expect("history report");
        let rows = report["runs"].as_array().expect("rows");
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["run_id"], "run-active");
        assert_eq!(rows[0]["lifecycle_state"], "active");
        assert_eq!(rows[0]["graph_fingerprint"], "graph-train");
        assert_eq!(rows[0]["run_dir"], json!("run-active"));
        assert_eq!(rows[0]["output_location"], json!("run-active/outputs"));
    }

    #[test]
    fn history_rows_sort_recent_first_and_surface_child_lineage() {
        let temp = tempfile::tempdir().expect("tmp");

        for (run_id, created, parent_run_id) in [
            ("run-parent", 10_u64, None),
            ("run-child", 30_u64, Some("run-parent")),
            ("run-middle", 20_u64, None),
        ] {
            let run_dir = temp.path().join(run_id);
            std::fs::create_dir_all(run_dir.join("outputs")).expect("mkdir outputs");
            std::fs::write(
                run_dir.join("manifest.json"),
                serde_json::to_vec_pretty(&json!({
                    "run_id": run_id,
                    "status": "success",
                    "created_unix_ms": created,
                    "started_unix_ms": created + 1,
                    "finished_unix_ms": created + 2,
                    "graph_fingerprint": "graph-train",
                    "run_metadata": {
                        "submission_source": "manual",
                        "trigger_source": "cli",
                        "parent_run_id": parent_run_id,
                        "source_run_id": parent_run_id,
                        "labels": []
                    }
                }))
                .expect("manifest"),
            )
            .expect("write manifest");
            std::fs::write(
                run_dir.join("graph.snapshot.json"),
                serde_json::to_vec_pretty(&json!({
                    "graph_fingerprint": "graph-train",
                    "graph": {"meta": {"name": "training-pipeline"}, "nodes": [], "edges": []}
                }))
                .expect("snapshot"),
            )
            .expect("write snapshot");
            std::fs::write(run_dir.join("outputs").join("index.json"), "{\"files\":[]}")
                .expect("outputs");
        }

        let report = runs_history_query_with_filters(
            temp.path(),
            None,
            None,
            Some("graph-train"),
            None,
            None,
        )
        .expect("history report");
        let rows = report["runs"].as_array().expect("rows");
        assert_eq!(rows[0]["run_id"], "run-child");
        assert_eq!(rows[1]["run_id"], "run-middle");
        assert_eq!(rows[2]["run_id"], "run-parent");
        assert_eq!(rows[2]["lineage"]["child_run_ids"], json!(["run-child"]));
    }

    #[test]
    fn completion_summary_reads_nested_output_index_and_success_next_action() {
        let temp = tempfile::tempdir().expect("tmp");
        let run_dir = temp.path().join("run-success");
        std::fs::create_dir_all(run_dir.join("outputs")).expect("mkdir outputs");
        std::fs::write(
            run_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": "run-success",
                "status": "success",
                "started_unix_ms": 100,
                "finished_unix_ms": 180,
                "node_counts": {"success": 2, "failed": 0, "skipped": 0, "cached": 1, "cancelled": 0},
                "run_metadata": {"submission_source": "manual"},
                "run_summary": {
                    "promoted_outputs": [
                        {"artifact_id": "deliverable:report", "output_name": "report"}
                    ]
                }
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        std::fs::write(run_dir.join("snapshot.json"), "{}").expect("write snapshot");
        std::fs::write(
            run_dir.join("outputs").join("index.json"),
            r#"{"files":[{"name":"report"},{"name":"metrics"}]}"#,
        )
        .expect("write outputs index");

        let summary = run_completion_summary(&run_dir).expect("completion summary");
        assert_eq!(summary["duration_ms"], 80);
        assert_eq!(summary["artifact_count"], 2);
        assert_eq!(summary["promoted_artifact_count"], 1);
        assert_eq!(summary["suggested_next_action"]["action"], "inspect-run");
        assert!(summary["suggested_next_action"]["command"]
            .as_str()
            .is_some_and(|command| command.contains("bijux-dag runs inspect")));
    }

    #[test]
    fn completion_summary_surfaces_failure_reasons_and_doctor_action_for_incomplete_run() {
        let temp = tempfile::tempdir().expect("tmp");
        let run_dir = temp.path().join("run-failed");
        std::fs::create_dir_all(run_dir.join("nodes").join("transform")).expect("mkdir nodes");
        std::fs::write(
            run_dir.join("manifest.json"),
            serde_json::to_vec_pretty(&json!({
                "run_id": "run-failed",
                "status": "failed",
                "started_unix_ms": 100,
                "finished_unix_ms": 150,
                "node_counts": {"success": 0, "failed": 1, "skipped": 0, "cached": 0, "cancelled": 0},
                "run_metadata": {"submission_source": "manual"}
            }))
            .expect("manifest"),
        )
        .expect("write manifest");
        std::fs::write(
            run_dir.join("nodes").join("transform").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "status": "failed",
                "attempt": 1,
                "failure": {
                    "class": "execution",
                    "kind": "Execution",
                    "code": "EXEC_ERROR",
                    "message": "tool exited non-zero"
                }
            }))
            .expect("trace"),
        )
        .expect("write trace");

        let summary = run_completion_summary(&run_dir).expect("completion summary");
        let reasons = summary["failed_node_reasons"].as_array().expect("reason list");
        assert_eq!(reasons.len(), 1);
        assert_eq!(reasons[0]["node_id"], "transform");
        assert_eq!(reasons[0]["code"], "EXEC_ERROR");
        assert_eq!(summary["integrity_state"], "incomplete");
        assert_eq!(summary["suggested_next_action"]["action"], "doctor-run");
    }
}
