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

pub fn inspect_summary(run_dir: &Path) -> Result<Value, std::io::Error> {
    let manifest = read_json(&run_dir.join("manifest.json"))?;
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
        "graph_fingerprint": manifest.get("graph_fingerprint").cloned().unwrap_or(Value::Null),
        "timing_ms": {
            "started": manifest.get("started_unix_ms").cloned().unwrap_or(Value::Null),
            "finished": manifest.get("finished_unix_ms").cloned().unwrap_or(Value::Null)
        },
        "node_counts": manifest.get("node_counts").cloned().unwrap_or(Value::Null),
        "retry_count": retry_total,
        "cache_hits": cache_hits,
        "artifact_count": artifact_count,
        "failed_nodes": failed_nodes
    }))
}

pub fn run_tree(run_dir: &Path) -> Result<Value, std::io::Error> {
    let snapshot = read_json(&run_dir.join("snapshot.json"))?;
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
        let finish = trace.get("finished_unix_ms").cloned().unwrap_or(Value::Null);
        let status = trace.get("status").cloned().unwrap_or(Value::Null);
        events.push(json!({
            "node_id": node_id,
            "started_unix_ms": start,
            "finished_unix_ms": finish,
            "status": status
        }));
    }
    events.sort_by_key(|e| e.get("started_unix_ms").and_then(Value::as_u64).unwrap_or(0));
    Ok(json!({"events": events}))
}

pub fn explain_failure(run_dir: &Path) -> Result<Value, std::io::Error> {
    let traces = read_node_traces(run_dir)?;
    let mut failed = Vec::new();
    let mut skipped = Vec::new();
    for (node_id, trace) in traces {
        match trace.get("status").and_then(Value::as_str).unwrap_or("unknown") {
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
    json!({
        "status": if findings.is_empty() { "ok" } else { "corrupt" },
        "findings": findings
    })
}

pub fn format_inspect_human(summary: &Value) -> String {
    format!(
        "run_id: {}\nstatus: {}\nretry_count: {}\ncache_hits: {}\nartifact_count: {}",
        summary.get("run_id").unwrap_or(&Value::Null),
        summary.get("status").unwrap_or(&Value::Null),
        summary.get("retry_count").unwrap_or(&Value::Null),
        summary.get("cache_hits").unwrap_or(&Value::Null),
        summary.get("artifact_count").unwrap_or(&Value::Null),
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
                .or_else(|| v.get("outputs").and_then(Value::as_array).map(|arr| arr.len()))
        })
        .unwrap_or(0)
}

fn read_node_traces(run_dir: &Path) -> Result<std::collections::BTreeMap<String, Value>, std::io::Error> {
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
