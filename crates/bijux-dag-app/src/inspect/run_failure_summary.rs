use bijux_dag_artifacts::{
    FailureAffectedGroups, FailureCauseRecord, FailureInfo, FailurePropagationRecord,
    RunFailureSummary,
};
use bijux_dag_core::Graph;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
struct FailureTrace {
    status: String,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    failure: Option<FailureInfo>,
    skip_reason: Option<String>,
    transition_cause: Option<String>,
}

pub fn explain_failure(run_dir: &Path) -> Result<Value, std::io::Error> {
    let traces = read_node_traces(run_dir)?;
    let graph = read_graph_snapshot(run_dir);
    let summary = read_failure_summary(run_dir)
        .filter(|summary| {
            summary.primary_failure.is_some()
                || traces.values().all(|trace| trace.status != "failed")
        })
        .unwrap_or_else(|| derive_failure_summary(&traces, graph.as_ref()));
    Ok(render_failure_report(summary, &traces))
}

fn read_failure_summary(run_dir: &Path) -> Option<RunFailureSummary> {
    let path = run_dir.join("observability.root-causes.json");
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn read_node_traces(run_dir: &Path) -> Result<BTreeMap<String, FailureTrace>, std::io::Error> {
    let nodes_dir = run_dir.join("nodes");
    let mut traces = BTreeMap::new();
    if !nodes_dir.exists() {
        return Ok(traces);
    }

    let mut entries = fs::read_dir(nodes_dir)?.filter_map(Result::ok).collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        if !trace_path.exists() {
            continue;
        }
        let raw = fs::read_to_string(trace_path)?;
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let Some(trace) = failure_trace_from_value(&value) else {
            continue;
        };
        traces.insert(node_id, trace);
    }
    Ok(traces)
}

fn read_graph_snapshot(run_dir: &Path) -> Option<Graph> {
    for path in [run_dir.join("graph.snapshot.json"), run_dir.join("snapshot.json")] {
        let Ok(raw) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<Value>(&raw) else {
            continue;
        };
        let graph_value = value.get("graph").cloned().unwrap_or(value);
        if let Ok(graph) = serde_json::from_value::<Graph>(graph_value) {
            return Some(graph);
        }
    }
    None
}

fn derive_failure_summary(
    traces: &BTreeMap<String, FailureTrace>,
    graph: Option<&Graph>,
) -> RunFailureSummary {
    let upstream = graph.map_or_else(BTreeMap::new, upstream_nodes);
    let downstream = graph.map_or_else(BTreeMap::new, downstream_nodes);
    let ordered_failures = sorted_failures(traces);
    let propagated_failure_ids = ordered_failures
        .iter()
        .filter(|(node_id, trace)| is_propagated_failure(node_id, trace, &upstream, traces))
        .map(|(node_id, _)| (*node_id).to_string())
        .collect::<BTreeSet<_>>();

    let causal_failures = ordered_failures
        .iter()
        .filter(|(node_id, _)| !propagated_failure_ids.contains(*node_id))
        .map(|(node_id, trace)| (*node_id, *trace))
        .collect::<Vec<_>>();
    let root_candidates =
        if causal_failures.is_empty() { ordered_failures.clone() } else { causal_failures.clone() };

    let roots = root_candidates
        .iter()
        .map(|(node_id, trace)| format!("{node_id}:{}", failure_reason(trace)))
        .collect::<Vec<_>>();
    let primary_failure =
        causal_failures.first().or_else(|| ordered_failures.first()).map(|(node_id, trace)| {
            FailureCauseRecord {
                node_id: (*node_id).to_string(),
                failure_class: failure_class(trace),
                failure_code: trace.failure.as_ref().map(|failure| failure.code.clone()),
                message: trace.failure.as_ref().map(|failure| failure.message.clone()),
                reason: Some(failure_reason(trace)),
                finished_unix_ms: Some(trace.finished_unix_ms),
            }
        });

    let propagated_failures = ordered_failures
        .iter()
        .filter(|(node_id, _)| propagated_failure_ids.contains(*node_id))
        .map(|(node_id, _)| build_propagation_record(node_id, "failed", &upstream, traces))
        .collect::<Vec<_>>();

    let mut propagated_skips = traces
        .iter()
        .filter(|(node_id, trace)| is_propagated_skip(node_id, trace, &upstream, traces))
        .map(|(node_id, trace)| {
            (
                node_id.as_str(),
                trace.finished_unix_ms,
                build_propagation_record(node_id, &trace.status, &upstream, traces),
            )
        })
        .collect::<Vec<_>>();
    propagated_skips.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(right.0)));
    let propagated_skips = propagated_skips.into_iter().map(|(_, _, record)| record).collect();

    let (downstream_affected_nodes, downstream_affected_groups) =
        downstream_affected(primary_failure.as_ref(), &downstream, traces);

    RunFailureSummary {
        roots,
        primary_failure,
        propagated_failures,
        propagated_skips,
        downstream_affected_nodes,
        downstream_affected_groups,
    }
}

fn render_failure_report(
    summary: RunFailureSummary,
    traces: &BTreeMap<String, FailureTrace>,
) -> Value {
    let failed_nodes = sorted_failures(traces)
        .into_iter()
        .map(|(node_id, _)| node_id.to_string())
        .collect::<Vec<_>>();
    let failure_classes = traces
        .iter()
        .filter(|(_, trace)| trace.status == "failed")
        .filter_map(|(node_id, trace)| {
            failure_class(trace)
                .map(|failure_class| (node_id.clone(), Value::String(failure_class)))
        })
        .collect::<serde_json::Map<_, _>>();
    let propagated_or_skipped_nodes = summary
        .propagated_failures
        .iter()
        .chain(summary.propagated_skips.iter())
        .map(|record| record.node_id.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let primary_failure = summary.primary_failure.clone();

    json!({
        "roots": summary.roots,
        "primary_failure": summary.primary_failure,
        "root_failure": primary_failure.as_ref().map(|record| record.node_id.clone()),
        "root_failure_class": primary_failure.as_ref().and_then(|record| record.failure_class.clone()),
        "root_failure_code": primary_failure.as_ref().and_then(|record| record.failure_code.clone()),
        "root_failure_message": primary_failure.as_ref().and_then(|record| record.message.clone()),
        "root_failure_reason": primary_failure.as_ref().and_then(|record| record.reason.clone()),
        "failed_nodes": failed_nodes,
        "failure_classes": failure_classes,
        "propagated_failures": summary.propagated_failures,
        "propagated_skips": summary.propagated_skips,
        "propagated_or_skipped_nodes": propagated_or_skipped_nodes,
        "downstream_affected_nodes": summary.downstream_affected_nodes,
        "downstream_affected_groups": summary.downstream_affected_groups,
    })
}

fn sorted_failures(traces: &BTreeMap<String, FailureTrace>) -> Vec<(&str, &FailureTrace)> {
    let mut failures = traces
        .iter()
        .filter(|(_, trace)| trace.status == "failed")
        .map(|(node_id, trace)| (node_id.as_str(), trace))
        .collect::<Vec<_>>();
    failures.sort_by(|left, right| {
        left.1
            .finished_unix_ms
            .cmp(&right.1.finished_unix_ms)
            .then_with(|| left.1.started_unix_ms.cmp(&right.1.started_unix_ms))
            .then_with(|| left.0.cmp(right.0))
    });
    failures
}

fn upstream_nodes(graph: &Graph) -> BTreeMap<String, BTreeSet<String>> {
    let mut upstream = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in &graph.edges {
        upstream.entry(edge.to.node_id.clone()).or_default().insert(edge.from.node_id.clone());
    }
    upstream
}

fn downstream_nodes(graph: &Graph) -> BTreeMap<String, BTreeSet<String>> {
    let mut downstream = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in &graph.edges {
        downstream.entry(edge.from.node_id.clone()).or_default().insert(edge.to.node_id.clone());
    }
    downstream
}

fn is_propagated_failure(
    node_id: &str,
    trace: &FailureTrace,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> bool {
    trace.failure.as_ref().is_some_and(|failure| failure.code == "UPSTREAM_FAILED")
        || trace.transition_cause.as_deref() == Some("DependencyFailed")
        || trace.failure.as_ref().is_some_and(|failure| failure.kind == "Dependency")
        || (trace.failure.is_none() && has_non_success_upstream(node_id, upstream, traces))
}

fn is_propagated_skip(
    node_id: &str,
    trace: &FailureTrace,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> bool {
    matches!(trace.status.as_str(), "skipped" | "cancelled")
        && (trace.skip_reason.as_deref() == Some("upstream_failed")
            || trace.transition_cause.as_deref() == Some("DependencyFailed")
            || has_non_success_upstream(node_id, upstream, traces))
}

fn has_non_success_upstream(
    node_id: &str,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> bool {
    blocking_nodes(node_id, upstream, traces).into_iter().next().is_some()
}

fn blocking_nodes(
    node_id: &str,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> Vec<String> {
    upstream
        .get(node_id)
        .into_iter()
        .flatten()
        .filter(|upstream_id| {
            traces
                .get(*upstream_id)
                .is_some_and(|trace| !matches!(trace.status.as_str(), "success" | "cached"))
        })
        .cloned()
        .collect::<Vec<_>>()
}

fn build_propagation_record(
    node_id: &str,
    status: &str,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> FailurePropagationRecord {
    let trace = traces.get(node_id);
    FailurePropagationRecord {
        node_id: node_id.to_string(),
        status: status.to_string(),
        reason: trace.map_or_else(|| "upstream_failed".to_string(), propagation_reason),
        propagation_mode: None,
        blocking_nodes: blocking_nodes(node_id, upstream, traces),
    }
}

fn downstream_affected(
    primary_failure: Option<&FailureCauseRecord>,
    downstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, FailureTrace>,
) -> (Vec<String>, FailureAffectedGroups) {
    let Some(primary_failure) = primary_failure else {
        return (Vec::new(), FailureAffectedGroups::default());
    };
    let descendants = descendants_of(&primary_failure.node_id, downstream);
    let mut affected = Vec::new();
    let mut groups = FailureAffectedGroups::default();

    for node_id in descendants {
        let Some(trace) = traces.get(&node_id) else {
            continue;
        };
        match trace.status.as_str() {
            "failed" => groups.failed.push(node_id.clone()),
            "skipped" => groups.skipped.push(node_id.clone()),
            "cancelled" => groups.cancelled.push(node_id.clone()),
            _ => continue,
        }
        affected.push(node_id);
    }

    (affected, groups)
}

fn descendants_of(node_id: &str, downstream: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    let mut visited = BTreeSet::<String>::new();
    let mut queue = VecDeque::<String>::new();
    if let Some(children) = downstream.get(node_id) {
        queue.extend(children.iter().cloned());
    }

    while let Some(candidate) = queue.pop_front() {
        if !visited.insert(candidate.clone()) {
            continue;
        }
        if let Some(children) = downstream.get(&candidate) {
            queue.extend(children.iter().cloned());
        }
    }

    visited.into_iter().collect()
}

fn propagation_reason(trace: &FailureTrace) -> String {
    if let Some(skip_reason) = &trace.skip_reason {
        return skip_reason.clone();
    }
    if let Some(failure) = &trace.failure {
        return failure_reason_from_info(failure);
    }
    if let Some(transition_cause) = &trace.transition_cause {
        return snake_case_transition_cause(transition_cause);
    }
    "upstream_failed".to_string()
}

fn failure_reason(trace: &FailureTrace) -> String {
    trace
        .failure
        .as_ref()
        .map(failure_reason_from_info)
        .or_else(|| trace.transition_cause.as_ref().map(|cause| snake_case_transition_cause(cause)))
        .unwrap_or_else(|| "execution_failed".to_string())
}

fn failure_reason_from_info(failure: &FailureInfo) -> String {
    match failure.code.as_str() {
        "POLICY_DENIED" | "POLICY_UNENFORCEABLE" => "policy_denied".to_string(),
        "UPSTREAM_FAILED" => "upstream_failed".to_string(),
        "RUN_ABORTED" => "execution_aborted".to_string(),
        "EXEC_CANCELLED" => "cancel_requested".to_string(),
        "RUN_TIMEOUT" | "EXEC_TIMEOUT" => "timeout_exceeded".to_string(),
        "CONTAINER_ENGINE_UNAVAILABLE" => "infrastructure_failed".to_string(),
        "OUTPUT_MISSING" => "missing_required_output".to_string(),
        "INPUT_MISSING" => "missing_required_input".to_string(),
        _ => match failure.kind.as_str() {
            "Policy" => "policy_denied".to_string(),
            "Infrastructure" => "infrastructure_failed".to_string(),
            _ => "execution_failed".to_string(),
        },
    }
}

fn failure_class(trace: &FailureTrace) -> Option<String> {
    trace.failure.as_ref().map(|failure| failure.operator_class().as_str().to_string())
}

fn snake_case_transition_cause(cause: &str) -> String {
    let mut result = String::with_capacity(cause.len());
    for (index, ch) in cause.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if index > 0 {
                result.push('_');
            }
            result.push(ch.to_ascii_lowercase());
        } else {
            result.push(ch);
        }
    }
    result
}

fn failure_trace_from_value(value: &Value) -> Option<FailureTrace> {
    let status = value.get("status").and_then(Value::as_str)?.to_string();
    let failure = value
        .get("failure")
        .cloned()
        .and_then(|failure| serde_json::from_value::<FailureInfo>(failure).ok());
    Some(FailureTrace {
        status,
        started_unix_ms: value.get("started_unix_ms").and_then(Value::as_u64).map_or(0, u128::from),
        finished_unix_ms: value
            .get("finished_unix_ms")
            .and_then(Value::as_u64)
            .map_or(0, u128::from),
        failure,
        skip_reason: value
            .get("skip_reason")
            .and_then(|skip_reason| skip_reason.get("reason"))
            .and_then(Value::as_str)
            .map(ToString::to_string),
        transition_cause: value
            .get("transition_cause")
            .and_then(Value::as_str)
            .map(ToString::to_string),
    })
}

#[cfg(test)]
mod tests {
    use super::explain_failure;
    use serde_json::json;
    use std::fs;

    #[test]
    fn explain_failure_derives_primary_failure_when_only_legacy_roots_are_present() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let run_dir = tmp.path().join("run-legacy");
        fs::create_dir_all(run_dir.join("nodes").join("build")).expect("build dir");
        fs::create_dir_all(run_dir.join("nodes").join("report")).expect("report dir");
        fs::create_dir_all(run_dir.join("nodes").join("publish")).expect("publish dir");
        fs::write(
            run_dir.join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&json!({
                "graph": {
                    "spec": "bijux-dag/v0.1",
                    "nodes": [
                        {"id":"build","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                        {"id":"report","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}},
                        {"id":"publish","kind":"shell","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{}}
                    ],
                    "edges": [
                        {"from":{"node_id":"build","port":"out"},"to":{"node_id":"report","port":"in"}},
                        {"from":{"node_id":"report","port":"out"},"to":{"node_id":"publish","port":"in"}}
                    ]
                },
                "graph_fingerprint": "graph-fingerprint"
            }))
            .expect("snapshot"),
        )
        .expect("write snapshot");
        fs::write(
            run_dir.join("observability.root-causes.json"),
            serde_json::to_vec_pretty(&json!({"roots":["build:execution_failed"]})).expect("roots"),
        )
        .expect("write roots");
        fs::write(
            run_dir.join("nodes").join("build").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"build",
                "status":"failed",
                "started_unix_ms":10,
                "finished_unix_ms":20,
                "attempt":1,
                "fingerprint":"fp-build",
                "adapter_id":"shell",
                "adapter_version":"1",
                "adapter_outputs_schema_version":"v1",
                "outputs":[],
                "failure":{"kind":"Execution","code":"EXEC_FAIL","message":"compiler exited with status 7"},
                "transition_cause":"ExecutionFailed",
                "lifecycle_transitions":[]
            }))
            .expect("build trace"),
        )
        .expect("write build trace");
        fs::write(
            run_dir.join("nodes").join("report").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"report",
                "status":"failed",
                "started_unix_ms":21,
                "finished_unix_ms":30,
                "attempt":1,
                "fingerprint":"fp-report",
                "adapter_id":"shell",
                "adapter_version":"1",
                "adapter_outputs_schema_version":"v1",
                "outputs":[],
                "failure":{"kind":"Dependency","code":"UPSTREAM_FAILED","message":"dependency trigger blocked execution for report"},
                "transition_cause":"DependencyFailed",
                "lifecycle_transitions":[]
            }))
            .expect("report trace"),
        )
        .expect("write report trace");
        fs::write(
            run_dir.join("nodes").join("publish").join("trace.json"),
            serde_json::to_vec_pretty(&json!({
                "node_id":"publish",
                "status":"skipped",
                "started_unix_ms":31,
                "finished_unix_ms":31,
                "attempt":1,
                "fingerprint":"fp-publish",
                "adapter_id":"shell",
                "adapter_version":"1",
                "adapter_outputs_schema_version":"v1",
                "outputs":[],
                "skip_reason":{"reason":"upstream_failed"},
                "transition_cause":"DependencyFailed",
                "lifecycle_transitions":[]
            }))
            .expect("publish trace"),
        )
        .expect("write publish trace");

        let report = explain_failure(&run_dir).expect("report");
        assert_eq!(report["root_failure"], "build");
        assert_eq!(report["root_failure_class"], "execution");
        assert_eq!(report["root_failure_message"], "compiler exited with status 7");
        assert_eq!(report["propagated_failures"][0]["node_id"], "report");
        assert_eq!(report["propagated_skips"][0]["node_id"], "publish");
        assert_eq!(report["downstream_affected_groups"]["failed"], json!(["report"]));
        assert_eq!(report["downstream_affected_groups"]["skipped"], json!(["publish"]));
    }
}
