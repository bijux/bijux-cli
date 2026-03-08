use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub fn resolve_run_dir(root: &Path, run_id: &str) -> PathBuf {
    root.join(run_id)
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
    let required = ["manifest.json", "snapshot.json", "outputs.index.json"];
    if required.iter().any(|rel| !run_dir.join(rel).exists()) {
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
    let (retry_total, failed_nodes, cache_hits) = traces.iter().fold(
        (0usize, Vec::<String>::new(), 0usize),
        |mut acc, (node_id, trace)| {
            if trace.get("status").and_then(Value::as_str) == Some("failed") {
                acc.1.push(node_id.clone());
            }
            if let Some(attempt) = trace.get("attempt").and_then(Value::as_u64) {
                if attempt > 1 {
                    acc.0 += (attempt - 1) as usize;
                }
            }
            if trace.get("cache_hit").and_then(Value::as_bool) == Some(true) {
                acc.2 += 1;
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
        "integrity_state": integrity_state
    }))
}

pub fn explain_run_id(root: &Path, run_id: &str) -> Result<Value, std::io::Error> {
    let run_dir = resolve_run_dir(root, run_id);
    let manifest = read_json(&run_dir.join("manifest.json"))?;
    let run_metadata = manifest
        .get("run_metadata")
        .cloned()
        .unwrap_or_else(|| json!({}));
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

pub fn runs_history(root: &Path) -> Result<Value, std::io::Error> {
    runs_history_query(root, None, None, None)
}

pub fn runs_history_query(
    root: &Path,
    status_filter: Option<&str>,
    source_filter: Option<&str>,
    pagination: Option<(usize, usize)>,
) -> Result<Value, std::io::Error> {
    let run_ids = list_runs(root)?;
    let mut rows = Vec::new();
    for run_id in run_ids {
        let run_dir = resolve_run_dir(root, &run_id);
        let manifest = read_json(&run_dir.join("manifest.json"))?;
        let metadata = manifest
            .get("run_metadata")
            .cloned()
            .unwrap_or_else(|| json!({}));
        let row = json!({
            "run_id": manifest.get("run_id").cloned().unwrap_or(json!(run_id)),
            "status": manifest.get("status").cloned().unwrap_or(Value::Null),
            "created_unix_ms": manifest.get("created_unix_ms").cloned().unwrap_or(Value::Null),
            "parent_run_id": metadata.get("parent_run_id").cloned().unwrap_or(Value::Null),
            "source_run_id": metadata.get("source_run_id").cloned().unwrap_or(Value::Null),
            "submission_source": metadata.get("submission_source").cloned().unwrap_or(Value::Null),
            "trigger_source": metadata.get("trigger_source").cloned().unwrap_or(Value::Null)
        });
        rows.push(row);
    }

    if let Some(filter_status) = status_filter {
        rows.retain(|row| row.get("status").and_then(Value::as_str) == Some(filter_status));
    }
    if let Some(filter_source) = source_filter {
        rows.retain(|row| row.get("submission_source").and_then(Value::as_str) == Some(filter_source));
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
        let node_id = node
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        let parents: Vec<String> = edges
            .iter()
            .filter_map(|e| {
                if e.get("to")
                    .and_then(|to| to.get("node_id"))
                    .and_then(Value::as_str)
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

pub fn run_timeline(run_dir: &Path) -> Result<Value, std::io::Error> {
    let traces = read_node_traces(run_dir)?;
    let mut events = Vec::new();
    for (node_id, trace) in traces {
        let start = trace.get("started_unix_ms").cloned().unwrap_or(Value::Null);
        let finish = trace
            .get("finished_unix_ms")
            .cloned()
            .unwrap_or(Value::Null);
        let status = trace.get("status").cloned().unwrap_or(Value::Null);
        let attempt = trace.get("attempt").and_then(Value::as_u64).unwrap_or(1);
        let cache_hit = trace
            .get("cache_hit")
            .and_then(Value::as_bool)
            .unwrap_or(false);
        let event_kind = if cache_hit {
            "cache_hit"
        } else if attempt > 1 {
            "retry"
        } else {
            "execution"
        };
        events.push(json!({
            "node_id": node_id,
            "started_unix_ms": start,
            "finished_unix_ms": finish,
            "status": status,
            "attempt": attempt,
            "cache_hit": cache_hit,
            "event_kind": event_kind
        }));
    }
    events.sort_by_key(|e| {
        e.get("started_unix_ms")
            .and_then(Value::as_u64)
            .unwrap_or(0)
    });
    Ok(json!({"events": events}))
}

pub fn explain_failure(run_dir: &Path) -> Result<Value, std::io::Error> {
    let traces = read_node_traces(run_dir)?;
    let mut failed = Vec::new();
    let mut skipped = Vec::new();
    for (node_id, trace) in traces {
        match trace
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
        {
            "failed" => failed.push(node_id),
            "skipped" => skipped.push(node_id),
            _ => {}
        }
    }
    let root = failed.first().cloned();
    Ok(json!({
        "root_failure": root,
        "failed_nodes": failed,
        "propagated_or_skipped_nodes": skipped
    }))
}

pub fn doctor_run(run_dir: &Path) -> Value {
    let mut findings = Vec::new();
    for rel in ["manifest.json", "snapshot.json", "outputs.index.json"] {
        if !run_dir.join(rel).exists() {
            findings.push(format!("missing {rel}"));
        }
    }
    let manifest = read_json(&run_dir.join("manifest.json")).unwrap_or(Value::Null);
    let expected_trace_nodes = manifest
        .get("node_counts")
        .and_then(Value::as_object)
        .map(|counts| {
            counts
                .values()
                .filter_map(Value::as_u64)
                .sum::<u64>() as usize
        })
        .unwrap_or(0);
    let observed_trace_nodes = read_node_traces(run_dir)
        .map(|traces| traces.len())
        .unwrap_or(0);
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
        let status = summary
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
        *statuses.entry(status).or_insert(0) += 1;
        if summary.get("status").and_then(Value::as_str) == Some("failed") {
            failed_run_count += 1;
        }
        if summary
            .get("retry_count")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == 0
        {
            replay_equivalent_runs += 1;
        }
        total_retries += summary
            .get("retry_count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        total_cache_hits += summary
            .get("cache_hits")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
        total_artifacts += summary
            .get("artifact_count")
            .and_then(Value::as_u64)
            .unwrap_or(0) as usize;
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

pub fn runs_compare(root: &Path, run_a: &str, run_b: &str) -> Result<Value, std::io::Error> {
    let a = inspect_summary(&resolve_run_dir(root, run_a))?;
    let b = inspect_summary(&resolve_run_dir(root, run_b))?;
    Ok(json!({
        "run_a": run_a,
        "run_b": run_b,
        "status": {"a": a.get("status"), "b": b.get("status")},
        "retries": {"a": a.get("retry_count"), "b": b.get("retry_count")},
        "cache_hits": {"a": a.get("cache_hits"), "b": b.get("cache_hits")},
        "artifact_count": {"a": a.get("artifact_count"), "b": b.get("artifact_count")},
        "timing_ms": {"a": a.get("timing_ms"), "b": b.get("timing_ms")}
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
        let status = summary
            .get("status")
            .and_then(Value::as_str)
            .unwrap_or("unknown")
            .to_string();
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
    let origin = match summary
        .get("submission_source")
        .and_then(Value::as_str)
        .unwrap_or("manual")
    {
        "import" => "imported",
        _ => "native",
    };
    format!(
        "run_id: {}\nstatus: {}\norigin: {}\nintegrity_state: {}\nretry_count: {}\ncache_hits: {}\nartifact_count: {}",
        summary.get("run_id").unwrap_or(&Value::Null),
        summary.get("status").unwrap_or(&Value::Null),
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

fn read_outputs_count(run_dir: &Path) -> usize {
    let path = run_dir.join("outputs.index.json");
    let payload = fs::read_to_string(path).ok();
    payload
        .and_then(|p| serde_json::from_str::<Value>(&p).ok())
        .and_then(|v| {
            v.get("files")
                .and_then(Value::as_array)
                .map(|arr| arr.len())
                .or_else(|| {
                    v.get("outputs")
                        .and_then(Value::as_array)
                        .map(|arr| arr.len())
                })
        })
        .unwrap_or(0)
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
