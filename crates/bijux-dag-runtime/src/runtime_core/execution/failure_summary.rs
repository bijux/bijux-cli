use crate::io::Fs;
use crate::RuntimeError;
use bijux_dag_artifacts::{
    FailureAffectedGroups, FailureCauseRecord, FailurePropagationRecord, NodeTrace,
    RunFailureSummary,
};
use bijux_dag_core::Graph;
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::path::Path;

pub(crate) fn build_run_failure_summary(
    fs: &dyn Fs,
    run_dir: &Path,
    graph: &Graph,
) -> Result<RunFailureSummary, RuntimeError> {
    let traces = read_node_traces(fs, run_dir)?;
    if traces.is_empty() {
        return Ok(RunFailureSummary::default());
    }

    let upstream = upstream_nodes(graph);
    let downstream = downstream_nodes(graph);
    let ordered_failures = sorted_failures(&traces);
    let propagated_failure_ids = ordered_failures
        .iter()
        .filter(|(node_id, trace)| is_propagated_failure(node_id, trace, &upstream, &traces))
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
        .map(|(node_id, _)| build_propagation_record(node_id, "failed", &upstream, &traces))
        .collect::<Vec<_>>();

    let propagated_skips = traces
        .iter()
        .filter(|(node_id, trace)| is_propagated_skip(node_id, trace, &upstream, &traces))
        .map(|(node_id, trace)| {
            (
                node_id.as_str(),
                trace.finished_unix_ms,
                build_propagation_record(node_id, &trace.status, &upstream, &traces),
            )
        })
        .collect::<Vec<_>>();

    let mut propagated_skips = propagated_skips.into_iter().collect::<Vec<_>>();
    propagated_skips.sort_by(|left, right| left.1.cmp(&right.1).then_with(|| left.0.cmp(right.0)));
    let propagated_skips = propagated_skips.into_iter().map(|(_, _, record)| record).collect();

    let (downstream_affected_nodes, downstream_affected_groups) =
        downstream_affected(primary_failure.as_ref(), &downstream, &traces);

    Ok(RunFailureSummary {
        roots,
        primary_failure,
        propagated_failures,
        propagated_skips,
        downstream_affected_nodes,
        downstream_affected_groups,
    })
}

fn read_node_traces(
    fs: &dyn Fs,
    run_dir: &Path,
) -> Result<BTreeMap<String, NodeTrace>, RuntimeError> {
    let nodes_dir = run_dir.join("nodes");
    let entries = match fs.read_dir(&nodes_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
        Err(error) => {
            return Err(RuntimeError::Executor(format!(
                "failed to inspect node traces under {}: {}",
                nodes_dir.display(),
                error
            )));
        }
    };

    let mut traces = BTreeMap::new();
    for entry in entries {
        let node_id = entry.file_name().to_string_lossy().to_string();
        let trace_path = entry.path().join("trace.json");
        let raw = match fs.read_to_string(&trace_path) {
            Ok(raw) => raw,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => {
                return Err(RuntimeError::Executor(format!(
                    "failed to read node trace {}: {}",
                    trace_path.display(),
                    error
                )));
            }
        };
        let trace = serde_json::from_str::<NodeTrace>(&raw).map_err(|error| {
            RuntimeError::Executor(format!(
                "failed to parse node trace {}: {}",
                trace_path.display(),
                error
            ))
        })?;
        traces.insert(node_id, trace);
    }
    Ok(traces)
}

fn sorted_failures(traces: &BTreeMap<String, NodeTrace>) -> Vec<(&str, &NodeTrace)> {
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
    trace: &NodeTrace,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, NodeTrace>,
) -> bool {
    trace.failure.as_ref().is_some_and(|failure| failure.code == "UPSTREAM_FAILED")
        || trace.transition_cause.as_deref() == Some("DependencyFailed")
        || trace.failure.as_ref().is_some_and(|failure| failure.kind == "Dependency")
        || (trace.failure.is_none() && has_non_success_upstream(node_id, upstream, traces))
}

fn is_propagated_skip(
    node_id: &str,
    trace: &NodeTrace,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, NodeTrace>,
) -> bool {
    matches!(trace.status.as_str(), "skipped" | "cancelled")
        && (trace.skip_reason.as_ref().is_some_and(|reason| reason.reason == "upstream_failed")
            || trace.transition_cause.as_deref() == Some("DependencyFailed")
            || has_non_success_upstream(node_id, upstream, traces))
}

fn has_non_success_upstream(
    node_id: &str,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, NodeTrace>,
) -> bool {
    blocking_nodes(node_id, upstream, traces).into_iter().next().is_some()
}

fn blocking_nodes(
    node_id: &str,
    upstream: &BTreeMap<String, BTreeSet<String>>,
    traces: &BTreeMap<String, NodeTrace>,
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
    traces: &BTreeMap<String, NodeTrace>,
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
    traces: &BTreeMap<String, NodeTrace>,
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

fn propagation_reason(trace: &NodeTrace) -> String {
    if let Some(skip_reason) = &trace.skip_reason {
        return skip_reason.reason.clone();
    }
    if let Some(failure) = &trace.failure {
        return crate::failure_propagation_cause(Some(failure)).to_string();
    }
    if let Some(transition_cause) = &trace.transition_cause {
        return snake_case_transition_cause(transition_cause);
    }
    "upstream_failed".to_string()
}

fn failure_reason(trace: &NodeTrace) -> String {
    trace
        .failure
        .as_ref()
        .map(|failure| crate::failure_propagation_cause(Some(failure)).to_string())
        .or_else(|| trace.transition_cause.as_ref().map(|cause| snake_case_transition_cause(cause)))
        .unwrap_or_else(|| "execution_failed".to_string())
}

fn failure_class(trace: &NodeTrace) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::build_run_failure_summary;
    use crate::io::StdFs;
    use bijux_dag_artifacts::{FailureClass, FailureInfo, NodeTrace, RunDir};
    use bijux_dag_core::{
        CacheBehavior, Edge, Graph, Node, NodeKind, OutputSpec, ParamValue, PortRef,
        SemanticNodeKind, TriggerRule,
    };
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn summary_separates_primary_failure_from_propagated_failures_and_skips() {
        let tmp = tempfile::tempdir().expect("tmpdir");
        let run_dir = RunDir::create_with_id(tmp.path(), "failure-summary").expect("run dir");
        fs::create_dir_all(run_dir.staging_path().join("nodes").join("build")).expect("build dir");
        fs::create_dir_all(run_dir.staging_path().join("nodes").join("report"))
            .expect("report dir");
        fs::create_dir_all(run_dir.staging_path().join("nodes").join("publish"))
            .expect("publish dir");

        write_trace(
            &run_dir.staging_path().join("nodes").join("build").join("trace.json"),
            NodeTrace {
                node_id: "build".to_string(),
                status: "failed".to_string(),
                started_unix_ms: 10,
                finished_unix_ms: 20,
                attempt: 1,
                fingerprint: "fp-build".to_string(),
                planner_contract_version: None,
                execution_fingerprint: None,
                evidence_fingerprint: None,
                adapter_id: "shell".to_string(),
                adapter_version: "1".to_string(),
                adapter_outputs_schema_version: "v1".to_string(),
                adapter_binary_sha256: None,
                resources: None,
                inputs_index: None,
                resolved_params: None,
                outputs: Vec::new(),
                container: None,
                cache_proof: None,
                cache_identity: None,
                branch_decision: None,
                trigger_evaluation: None,
                skip_reason: None,
                failure: Some(FailureInfo::new(
                    FailureClass::Execution,
                    "Execution",
                    "EXEC_FAIL",
                    "compiler exited with status 7",
                    None,
                )),
                transition_cause: Some("ExecutionFailed".to_string()),
                lifecycle_state: None,
                lifecycle_transitions: Vec::new(),
                replay_provenance: None,
            },
        );
        write_trace(
            &run_dir.staging_path().join("nodes").join("report").join("trace.json"),
            NodeTrace {
                node_id: "report".to_string(),
                status: "failed".to_string(),
                started_unix_ms: 21,
                finished_unix_ms: 30,
                attempt: 1,
                fingerprint: "fp-report".to_string(),
                planner_contract_version: None,
                execution_fingerprint: None,
                evidence_fingerprint: None,
                adapter_id: "shell".to_string(),
                adapter_version: "1".to_string(),
                adapter_outputs_schema_version: "v1".to_string(),
                adapter_binary_sha256: None,
                resources: None,
                inputs_index: None,
                resolved_params: None,
                outputs: Vec::new(),
                container: None,
                cache_proof: None,
                cache_identity: None,
                branch_decision: None,
                trigger_evaluation: None,
                skip_reason: None,
                failure: Some(FailureInfo::new(
                    FailureClass::Execution,
                    "Dependency",
                    "UPSTREAM_FAILED",
                    "dependency trigger blocked execution for report",
                    None,
                )),
                transition_cause: Some("DependencyFailed".to_string()),
                lifecycle_state: None,
                lifecycle_transitions: Vec::new(),
                replay_provenance: None,
            },
        );
        write_trace(
            &run_dir.staging_path().join("nodes").join("publish").join("trace.json"),
            NodeTrace {
                node_id: "publish".to_string(),
                status: "skipped".to_string(),
                started_unix_ms: 31,
                finished_unix_ms: 31,
                attempt: 1,
                fingerprint: "fp-publish".to_string(),
                planner_contract_version: None,
                execution_fingerprint: None,
                evidence_fingerprint: None,
                adapter_id: "shell".to_string(),
                adapter_version: "1".to_string(),
                adapter_outputs_schema_version: "v1".to_string(),
                adapter_binary_sha256: None,
                resources: None,
                inputs_index: None,
                resolved_params: None,
                outputs: Vec::new(),
                container: None,
                cache_proof: None,
                cache_identity: None,
                branch_decision: None,
                trigger_evaluation: None,
                skip_reason: Some(bijux_dag_artifacts::SkipReason {
                    reason: "upstream_failed".to_string(),
                }),
                failure: None,
                transition_cause: Some("DependencyFailed".to_string()),
                lifecycle_state: None,
                lifecycle_transitions: Vec::new(),
                replay_provenance: None,
            },
        );

        let summary = build_run_failure_summary(&StdFs, run_dir.staging_path(), &sample_graph())
            .expect("summary");

        assert_eq!(summary.roots, vec!["build:execution_failed"]);
        let primary = summary.primary_failure.expect("primary failure");
        assert_eq!(primary.node_id, "build");
        assert_eq!(primary.failure_class.as_deref(), Some("execution"));
        assert_eq!(primary.failure_code.as_deref(), Some("EXEC_FAIL"));
        assert_eq!(primary.message.as_deref(), Some("compiler exited with status 7"));
        assert_eq!(primary.reason.as_deref(), Some("execution_failed"));
        assert_eq!(summary.propagated_failures.len(), 1);
        assert_eq!(summary.propagated_failures[0].node_id, "report");
        assert_eq!(summary.propagated_failures[0].blocking_nodes, vec!["build"]);
        assert_eq!(summary.propagated_skips.len(), 1);
        assert_eq!(summary.propagated_skips[0].node_id, "publish");
        assert_eq!(summary.propagated_skips[0].blocking_nodes, vec!["report"]);
        assert_eq!(summary.downstream_affected_nodes, vec!["publish", "report"]);
        assert_eq!(summary.downstream_affected_groups.failed, vec!["report"]);
        assert_eq!(summary.downstream_affected_groups.skipped, vec!["publish"]);
    }

    fn write_trace(path: &std::path::Path, trace: NodeTrace) {
        fs::write(path, serde_json::to_vec_pretty(&trace).expect("encode trace")).expect("trace");
    }

    fn sample_graph() -> Graph {
        Graph {
            spec: "bijux-dag/v0.1".to_string(),
            meta: None,
            inputs: BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![sample_node("build"), sample_node("report"), sample_node("publish")],
            edges: vec![
                Edge {
                    id: None,
                    kind: Default::default(),
                    decision: None,
                    from: PortRef { node_id: "build".to_string(), port: "out".to_string() },
                    to: PortRef { node_id: "report".to_string(), port: "in".to_string() },
                },
                Edge {
                    id: None,
                    kind: Default::default(),
                    decision: None,
                    from: PortRef { node_id: "report".to_string(), port: "out".to_string() },
                    to: PortRef { node_id: "publish".to_string(), port: "in".to_string() },
                },
            ],
        }
    }

    fn sample_node(id: &str) -> Node {
        Node {
            id: id.to_string(),
            kind: NodeKind::Shell,
            semantic_kind: SemanticNodeKind::Task,
            inputs: vec!["in".to_string()],
            outputs: vec![OutputSpec::new("out", "out")],
            params: ParamValue::default(),
            container: None,
            timeout_ms: None,
            resources: None,
            tags: Vec::new(),
            retry: Default::default(),
            cache: CacheBehavior::default(),
            effects: Vec::new(),
            env_allowlist: Vec::new(),
            group: None,
            trigger_rule: TriggerRule::default(),
            branch: None,
            dynamic: None,
        }
    }
}
