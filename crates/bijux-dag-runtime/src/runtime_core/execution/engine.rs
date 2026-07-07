use crate::failure_summary::build_run_failure_summary;
use crate::{
    bind_path_variables_in_value, build_run_outputs_index, cache_dir_from_env, cache_mode_string,
    canonicalize_event_records, category_from_runtime_event_name, collect_outputs_summary,
    current_process_memory_bytes, node_fingerprint_from_ctx, node_fingerprint_with_inputs,
    reconstruct_timeline_from_events, reduce_execution_config, registered_adapters,
    sacred_execution, serialize_timeline_export, set_node_fingerprint, CacheProof, EffectSet,
    EventRecord, ExecutionCheckpoint, InMemoryMetricsRegistry, MetricsRegistry, NodeMetrics,
    NodePathBindings, NodeResult, NodeStatus, ReduceEmptyPolicy, ReduceExecutionMode,
    ReplayNodeAction, RunAttempt, RunContext, RunId, RunSnapshot, Runtime, RuntimeConfig,
    RuntimeError, SchedulerEventHook,
};
#[path = "engine_dispatch.rs"]
mod engine_dispatch;
#[path = "engine_finalize.rs"]
mod engine_finalize;
#[path = "engine_metrics.rs"]
mod engine_metrics;
#[path = "engine_observe.rs"]
mod engine_observe;
#[path = "engine_record.rs"]
mod engine_record;
use bijux_dag_artifacts::{
    finalize_run_manifest_with_mode, write_incomplete_run_marker, write_provenance,
    write_run_outputs_index, write_run_schema_index, FailureClass, FailureInfo,
    FailurePropagationRecord, InputsIndex, Manifest, NodeCounts, Provenance, ReplayProvenance,
    RunDir, RunDirLayout, RunDirSchemaIndex, RunFinalizationMode, RunMetadata, RunStopRequest,
    TriggerEvaluation, TriggerParentStatus,
};
use bijux_dag_core::{
    evaluate_trigger_rule, Effect, Graph, Node, NodeKind, SemanticNodeKind, TriggerRule,
    UpstreamTerminalOutcome, SPEC_VERSION,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
struct NodeLifecycleTimestamps {
    eligible_unix_ms: Option<u128>,
    queued_unix_ms: Option<u128>,
    running_unix_ms: Option<u128>,
}

#[derive(Debug, Clone)]
struct BranchResolution {
    decision: String,
    used_default: bool,
}

struct StopRequestWatcher {
    request: Arc<Mutex<Option<RunStopRequest>>>,
    shutdown: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl StopRequestWatcher {
    fn spawn(stop_request_path: PathBuf, cancel: Arc<AtomicBool>) -> Self {
        let request = Arc::new(Mutex::new(None));
        let shutdown = Arc::new(AtomicBool::new(false));
        let request_slot = Arc::clone(&request);
        let shutdown_flag = Arc::clone(&shutdown);
        let handle = std::thread::spawn(move || {
            while !shutdown_flag.load(Ordering::SeqCst) && !cancel.load(Ordering::SeqCst) {
                if let Some(stop_request) = read_stop_request_file(&stop_request_path) {
                    cancel.store(true, Ordering::SeqCst);
                    if let Ok(mut slot) = request_slot.lock() {
                        if slot.is_none() {
                            *slot = Some(stop_request);
                        }
                    }
                    break;
                }
                std::thread::sleep(Duration::from_millis(25));
            }
        });
        Self { request, shutdown, handle: Some(handle) }
    }

    fn observed_request(&self) -> Option<RunStopRequest> {
        self.request.lock().ok().and_then(|slot| slot.clone())
    }
}

impl Drop for StopRequestWatcher {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

fn read_stop_request_file(path: &Path) -> Option<RunStopRequest> {
    let raw = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&raw).ok()
}

fn resolve_branch_decision(
    ctx: &RunContext,
    node: &Node,
) -> Result<Option<BranchResolution>, FailureInfo> {
    if node.semantic_kind != SemanticNodeKind::Branch {
        return Ok(None);
    }
    let Some(branch) = &node.branch else {
        return Ok(None);
    };
    let Some(output) = node.outputs.iter().find(|output| output.name == branch.decision_output)
    else {
        return Err(FailureInfo::new(
            FailureClass::User,
            "User",
            "BRANCH_OUTPUT_MISSING",
            format!(
                "branch node {} is missing declared decision output {}",
                node.id, branch.decision_output
            ),
            Some(serde_json::json!({
                "node_id": node.id,
                "decision_output": branch.decision_output,
            })),
        ));
    };
    let output_path = ctx.run_dir.node_outputs_dir(&node.id).join(&output.path);
    let raw = ctx.fs.read_to_string(&output_path).map_err(|_| {
        FailureInfo::new(
            FailureClass::User,
            "User",
            "BRANCH_DECISION_UNREADABLE",
            format!("branch decision output for {} could not be read", node.id),
            Some(serde_json::json!({
                "node_id": node.id,
                "path": output.path,
            })),
        )
    })?;
    let decision = serde_json::from_str::<Value>(&raw)
        .ok()
        .and_then(|value| match value {
            Value::String(text) => Some(text),
            Value::Number(number) => Some(number.to_string()),
            Value::Bool(flag) => Some(flag.to_string()),
            _ => None,
        })
        .unwrap_or_else(|| raw.trim().to_string());
    if branch.decisions.iter().any(|candidate| candidate == &decision) {
        return Ok(Some(BranchResolution { decision: decision.clone(), used_default: false }));
    }
    if let Some(default_decision) = &branch.default_decision {
        return Ok(Some(BranchResolution {
            decision: default_decision.clone(),
            used_default: true,
        }));
    }
    Err(FailureInfo::new(
        FailureClass::User,
        "User",
        "INVALID_BRANCH_DECISION",
        format!("branch node {} produced undeclared decision {}", node.id, decision),
        Some(serde_json::json!({
            "node_id": node.id,
            "produced_decision": decision,
            "declared_decisions": branch.decisions,
        })),
    ))
}

fn branch_nodes_to_skip(
    graph: &Graph,
    branch_node_id: &str,
    selected_decision: &str,
) -> Vec<String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
    let mut selected_reachable = BTreeSet::new();
    let mut by_decision = BTreeMap::<String, BTreeSet<String>>::new();
    for edge in graph.edges.iter().filter(|edge| {
        edge.from.node_id == branch_node_id && edge.kind == bijux_dag_core::EdgeKind::Conditional
    }) {
        let Some(decision) = edge.decision.as_ref() else {
            continue;
        };
        let nodes = by_decision.entry(decision.clone()).or_default();
        let mut reachable = BTreeSet::from([edge.to.node_id.clone()]);
        let mut queue = vec![edge.to.node_id.clone()];
        while let Some(node_id) = queue.pop() {
            if let Some(children) = adjacency.get(&node_id) {
                for child in children {
                    if reachable.insert(child.clone()) {
                        queue.push(child.clone());
                    }
                }
            }
        }
        nodes.extend(reachable);
    }
    if let Some(selected) = by_decision.get(selected_decision) {
        selected_reachable.extend(selected.iter().cloned());
    }
    let mut pruned = BTreeSet::new();
    for (decision, nodes) in by_decision {
        if decision == selected_decision {
            continue;
        }
        for node_id in nodes {
            if !selected_reachable.contains(&node_id) {
                pruned.insert(node_id);
            }
        }
    }
    pruned.into_iter().collect()
}

fn replayed_branch_decisions(
    fs: &dyn crate::Fs,
    out_dir: &Path,
    parent_run_id: &str,
    plan: &crate::ExecutionPlan,
    graph: &Graph,
) -> Result<Vec<(String, String)>, RuntimeError> {
    let parent_layout = RunDirLayout::preview(out_dir, Some(parent_run_id))
        .map_err(|error| RuntimeError::Executor(format!("invalid parent run id: {error}")))?;
    let mut decisions = Vec::new();
    for node in &graph.nodes {
        if node.semantic_kind != SemanticNodeKind::Branch
            || !plan.filter_reasons.contains_key(&node.id)
        {
            continue;
        }
        let trace_path = parent_layout.final_path.join("nodes").join(&node.id).join("trace.json");
        if fs.metadata(&trace_path).is_err() {
            continue;
        }
        let raw = fs.read_to_string(&trace_path).map_err(|error| {
            RuntimeError::Executor(format!(
                "failed to read parent branch trace for {}: {}",
                node.id, error
            ))
        })?;
        let trace: bijux_dag_artifacts::NodeTrace =
            serde_json::from_str(&raw).map_err(|error| {
                RuntimeError::Executor(format!(
                    "failed to parse parent branch trace for {}: {}",
                    node.id, error
                ))
            })?;
        let Some(decision) = trace.branch_decision else {
            continue;
        };
        decisions.push((node.id.clone(), decision));
    }
    decisions.sort();
    Ok(decisions)
}

fn seed_replayed_branch_pruning(
    fs: &dyn crate::Fs,
    out_dir: &Path,
    parent_run_id: &str,
    plan: &crate::ExecutionPlan,
    graph: &Graph,
    branch_pruned_nodes: &mut BTreeSet<String>,
) -> Result<Vec<(String, String)>, RuntimeError> {
    let decisions = replayed_branch_decisions(fs, out_dir, parent_run_id, plan, graph)?;
    for (branch_node_id, decision) in &decisions {
        for pruned in branch_nodes_to_skip(graph, branch_node_id, decision) {
            branch_pruned_nodes.insert(pruned);
        }
    }
    Ok(decisions)
}

#[derive(Debug, Clone)]
struct ResumeBootstrap {
    attempt_index: u32,
    reusable_nodes: HashMap<String, NodeStatus>,
    branch_decisions: Vec<(String, String)>,
    reset_nodes: Vec<String>,
    summary: crate::ResumeSummary,
}

fn parse_trace_status(status: &str) -> Option<NodeStatus> {
    match status {
        "success" => Some(NodeStatus::Success),
        "failed" => Some(NodeStatus::Failed),
        "skipped" => Some(NodeStatus::Skipped),
        "cached" => Some(NodeStatus::Cached),
        "cancelled" => Some(NodeStatus::Cancelled),
        _ => None,
    }
}

fn read_resume_attempts(ctx: &RunContext) -> Result<Vec<RunAttempt>, RuntimeError> {
    let attempts_path = ctx.run_dir.staging_path().join("run.attempts.json");
    let raw = ctx.fs.read_to_string(&attempts_path).map_err(|error| {
        RuntimeError::Executor(format!(
            "resume metadata missing at {}: {}",
            attempts_path.display(),
            error
        ))
    })?;
    let attempts: Vec<RunAttempt> = serde_json::from_str(&raw).map_err(|error| {
        RuntimeError::Executor(format!(
            "resume metadata is invalid at {}: {}",
            attempts_path.display(),
            error
        ))
    })?;
    if attempts.is_empty() {
        return Err(RuntimeError::Executor(format!(
            "resume metadata is empty at {}",
            attempts_path.display()
        )));
    }
    Ok(attempts)
}

fn read_node_trace(
    ctx: &RunContext,
    node_id: &str,
) -> Result<Option<bijux_dag_artifacts::NodeTrace>, RuntimeError> {
    let trace_path = ctx.run_dir.node_trace_path(node_id);
    match ctx.fs.metadata(&trace_path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(RuntimeError::Executor(format!(
                "failed to inspect resume trace for {}: {}",
                node_id, error
            )));
        }
    }
    let raw = ctx.fs.read_to_string(&trace_path).map_err(|error| {
        RuntimeError::Executor(format!("failed to read resume trace for {}: {}", node_id, error))
    })?;
    let trace = serde_json::from_str(&raw).map_err(|error| {
        RuntimeError::Executor(format!("failed to parse resume trace for {}: {}", node_id, error))
    })?;
    Ok(Some(trace))
}

fn trace_outputs_match(
    ctx: &RunContext,
    graph: &Graph,
    node_id: &str,
    trace: &bijux_dag_artifacts::NodeTrace,
) -> bool {
    let Some(node) = graph.nodes.iter().find(|node| node.id == node_id) else {
        return false;
    };
    let inspection = crate::inspect_declared_outputs(
        ctx.run_dir.node_outputs_dir(node_id).as_path(),
        &node.outputs,
    );
    inspection.failure.is_none() && inspection.output_evidence == trace.outputs
}

fn resume_fingerprint_matches(
    ctx: &RunContext,
    node_id: &str,
    trace: &bijux_dag_artifacts::NodeTrace,
) -> Result<bool, RuntimeError> {
    let inputs_index_path = ctx.run_dir.node_inputs_index_path(node_id);
    let inputs_index = match ctx.fs.metadata(&inputs_index_path) {
        Ok(_) => {
            let raw = ctx.fs.read_to_string(&inputs_index_path).map_err(|error| {
                RuntimeError::Executor(format!(
                    "failed to read persisted inputs index for {}: {}",
                    node_id, error
                ))
            })?;
            serde_json::from_str::<InputsIndex>(&raw).map_err(|error| {
                RuntimeError::Executor(format!(
                    "failed to parse persisted inputs index for {}: {}",
                    node_id, error
                ))
            })?
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            InputsIndex { collections: vec![], files: vec![] }
        }
        Err(error) => {
            return Err(RuntimeError::Executor(format!(
                "failed to inspect persisted inputs index for {}: {}",
                node_id, error
            )));
        }
    };
    let current_fingerprint =
        node_fingerprint_with_inputs(&node_fingerprint_from_ctx(ctx, node_id), &inputs_index)?;
    Ok(trace.fingerprint == current_fingerprint)
}

fn clear_resumed_node_dir(ctx: &RunContext, node_id: &str) -> Result<(), RuntimeError> {
    let node_dir = ctx.run_dir.node_dir(node_id);
    match ctx.fs.metadata(&node_dir) {
        Ok(_) => ctx.fs.remove_dir_all(&node_dir)?,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(RuntimeError::Executor(format!(
                "failed to inspect node directory for {} during resume cleanup: {}",
                node_id, error
            )));
        }
    }
    Ok(())
}

fn prepare_resume_bootstrap(
    ctx: &RunContext,
    graph: &Graph,
    plan: &crate::ExecutionPlan,
    options: &RuntimeConfig,
) -> Result<ResumeBootstrap, RuntimeError> {
    let previous_attempts = read_resume_attempts(ctx)?;
    let attempt_index =
        previous_attempts.iter().map(|attempt| attempt.attempt_index).max().unwrap_or(0) + 1;
    let mut reused_nodes = BTreeSet::new();
    let mut reusable_nodes = HashMap::new();
    let mut branch_decisions = Vec::new();

    for node_id in &plan.order {
        let Some(trace) = read_node_trace(ctx, node_id)? else {
            continue;
        };
        if plan
            .dep_map
            .get(node_id)
            .is_some_and(|deps| deps.iter().any(|dependency| !reused_nodes.contains(dependency)))
        {
            continue;
        }
        let Some(status) = parse_trace_status(&trace.status) else {
            continue;
        };
        if !matches!(status, NodeStatus::Success | NodeStatus::Cached) {
            continue;
        }
        if !resume_fingerprint_matches(ctx, node_id, &trace)? {
            continue;
        }
        if !trace_outputs_match(ctx, graph, node_id, &trace) {
            continue;
        }
        reused_nodes.insert(node_id.clone());
        reusable_nodes.insert(node_id.clone(), status);
        if let Some(decision) = trace.branch_decision {
            branch_decisions.push((node_id.clone(), decision));
        }
    }

    branch_decisions.sort();
    let mut pruned_nodes = BTreeSet::new();
    for (branch_node_id, decision) in &branch_decisions {
        for pruned in branch_nodes_to_skip(graph, branch_node_id, decision) {
            pruned_nodes.insert(pruned);
        }
    }

    let mut reset_nodes = plan
        .order
        .iter()
        .filter(|node_id| !reused_nodes.contains(*node_id))
        .cloned()
        .collect::<Vec<_>>();
    reset_nodes.sort();

    let mut unresolved_nodes = plan
        .order
        .iter()
        .filter(|node_id| {
            !reused_nodes.contains(*node_id)
                && !plan.filter_reasons.contains_key(*node_id)
                && !pruned_nodes.contains(*node_id)
        })
        .cloned()
        .collect::<Vec<_>>();
    unresolved_nodes.sort();

    let (rerun_nodes, rejected_nodes) = match options.resume_failure_mode {
        crate::ResumeFailureMode::RerunIncomplete => (unresolved_nodes, Vec::new()),
        crate::ResumeFailureMode::RejectIncomplete => {
            if unresolved_nodes.is_empty() {
                (Vec::new(), Vec::new())
            } else {
                (Vec::new(), unresolved_nodes)
            }
        }
    };

    Ok(ResumeBootstrap {
        attempt_index,
        reusable_nodes,
        branch_decisions,
        reset_nodes,
        summary: crate::ResumeSummary {
            failure_mode: options.resume_failure_mode,
            reused_nodes: reused_nodes.into_iter().collect(),
            rerun_nodes,
            rejected_nodes,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::{replayed_branch_decisions, seed_replayed_branch_pruning};
    use crate::{build_plan, Selector, SelectorSet, StdFs};
    use bijux_dag_artifacts::RunDirLayout;
    use bijux_dag_core::parse_graph_strict;
    use std::collections::BTreeSet;
    use std::fs;

    fn filtered_branch_graph() -> bijux_dag_core::Graph {
        parse_graph_strict(
            r#"{
              "spec":"bijux-dag/v0.1",
              "nodes":[
                {"id":"seed","kind":"const","outputs":[{"name":"out","path":"seed/out"}],"params":{"value":1}},
                {
                  "id":"decide",
                  "kind":"const",
                  "semantic_kind":"branch",
                  "inputs":["in"],
                  "outputs":[{"name":"decision","path":"decide/decision.txt"}],
                  "params":{"value":"left"},
                  "branch":{"decisions":["left","right"],"decision_output":"decision"}
                },
                {"id":"left","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"left/out"}],"params":{"value":"left"},"trigger_rule":"any_success"},
                {"id":"right","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"right/out"}],"params":{"value":"right"},"trigger_rule":"any_success"}
              ],
              "edges":[
                {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"decide","port":"in"}},
                {"kind":"conditional","decision":"left","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"left","port":"in"}},
                {"kind":"conditional","decision":"right","from":{"node_id":"decide","port":"decision"},"to":{"node_id":"right","port":"in"}}
              ]
            }"#,
        )
        .expect("graph")
    }

    fn write_parent_branch_trace(
        out_dir: &std::path::Path,
        run_id: &str,
        node_id: &str,
        decision: &str,
    ) {
        let layout = RunDirLayout::preview(out_dir, Some(run_id)).expect("parent layout");
        let node_dir = layout.final_path.join("nodes").join(node_id);
        fs::create_dir_all(&node_dir).expect("create parent node dir");
        fs::write(
            node_dir.join("trace.json"),
            format!(
                r#"{{"node_id":"{node_id}","status":"success","started_unix_ms":0,"finished_unix_ms":0,"attempt":1,"fingerprint":"fp","adapter_id":"const","adapter_version":"1","adapter_outputs_schema_version":"1","branch_decision":"{decision}"}}"#
            ),
        )
        .expect("write parent trace");
    }

    #[test]
    fn replayed_branch_decisions_read_parent_trace_for_filtered_branch_nodes() {
        let graph = filtered_branch_graph();
        let plan = build_plan(
            &graph,
            &crate::RuntimeConfig {
                selectors: SelectorSet {
                    include: vec![Selector::IdPrefix("left".to_string())],
                    exclude: vec![Selector::IdPrefix("decide".to_string())],
                },
                partial_rerun_dependency_closure: false,
                ..crate::RuntimeConfig::default()
            },
        );
        let out_dir = tempfile::tempdir().expect("tempdir");
        write_parent_branch_trace(out_dir.path(), "run-parent", "decide", "left");

        let decisions =
            replayed_branch_decisions(&StdFs, out_dir.path(), "run-parent", &plan, &graph)
                .expect("replayed branch decisions");

        assert_eq!(decisions, vec![("decide".to_string(), "left".to_string())]);
    }

    #[test]
    fn replayed_branch_decisions_seed_pruned_nodes_from_parent_trace() {
        let graph = filtered_branch_graph();
        let plan = build_plan(
            &graph,
            &crate::RuntimeConfig {
                selectors: SelectorSet {
                    include: vec![Selector::IdPrefix("left".to_string())],
                    exclude: vec![Selector::IdPrefix("decide".to_string())],
                },
                partial_rerun_dependency_closure: false,
                ..crate::RuntimeConfig::default()
            },
        );
        let out_dir = tempfile::tempdir().expect("tempdir");
        write_parent_branch_trace(out_dir.path(), "run-parent", "decide", "left");
        let mut pruned = BTreeSet::new();

        let decisions = seed_replayed_branch_pruning(
            &StdFs,
            out_dir.path(),
            "run-parent",
            &plan,
            &graph,
            &mut pruned,
        )
        .expect("seed branch pruning");

        assert_eq!(decisions, vec![("decide".to_string(), "left".to_string())]);
        assert!(pruned.contains("right"));
        assert!(!pruned.contains("left"));
    }
}

fn partial_rerun_selected(options: &RuntimeConfig) -> bool {
    options.parent_run_id.is_some()
        && (!options.selectors.include.is_empty()
            || !options.upstream_selection_targets.is_empty()
            || !options.downstream_selection_roots.is_empty())
}

fn selector_matches(node: &Node, selector: &crate::Selector) -> bool {
    match selector {
        crate::Selector::Id(id) => node.id == *id,
        crate::Selector::IdPrefix(prefix) => node.id.starts_with(prefix),
        crate::Selector::Tag(tag) => node.tags.iter().any(|candidate| candidate == tag),
        crate::Selector::Kind(kind) => node.kind.as_str() == kind,
    }
}

fn selected_rerun_targets(graph: &Graph, options: &RuntimeConfig) -> Vec<String> {
    let mut selected = graph
        .nodes
        .iter()
        .filter(|node| {
            options.selectors.include.iter().any(|selector| selector_matches(node, selector))
                && !options
                    .selectors
                    .exclude
                    .iter()
                    .any(|selector| selector_matches(node, selector))
        })
        .map(|node| node.id.clone())
        .collect::<Vec<_>>();
    if !options.upstream_selection_targets.is_empty() {
        selected.extend(crate::compute_upstream_run_closure(
            graph,
            &options.upstream_selection_targets,
        ));
    }
    if !options.downstream_selection_roots.is_empty() {
        selected.extend(crate::compute_downstream_run_closure(
            graph,
            &options.downstream_selection_roots,
        ));
    }
    selected.sort();
    selected.dedup();
    selected
}

fn trigger_rule_value(graph: &Graph, node_id: &str) -> serde_json::Value {
    graph
        .nodes
        .iter()
        .find(|node| node.id == node_id)
        .and_then(|node| serde_json::to_value(&node.trigger_rule).ok())
        .unwrap_or_else(|| serde_json::Value::String("unknown".to_string()))
}

fn upstream_nodes(dep_map: &HashMap<String, BTreeSet<String>>, node_id: &str) -> Vec<String> {
    let mut upstreams = dep_map
        .get(node_id)
        .map(|deps| deps.iter().cloned().collect::<Vec<_>>())
        .unwrap_or_default();
    upstreams.sort();
    upstreams
}

fn downstream_nodes(graph: &Graph, node_id: &str) -> Vec<String> {
    let mut adjacency = HashMap::<&str, Vec<&str>>::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from.node_id.as_str()).or_default().push(edge.to.node_id.as_str());
    }

    let mut visited = BTreeSet::<String>::new();
    let mut pending = adjacency.get(node_id).cloned().unwrap_or_default();
    while let Some(candidate) = pending.pop() {
        if !visited.insert(candidate.to_string()) {
            continue;
        }
        if let Some(children) = adjacency.get(candidate) {
            pending.extend(children.iter().copied());
        }
    }

    visited.into_iter().collect()
}

fn record_isolated_branch_descendants(
    graph: &Graph,
    failed_node_id: &str,
    isolated_branch_failures: &mut BTreeMap<String, BTreeSet<String>>,
) {
    for descendant in downstream_nodes(graph, failed_node_id) {
        isolated_branch_failures.entry(descendant).or_default().insert(failed_node_id.to_string());
    }
}

fn failure_propagation_record(
    node_id: &str,
    status: &str,
    reason: &str,
    propagation_mode: Option<&str>,
    blocking_nodes: Vec<String>,
) -> FailurePropagationRecord {
    FailurePropagationRecord {
        node_id: node_id.to_string(),
        status: status.to_string(),
        reason: reason.to_string(),
        propagation_mode: propagation_mode.map(str::to_string),
        blocking_nodes,
    }
}

fn upstream_terminal_outcome(status: &NodeStatus) -> UpstreamTerminalOutcome {
    match status {
        NodeStatus::Success => UpstreamTerminalOutcome::Success,
        NodeStatus::Cached => UpstreamTerminalOutcome::Cached,
        NodeStatus::Failed => UpstreamTerminalOutcome::Failed,
        NodeStatus::Skipped => UpstreamTerminalOutcome::Skipped,
        NodeStatus::Cancelled => UpstreamTerminalOutcome::Failed,
    }
}

fn trigger_rule_name(rule: &TriggerRule) -> String {
    serde_json::to_string(rule)
        .map(|value| value.trim_matches('"').to_string())
        .unwrap_or_else(|_| format!("{rule:?}").to_lowercase())
}

fn trigger_evaluation_for_dependencies(
    node: &Node,
    dependencies: &[String],
    parent_statuses: &[NodeStatus],
) -> TriggerEvaluation {
    let parent_outcomes = parent_statuses.iter().map(upstream_terminal_outcome).collect::<Vec<_>>();
    let evaluation = evaluate_trigger_rule(&node.trigger_rule, &parent_outcomes);
    let parent_statuses = dependencies
        .iter()
        .zip(parent_statuses.iter())
        .map(|(node_id, status)| TriggerParentStatus {
            node_id: node_id.clone(),
            status: crate::status_string(status),
        })
        .collect::<Vec<_>>();

    TriggerEvaluation {
        trigger_rule: trigger_rule_name(&evaluation.trigger_rule),
        satisfied: evaluation.satisfied,
        reason: evaluation.reason,
        parent_statuses,
    }
}

fn dependency_trigger_failure(node: &Node, trigger_evaluation: &TriggerEvaluation) -> FailureInfo {
    FailureInfo::new(
        FailureClass::Execution,
        "Dependency",
        "UPSTREAM_FAILED",
        format!("upstream dependencies did not satisfy trigger rule {:?}", node.trigger_rule),
        Some(serde_json::json!({
            "parent_statuses": trigger_evaluation.parent_statuses,
            "trigger_rule": trigger_evaluation.trigger_rule,
            "reason": trigger_evaluation.reason,
        })),
    )
}

enum ReducePreflightAction {
    Run,
    Skip,
    Fail(FailureInfo),
}

struct ReducePreflightDecision {
    trigger_evaluation: TriggerEvaluation,
    action: ReducePreflightAction,
}

fn reduce_empty_collection_failure(
    node: &Node,
    trigger_evaluation: &TriggerEvaluation,
) -> FailureInfo {
    FailureInfo::new(
        FailureClass::Execution,
        "Dependency",
        "REDUCE_EMPTY_COLLECTION",
        format!("reduce node {} did not receive any usable upstream outputs", node.id),
        Some(serde_json::json!({
            "node_id": node.id,
            "parent_statuses": trigger_evaluation.parent_statuses,
            "trigger_rule": trigger_evaluation.trigger_rule,
            "reason": trigger_evaluation.reason,
        })),
    )
}

fn reduce_preflight_decision(
    node: &Node,
    dependencies: &[String],
    parent_status_map: &HashMap<String, NodeStatus>,
) -> Result<ReducePreflightDecision, FailureInfo> {
    let config = reduce_execution_config(node)?;
    let ordered_statuses = dependencies
        .iter()
        .map(|dependency| {
            parent_status_map.get(dependency).cloned().ok_or_else(|| {
                FailureInfo::new(
                    FailureClass::Execution,
                    "Dependency",
                    "REDUCE_STATUS_UNAVAILABLE",
                    format!(
                        "missing terminal status for reduce dependency {} on node {}",
                        dependency, node.id
                    ),
                    Some(serde_json::json!({
                        "node_id": node.id,
                        "dependency": dependency,
                    })),
                )
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let parent_statuses = dependencies
        .iter()
        .zip(ordered_statuses.iter())
        .map(|(node_id, status)| TriggerParentStatus {
            node_id: node_id.clone(),
            status: crate::status_string(status),
        })
        .collect::<Vec<_>>();
    let usable_inputs = ordered_statuses
        .iter()
        .filter(|status| matches!(status, NodeStatus::Success | NodeStatus::Cached))
        .count();

    match config.mode {
        ReduceExecutionMode::AllSuccess => {
            let trigger_evaluation =
                trigger_evaluation_for_dependencies(node, dependencies, &ordered_statuses);
            if !trigger_evaluation.satisfied {
                return Ok(ReducePreflightDecision {
                    trigger_evaluation: trigger_evaluation.clone(),
                    action: ReducePreflightAction::Fail(dependency_trigger_failure(
                        node,
                        &trigger_evaluation,
                    )),
                });
            }
            let action = if usable_inputs > 0 {
                ReducePreflightAction::Run
            } else {
                match config.empty_policy {
                    ReduceEmptyPolicy::Allow => ReducePreflightAction::Run,
                    ReduceEmptyPolicy::Skip => ReducePreflightAction::Skip,
                    ReduceEmptyPolicy::Forbid => ReducePreflightAction::Fail(
                        reduce_empty_collection_failure(node, &trigger_evaluation),
                    ),
                }
            };
            Ok(ReducePreflightDecision { trigger_evaluation, action })
        }
        ReduceExecutionMode::Partial => {
            let reason = if usable_inputs > 0 {
                "reduce partial mode accepts terminal upstreams when at least one usable input exists"
            } else {
                match config.empty_policy {
                    ReduceEmptyPolicy::Allow => {
                        "reduce empty policy allows execution when no usable upstream outputs exist"
                    }
                    ReduceEmptyPolicy::Skip => {
                        "reduce empty policy skips execution when no usable upstream outputs exist"
                    }
                    ReduceEmptyPolicy::Forbid => {
                        "reduce empty policy forbids execution when no usable upstream outputs exist"
                    }
                }
            };
            let trigger_evaluation = TriggerEvaluation {
                trigger_rule: "reduce_partial".to_string(),
                satisfied: usable_inputs > 0 || config.empty_policy == ReduceEmptyPolicy::Allow,
                reason: reason.to_string(),
                parent_statuses,
            };
            let action = if usable_inputs > 0 || config.empty_policy == ReduceEmptyPolicy::Allow {
                ReducePreflightAction::Run
            } else if config.empty_policy == ReduceEmptyPolicy::Skip {
                ReducePreflightAction::Skip
            } else {
                ReducePreflightAction::Fail(reduce_empty_collection_failure(
                    node,
                    &trigger_evaluation,
                ))
            };
            Ok(ReducePreflightDecision { trigger_evaluation, action })
        }
    }
}

fn invalidated_downstream_nodes(graph: &Graph, selected_nodes: &[String]) -> Vec<String> {
    let mut adjacency = BTreeMap::<String, Vec<String>>::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from.node_id.clone()).or_default().push(edge.to.node_id.clone());
    }
    let mut visited = BTreeSet::new();
    let mut queue = selected_nodes.iter().cloned().collect::<Vec<_>>();
    while let Some(node_id) = queue.pop() {
        if let Some(children) = adjacency.get(&node_id) {
            for child in children {
                if visited.insert(child.clone()) {
                    queue.push(child.clone());
                }
            }
        }
    }
    for selected in selected_nodes {
        visited.remove(selected);
    }
    visited.into_iter().collect()
}

fn remember_first_timestamp(slot: &mut Option<u128>, unix_ms: u128) {
    if slot.is_none() {
        *slot = Some(unix_ms);
    }
}

fn lifecycle_terminal_state(
    status: &NodeStatus,
    failure: Option<&FailureInfo>,
    skip_reason: Option<&bijux_dag_artifacts::SkipReason>,
) -> crate::NodeState {
    match status {
        NodeStatus::Success => crate::NodeState::Success,
        NodeStatus::Cached => crate::NodeState::Cached,
        NodeStatus::Cancelled => crate::NodeState::Cancelled,
        NodeStatus::Skipped => {
            if skip_reason.is_some_and(|reason| reason.reason == "cancelled") {
                crate::NodeState::Cancelled
            } else {
                crate::NodeState::Skipped
            }
        }
        NodeStatus::Failed => {
            if failure.is_some_and(|record| {
                matches!(record.code.as_str(), "RUN_TIMEOUT" | "EXEC_TIMEOUT")
            }) {
                crate::NodeState::TimedOut
            } else if failure.is_some_and(|record| record.code == "RUN_ABORTED") {
                crate::NodeState::Cancelled
            } else {
                crate::NodeState::Failed
            }
        }
    }
}

fn lifecycle_terminal_cause(
    status: &NodeStatus,
    failure: Option<&FailureInfo>,
    skip_reason: Option<&bijux_dag_artifacts::SkipReason>,
) -> crate::TransitionCause {
    match status {
        NodeStatus::Success => crate::TransitionCause::ExecutionSucceeded,
        NodeStatus::Cached => crate::TransitionCause::CachedReuse,
        NodeStatus::Cancelled => crate::TransitionCause::CancelRequested,
        NodeStatus::Skipped => {
            if skip_reason.is_some_and(|reason| reason.reason == "cancelled") {
                crate::TransitionCause::CancelRequested
            } else {
                crate::TransitionCause::SelectionFiltered
            }
        }
        NodeStatus::Failed => match failure.map(|record| record.code.as_str()) {
            Some("POLICY_DENIED") => crate::TransitionCause::PolicyDenied,
            Some("UPSTREAM_FAILED") => crate::TransitionCause::DependencyFailed,
            Some("RUN_TIMEOUT" | "EXEC_TIMEOUT") => crate::TransitionCause::TimeoutExceeded,
            Some("RUN_ABORTED") => crate::TransitionCause::ExecutionAborted,
            _ => crate::TransitionCause::ExecutionFailed,
        },
    }
}

fn append_lifecycle_transition(
    transitions: &mut Vec<bijux_dag_artifacts::NodeLifecycleTransition>,
    from_state: crate::NodeState,
    to_state: crate::NodeState,
    cause: crate::TransitionCause,
    unix_ms: u128,
) -> Result<(), RuntimeError> {
    crate::validate_node_transition(&crate::NodeTransition {
        from: from_state.clone(),
        to: to_state.clone(),
        cause: cause.clone(),
    })
    .map_err(RuntimeError::Executor)?;
    transitions.push(bijux_dag_artifacts::NodeLifecycleTransition {
        from_state: crate::node_state_string(&from_state),
        to_state: crate::node_state_string(&to_state),
        cause: crate::transition_cause_string(&cause),
        unix_ms,
    });
    Ok(())
}

fn build_lifecycle_trace(
    run_started_unix_ms: u128,
    timestamps: Option<&NodeLifecycleTimestamps>,
    terminal_state: crate::NodeState,
    terminal_cause: crate::TransitionCause,
    finished_unix_ms: u128,
) -> Result<(String, Vec<bijux_dag_artifacts::NodeLifecycleTransition>), RuntimeError> {
    let mut transitions = Vec::new();
    let mut current_state = crate::NodeState::Pending;

    if let Some(timestamps) = timestamps {
        if let Some(eligible_unix_ms) = timestamps.eligible_unix_ms {
            append_lifecycle_transition(
                &mut transitions,
                current_state.clone(),
                crate::NodeState::Eligible,
                crate::TransitionCause::SchedulerEligible,
                eligible_unix_ms.max(run_started_unix_ms),
            )?;
            current_state = crate::NodeState::Eligible;
        }
        if let Some(queued_unix_ms) = timestamps.queued_unix_ms {
            append_lifecycle_transition(
                &mut transitions,
                current_state.clone(),
                crate::NodeState::Queued,
                crate::TransitionCause::SchedulerQueued,
                queued_unix_ms,
            )?;
            current_state = crate::NodeState::Queued;
        }
        if let Some(running_unix_ms) = timestamps.running_unix_ms {
            append_lifecycle_transition(
                &mut transitions,
                current_state.clone(),
                crate::NodeState::Running,
                crate::TransitionCause::ExecutionStarted,
                running_unix_ms,
            )?;
            current_state = crate::NodeState::Running;
        }
    }

    append_lifecycle_transition(
        &mut transitions,
        current_state,
        terminal_state.clone(),
        terminal_cause,
        finished_unix_ms,
    )?;

    Ok((crate::node_state_string(&terminal_state), transitions))
}

fn write_scheduler_invariant_bundle(
    ctx: &RunContext,
    loop_index: u64,
    checkpoint: &ExecutionCheckpoint,
    violations: &[String],
) -> Result<(), RuntimeError> {
    let payload = serde_json::json!({
        "loop_index": loop_index,
        "checkpoint": checkpoint,
        "violations": violations,
    });
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("scheduler.invariant-bundle.json"),
        &serde_json::to_vec_pretty(&payload)?,
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn record_skipped_node(
    runtime: &Runtime,
    ctx: &RunContext,
    graph: &Graph,
    run_log: &mut std::fs::File,
    run_log_index: &mut Vec<serde_json::Value>,
    failure_propagation_records: &mut Vec<FailurePropagationRecord>,
    status_map: &mut HashMap<String, NodeStatus>,
    options: &RuntimeConfig,
    node_id: &str,
    reason: &str,
    blocking_nodes: &[String],
    run_started_unix_ms: u128,
    lifecycle_timestamps: &HashMap<String, NodeLifecycleTimestamps>,
) -> Result<(), RuntimeError> {
    if status_map.contains_key(node_id) {
        return Ok(());
    }
    let node_status =
        if reason == "cancelled" { NodeStatus::Cancelled } else { NodeStatus::Skipped };
    sacred_execution::guard_terminal_node_status(&node_status)?;
    status_map.insert(node_id.to_string(), node_status.clone());
    let node_kind = graph
        .nodes
        .iter()
        .find(|n| n.id == node_id)
        .map(|n| n.kind.clone())
        .unwrap_or(NodeKind::Const);
    let (aid, aver) = runtime.adapter_meta_for_kind(&node_kind);
    let aschema = runtime.adapter_schema_for_kind(&node_kind);
    let adapter_hash = runtime.adapter_for_kind(&node_kind).ok().and_then(|a| a.binary_hash());
    let started = ctx.clock.now_unix_ms();
    let skip_reason = bijux_dag_artifacts::SkipReason { reason: reason.to_string() };
    let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
        run_started_unix_ms,
        lifecycle_timestamps.get(node_id),
        lifecycle_terminal_state(&node_status, None, Some(&skip_reason)),
        lifecycle_terminal_cause(&node_status, None, Some(&skip_reason)),
        started,
    )?;
    sacred_execution::run_write_trace(
        ctx,
        graph,
        node_id,
        node_status.clone(),
        None,
        Vec::new(),
        started,
        started,
        1,
        None,
        &aid,
        &aver,
        &aschema,
        None,
        adapter_hash,
        None,
        None,
        Some(skip_reason),
        Some(crate::transition_cause_for_skip_reason(reason).to_string()),
        Some(lifecycle_state),
        lifecycle_transitions,
        Some(ReplayProvenance {
            node_action: "skipped".to_string(),
            source_run_id: options.parent_run_id.clone(),
        }),
    )?;
    failure_propagation_records.push(failure_propagation_record(
        node_id,
        &crate::status_string(&node_status),
        reason,
        Some(crate::failure_mode_name(&options.failure_propagation)),
        blocking_nodes.to_vec(),
    ));
    engine_record::append_indexed_event(
        run_log,
        run_log_index,
        serde_json::json!({
            "event": "node_blocked",
            "ts": ctx.clock.now_unix_ms(),
            "node_id": node_id,
            "reason": reason,
        }),
    )?;
    engine_record::append_indexed_event(
        run_log,
        run_log_index,
        serde_json::json!({
            "event": "node_skipped",
            "ts": ctx.clock.now_unix_ms(),
            "node_id": node_id,
            "status": crate::status_string(&node_status),
            "reason": reason,
        }),
    )?;
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn record_failed_node(
    runtime: &Runtime,
    ctx: &RunContext,
    graph: &Graph,
    run_log: &mut std::fs::File,
    run_log_index: &mut Vec<serde_json::Value>,
    failure_propagation_records: &mut Vec<FailurePropagationRecord>,
    status_map: &mut HashMap<String, NodeStatus>,
    options: &RuntimeConfig,
    node: &Node,
    failure: FailureInfo,
    transition_cause: &str,
    run_started_unix_ms: u128,
    lifecycle_timestamps: &HashMap<String, NodeLifecycleTimestamps>,
    trigger_evaluation: Option<TriggerEvaluation>,
) -> Result<(), RuntimeError> {
    sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
    status_map.insert(node.id.clone(), NodeStatus::Failed);
    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
    let aschema = runtime.adapter_schema_for_kind(&node.kind);
    let finished_unix_ms = ctx.clock.now_unix_ms();
    let blocking_nodes = trigger_evaluation
        .as_ref()
        .map(|evaluation| {
            evaluation
                .parent_statuses
                .iter()
                .filter(|parent| !matches!(parent.status.as_str(), "success" | "cached"))
                .map(|parent| parent.node_id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
        run_started_unix_ms,
        lifecycle_timestamps.get(&node.id),
        lifecycle_terminal_state(&NodeStatus::Failed, Some(&failure), None),
        lifecycle_terminal_cause(&NodeStatus::Failed, Some(&failure), None),
        finished_unix_ms,
    )?;
    sacred_execution::run_write_trace(
        ctx,
        graph,
        &node.id,
        NodeStatus::Failed,
        Some(failure.clone()),
        Vec::new(),
        finished_unix_ms,
        finished_unix_ms,
        1,
        None,
        &aid,
        &aver,
        &aschema,
        None,
        runtime.adapter_for_kind(&node.kind).ok().and_then(|adapter| adapter.binary_hash()),
        trigger_evaluation.clone(),
        None,
        None,
        Some(transition_cause.to_string()),
        Some(lifecycle_state),
        lifecycle_transitions,
        Some(ReplayProvenance {
            node_action: "skipped".to_string(),
            source_run_id: options.parent_run_id.clone(),
        }),
    )?;
    failure_propagation_records.push(failure_propagation_record(
        &node.id,
        "failed",
        crate::failure_propagation_cause(Some(&failure)),
        Some(crate::failure_mode_name(&options.failure_propagation)),
        blocking_nodes,
    ));
    engine_record::append_indexed_event(
        run_log,
        run_log_index,
        serde_json::json!({
            "event": "node_finished",
            "ts": finished_unix_ms,
            "node_id": node.id,
            "status": "failed",
            "reason": crate::failure_propagation_cause(Some(&failure)),
            "failure_code": failure.code,
        }),
    )?;
    Ok(())
}

pub fn execute(
    runtime: &Runtime,
    graph: &Graph,
    plan: crate::ExecutionPlan,
    out_dir: impl AsRef<Path>,
    options: RuntimeConfig,
) -> Result<PathBuf, RuntimeError> {
    let out_dir = out_dir.as_ref().to_path_buf();
    if let (Some(run_id), Some(resume_run_id)) = (&options.run_id, &options.resume_run_id) {
        if run_id != resume_run_id {
            return Err(RuntimeError::Executor(format!(
                "resume run id {} does not match requested run id {}",
                resume_run_id, run_id
            )));
        }
    }
    if options.resume_run_id.is_some() && options.parent_run_id.is_some() {
        return Err(RuntimeError::Executor(
            "resume mode cannot be combined with parent run replay".to_string(),
        ));
    }
    let requested_run_id = options.resume_run_id.clone().or_else(|| options.run_id.clone());
    let run_dir = if let Some(ref run_id) = options.resume_run_id {
        RunDir::resume_with_id(&out_dir, run_id)?
    } else if let Some(ref run_id) = requested_run_id {
        RunDir::create_with_id(&out_dir, run_id)?
    } else {
        RunDir::create(&out_dir)?
    };
    let graph_fp = graph.graph_fingerprint()?;
    let graph_json = serde_json::json!({
        "graph": graph.canonicalize(),
        "graph_fingerprint": graph_fp,
    });
    run_dir.write_graph_snapshot(&serde_json::to_string_pretty(&graph_json)?)?;

    let run_id = requested_run_id.clone().unwrap_or_else(|| {
        let dir_name = run_dir
            .final_path()
            .file_name()
            .map(|v| v.to_string_lossy().to_string())
            .unwrap_or_default();
        dir_name.strip_prefix("run-").unwrap_or(dir_name.as_str()).to_string()
    });

    let started_unix_ms = runtime.clock.now_unix_ms();
    let effective_cache_dir = options.cache_dir.clone().or_else(cache_dir_from_env);
    let mut manifest = Manifest {
        manifest_version: "run-manifest/v0.1".to_string(),
        run_id: run_id.clone(),
        created_unix_ms: runtime.clock.now_unix_ms(),
        started_unix_ms,
        finished_unix_ms: started_unix_ms,
        graph_snapshot: "graph.snapshot.json".to_string(),
        status: "success".to_string(),
        spec: SPEC_VERSION.to_string(),
        graph_fingerprint: graph_fp,
        planner_contract_version: plan.planner_contract_version.clone(),
        planner_fingerprint: Some(plan.planner_fingerprint.clone()),
        execution_fingerprint: Some(plan.execution_fingerprint.clone()),
        evidence_fingerprint: Some(plan.evidence_fingerprint.clone()),
        tool_version: crate::tool_version(),
        jobs: options.jobs.max(1),
        adapters: registered_adapters(),
        outputs: Vec::new(),
        node_counts: NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: options.policy.deny_network,
            deny_env: options.policy.deny_env,
            deny_clock: options.policy.deny_clock,
            clean_env: options.policy.clean_env,
            container_image_reference_policy: options.policy.container_image_reference_policy,
        },
        cache_mode: cache_mode_string(&options.cache_mode),
        cache_dir: effective_cache_dir.as_ref().map(|p| p.display().to_string()),
        run_timeout_ms: options.run_timeout_ms,
        run_timeout_behavior: options.run_timeout_ms.map(|_| {
            match options.run_timeout_behavior {
                crate::RunTimeoutBehavior::FinishRunning => "finish_running",
                crate::RunTimeoutBehavior::CancelRunning => "cancel_running",
            }
            .to_string()
        }),
        run_cancellation_cause: None,
        run_metadata: None,
        run_summary: None,
    };
    manifest.run_metadata = Some(RunMetadata {
        submission_source: options.submission_source.clone(),
        trigger_source: options.trigger_source.clone(),
        operator: options.operator.clone(),
        labels: options.labels.clone(),
        parent_run_id: options.parent_run_id.clone(),
        source_run_id: options.parent_run_id.clone(),
        graph_inputs: graph
            .effective_inputs()
            .map_err(|_| RuntimeError::Graph(bijux_dag_core::GraphError::ValidationFailed))?
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
    });
    run_dir.write_manifest(&manifest)?;

    let registered = registered_adapters();
    let prov = Provenance {
        os: std::env::consts::OS.to_string(),
        arch: std::env::consts::ARCH.to_string(),
        rustc: crate::rustc_version(),
        tool_version: crate::tool_version(),
        planner_contract_version: Some(manifest.planner_contract_version.clone()),
        graph_fingerprint: Some(manifest.graph_fingerprint.clone()),
        planner_fingerprint: manifest.planner_fingerprint.clone(),
        execution_fingerprint: manifest.execution_fingerprint.clone(),
        evidence_fingerprint: manifest.evidence_fingerprint.clone(),
        runtime_fingerprint: Some(crate::runtime_fingerprint(&registered)),
        policy_fingerprint: Some(crate::policy_fingerprint(&options.policy)),
        adapters: registered,
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: options.policy.deny_network,
            deny_env: options.policy.deny_env,
            deny_clock: options.policy.deny_clock,
            clean_env: options.policy.clean_env,
            container_image_reference_policy: options.policy.container_image_reference_policy,
        },
        time_source: "system_clock".to_string(),
    };
    write_provenance(run_dir.provenance_path(), &prov)?;

    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    crate::install_runtime_cancellation_handler();
    crate::register_runtime_cancellation_flag(&cancel);
    let stop_request_watcher =
        StopRequestWatcher::spawn(run_dir.stop_request_path(), Arc::clone(&cancel));

    let run_dir_arc = Arc::new(run_dir.clone());
    let store = crate::store::ArtifactStore::new(Arc::clone(&run_dir_arc), Arc::clone(&runtime.fs));
    let mut run_log = store.open_run_log()?;
    let mut run_log_index: Vec<serde_json::Value> = Vec::new();
    let mut run_audit_events: Vec<serde_json::Value> = Vec::new();
    let mut failure_propagation_records: Vec<FailurePropagationRecord> = Vec::new();
    let mut isolated_branch_failures = BTreeMap::<String, BTreeSet<String>>::new();
    let mut node_metric_rows: Vec<NodeMetrics> = Vec::new();
    let mut metrics_registry = InMemoryMetricsRegistry::default();
    engine_record::append_indexed_event(
        &mut run_log,
        &mut run_log_index,
        serde_json::json!({
            "event": "run_started",
            "ts": started_unix_ms,
        }),
    )?;
    run_audit_events.push(serde_json::json!({
        "action": "start",
        "ts": started_unix_ms,
        "run_id": manifest.run_id.clone(),
    }));

    let layout = RunDirLayout {
        run_id: run_id.clone(),
        staging_path: run_dir.staging_path().to_path_buf(),
        final_path: run_dir.final_path().to_path_buf(),
    };
    let resolved = graph.resolve_graph()?;
    let ambient_env: BTreeMap<String, String> = std::env::vars().collect();
    let mut node_definition_fingerprints = HashMap::new();
    let mut declared_environment_fingerprints = HashMap::new();
    let mut params_fingerprints = HashMap::new();
    let mut command_fingerprints = HashMap::new();
    let mut base_fps = HashMap::new();
    for node in &graph.nodes {
        let params = resolved.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        let node_definition_fp = graph.node_fingerprint_with_params(node, &params)?;
        let env_allowlist = crate::effective_env_allowlist(node);
        let declared_env = crate::declared_environment(
            &ambient_env,
            options.policy.clean_env,
            &env_allowlist,
            &[],
        );
        let env_fp = crate::sha256_bytes(&serde_json::to_vec(&declared_env)?);
        node_definition_fingerprints.insert(node.id.clone(), node_definition_fp.clone());
        declared_environment_fingerprints.insert(node.id.clone(), env_fp.clone());
        params_fingerprints.insert(node.id.clone(), crate::params_fingerprint(&params)?);
        command_fingerprints
            .insert(node.id.clone(), crate::command_fingerprint(graph, node, &params)?);
        base_fps.insert(
            node.id.clone(),
            crate::sha256_bytes(format!("{node_definition_fp}:{env_fp}").as_bytes()),
        );
    }
    let mut resolved_params = HashMap::new();
    for node in &graph.nodes {
        let params = resolved.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        let bindings =
            NodePathBindings::for_host(&layout, &node.id, effective_cache_dir.as_deref());
        resolved_params.insert(
            node.id.clone(),
            bind_path_variables_in_value(&params, &bindings).map_err(RuntimeError::Executor)?,
        );
    }
    let graph_fingerprint = Arc::new(Mutex::new(base_fps.clone()));
    let ctx = RunContext {
        run_dir: Arc::clone(&run_dir_arc),
        replay_source_run_dir: options.replay_source_run_dir.clone(),
        graph_fingerprint: Arc::clone(&graph_fingerprint),
        node_definition_fingerprints: Arc::new(node_definition_fingerprints),
        declared_environment_fingerprints: Arc::new(declared_environment_fingerprints),
        params_fingerprints: Arc::new(params_fingerprints),
        command_fingerprints: Arc::new(command_fingerprints),
        planner_contract_version: plan.planner_contract_version.clone(),
        execution_fingerprint: plan.execution_fingerprint.clone(),
        evidence_fingerprint: plan.evidence_fingerprint.clone(),
        execution_contract_fingerprint: crate::execution_contract_fingerprint(&options),
        resolved_params,
        effective_cache_dir: effective_cache_dir.clone(),
        fs: Arc::clone(&runtime.fs),
        clock: Arc::clone(&runtime.clock),
        store,
        policy: options.policy.clone(),
        absolute_path_policy: options.absolute_path_policy,
        cancellation_requested: Arc::clone(&cancel),
    };
    let requested_selectors = options
        .selectors
        .include
        .iter()
        .map(|selector| crate::requested_selector_label("include", selector))
        .chain(
            options
                .upstream_selection_targets
                .iter()
                .map(|node_id| crate::requested_upstream_target_label(node_id)),
        )
        .chain(
            options
                .downstream_selection_roots
                .iter()
                .map(|node_id| crate::requested_downstream_root_label(node_id)),
        )
        .chain(
            options
                .selectors
                .exclude
                .iter()
                .map(|selector| crate::requested_selector_label("exclude", selector)),
        )
        .collect();
    if partial_rerun_selected(&options)
        && options.downstream_selection_roots.is_empty()
        && !options.partial_rerun_dependency_closure
    {
        return Err(RuntimeError::Executor(
            "partial rerun requires dependency closure to prevent stale downstream reuse"
                .to_string(),
        ));
    }
    let explicit_rerun_targets = selected_rerun_targets(graph, &options);
    let explicit_rerun_target_set = explicit_rerun_targets.iter().cloned().collect::<BTreeSet<_>>();
    let partial_rerun_contract =
        partial_rerun_selected(&options).then(|| crate::PartialRerunContract {
            selected_nodes: explicit_rerun_targets.clone(),
            invalidated_downstream_nodes: invalidated_downstream_nodes(
                graph,
                &explicit_rerun_targets,
            ),
            stale_downstream_reuse_forbidden: true,
        });
    let resume_bootstrap = options
        .resume_run_id
        .as_ref()
        .map(|_| prepare_resume_bootstrap(&ctx, graph, &plan, &options))
        .transpose()?;
    let run_snapshot = RunSnapshot {
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        graph_snapshot_path: "graph.snapshot.json".to_string(),
        planner_config: "default".to_string(),
        scheduler_config: "local".to_string(),
        policy_config: "runtime-policy-v0.1".to_string(),
        provenance: "provenance.json".to_string(),
        submission_source: options.submission_source.clone(),
        trigger_source: options.trigger_source.clone(),
        operator: options.operator.clone(),
        labels: options.labels.clone(),
        parent_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        requested_selectors,
        selected_nodes: plan.order.clone(),
        dependency_closure_enabled: options.partial_rerun_dependency_closure,
        replay_source_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        partial_rerun_contract,
    };
    let run_snapshot_path = ctx.run_dir.staging_path().join("run.snapshot.json");
    ctx.fs.write(&run_snapshot_path, &serde_json::to_vec_pretty(&run_snapshot)?)?;
    write_incomplete_run_marker(
        ctx.run_dir.staging_path(),
        "run not finalized; recover or repair before treating artifacts as complete",
    )
    .map_err(|err| RuntimeError::Executor(format!("incomplete run marker write failed: {err}")))?;
    let mut run_attempts = resume_bootstrap
        .as_ref()
        .map(|_| read_resume_attempts(&ctx))
        .transpose()?
        .unwrap_or_default();
    let run_attempt = RunAttempt {
        attempt_index: resume_bootstrap
            .as_ref()
            .map(|bootstrap| bootstrap.attempt_index)
            .unwrap_or(1),
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        parent_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        reason: if options.resume_run_id.is_some() {
            "resume".to_string()
        } else if options.parent_run_id.is_some() {
            "replay_or_retry".to_string()
        } else {
            "initial_submission".to_string()
        },
        resume_summary: resume_bootstrap.as_ref().map(|bootstrap| bootstrap.summary.clone()),
    };
    run_attempts.push(run_attempt);
    let run_attempts_path = ctx.run_dir.staging_path().join("run.attempts.json");
    ctx.fs.write(&run_attempts_path, &serde_json::to_vec_pretty(&run_attempts)?)?;
    let start = Instant::now();
    let mut status_map: HashMap<String, NodeStatus> = HashMap::new();
    let mut cache_proofs: HashMap<String, CacheProof> = HashMap::new();
    let mut fail_fast_aborted = false;
    let mut run_timed_out = false;
    let dep_map = plan.dep_map.clone();
    let mut dependency_counter = sacred_execution::resolve_dependencies(&plan);
    let mut ready_queue = sacred_execution::ready_queue_from_dependencies(&dependency_counter);
    let mut scheduler = crate::build_scheduler(&options.scheduler_policy);
    let scheduler_hook = crate::NoopSchedulerEventHook;
    let local_worker_capacity =
        options.jobs.max(1).min(options.scheduler_policy.max_parallelism.max(1));
    let mut local_workers =
        crate::LocalWorkerPool::<Result<NodeResult, RuntimeError>>::new(local_worker_capacity);
    let mut loop_index: u64 = 0;
    let mut branch_pruned_nodes = BTreeSet::new();
    let mut lifecycle_timestamps = graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), NodeLifecycleTimestamps::default()))
        .collect::<HashMap<_, _>>();
    engine_record::append_indexed_event(
        &mut run_log,
        &mut run_log_index,
        serde_json::json!({
            "event": "plan_built",
            "ts": ctx.clock.now_unix_ms(),
            "nodes": graph.nodes.len(),
        }),
    )?;
    if let Some(parent_run_id) = options.parent_run_id.as_deref() {
        for (branch_node_id, decision) in seed_replayed_branch_pruning(
            ctx.fs.as_ref(),
            &out_dir,
            parent_run_id,
            &plan,
            graph,
            &mut branch_pruned_nodes,
        )? {
            engine_record::append_indexed_event(
                &mut run_log,
                &mut run_log_index,
                serde_json::json!({
                    "event": "branch_decision_replayed",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": branch_node_id,
                    "decision": decision,
                    "source_run_id": parent_run_id,
                }),
            )?;
        }
    }
    if let Some(resume) = &resume_bootstrap {
        for node_id in &resume.reset_nodes {
            clear_resumed_node_dir(&ctx, node_id)?;
        }
        for (branch_node_id, decision) in &resume.branch_decisions {
            for pruned in branch_nodes_to_skip(graph, branch_node_id, decision) {
                branch_pruned_nodes.insert(pruned);
            }
        }
        engine_record::append_indexed_event(
            &mut run_log,
            &mut run_log_index,
            serde_json::json!({
                "event": "run_resumed",
                "ts": ctx.clock.now_unix_ms(),
                "run_id": manifest.run_id.clone(),
                "reused_nodes": resume.summary.reused_nodes.clone(),
                "rerun_nodes": resume.summary.rerun_nodes.clone(),
                "rejected_nodes": resume.summary.rejected_nodes.clone(),
                "failure_mode": match resume.summary.failure_mode {
                    crate::ResumeFailureMode::RerunIncomplete => "rerun_incomplete",
                    crate::ResumeFailureMode::RejectIncomplete => "reject_incomplete",
                },
            }),
        )?;
        run_audit_events.push(serde_json::json!({
            "action": "resume",
            "ts": ctx.clock.now_unix_ms(),
            "run_id": manifest.run_id.clone(),
            "reused_nodes": resume.summary.reused_nodes.clone(),
            "rerun_nodes": resume.summary.rerun_nodes.clone(),
            "rejected_nodes": resume.summary.rejected_nodes.clone(),
        }));
        if !resume.summary.rejected_nodes.is_empty() {
            return Err(RuntimeError::Executor(format!(
                "resume rejected incomplete nodes: {}",
                resume.summary.rejected_nodes.join(", ")
            )));
        }
        for node_id in
            plan.order.iter().filter(|node_id| resume.reusable_nodes.contains_key(*node_id))
        {
            let status = resume.reusable_nodes.get(node_id).expect("resume status");
            ready_queue.take(node_id);
            sacred_execution::guard_terminal_node_status(status)?;
            status_map.insert(node_id.clone(), status.clone());
            for newly_ready in dependency_counter.mark_completed(node_id) {
                ready_queue.insert(newly_ready);
            }
            engine_record::append_indexed_event(
                &mut run_log,
                &mut run_log_index,
                serde_json::json!({
                    "event": "node_resume_reused",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": crate::status_string(status),
                }),
            )?;
        }
    }
    let initial_ready = ready_queue.snapshot_sorted();
    for node_id in &initial_ready {
        scheduler_hook.on_node_eligible(node_id);
        if let Some(timestamps) = lifecycle_timestamps.get_mut(node_id) {
            remember_first_timestamp(&mut timestamps.eligible_unix_ms, started_unix_ms);
        }
    }
    for event in engine_observe::node_eligible_events(
        &initial_ready,
        ctx.clock.now_unix_ms(),
        "root_ready",
        "all_success",
    ) {
        engine_record::append_indexed_event(&mut run_log, &mut run_log_index, event)?;
    }
    while !ready_queue.is_empty() {
        loop_index = loop_index.saturating_add(1);
        let decision = engine_dispatch::next_scheduler_decision(
            scheduler.as_mut(),
            graph,
            &mut ready_queue,
            &options,
            start,
            cancel.load(Ordering::SeqCst),
        );
        if decision.cancelled {
            break;
        }
        if decision.timed_out {
            run_timed_out = true;
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "run_timeout",
                    "ts": ctx.clock.now_unix_ms(),
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "run_timeout",
                "ts": ctx.clock.now_unix_ms(),
            }));
            break;
        }
        let ready_candidates = decision.ready_candidates.clone();
        let blocked_reasons = decision.blocked_reasons.clone();
        let decision_reason = decision.decision_reason.clone();
        let tie_break_reason = decision.tie_break_reason.clone();
        let batch = decision.batch;
        let mut blocked_by_budget = decision.blocked_by_budget;
        blocked_by_budget.sort();
        engine_record::append_indexed_event(
            &mut run_log,
            &mut run_log_index,
            serde_json::json!({
                "event": "scheduler_decision",
                "ts": ctx.clock.now_unix_ms(),
                "loop_index": loop_index,
                "ready_queue_depth": ready_queue.len(),
                "ready_candidates": ready_candidates,
                "batch": batch.clone(),
                "blocked_by_budget": blocked_by_budget.clone(),
                "blocked_reasons": blocked_reasons.clone(),
                "decision_reason": decision_reason,
                "tie_break_reason": tie_break_reason,
            }),
        )?;
        for node_id in &blocked_by_budget {
            scheduler_hook.on_node_blocked_by_budget(node_id);
            let reason_code = blocked_reasons
                .get(node_id)
                .cloned()
                .unwrap_or_else(|| "blocked_by_policy".to_string());
            engine_record::append_indexed_event(
                &mut run_log,
                &mut run_log_index,
                serde_json::json!({
                    "event": "node_blocked",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "reason": reason_code,
                }),
            )?;
        }
        let schedule_unix_ms = ctx.clock.now_unix_ms();
        for node_id in &batch {
            if let Some(timestamps) = lifecycle_timestamps.get_mut(node_id) {
                remember_first_timestamp(&mut timestamps.queued_unix_ms, schedule_unix_ms);
            }
        }
        let forced_batch = batch.len() == 1;

        let mut skipped: Vec<(String, String, Vec<String>)> = Vec::new();
        let mut cached: Vec<(String, Node, CacheProof)> = Vec::new();
        let mut to_start: Vec<(String, Node, Value)> = Vec::new();
        let mut preflight_failures: Vec<(
            String,
            Node,
            FailureInfo,
            String,
            Option<TriggerEvaluation>,
        )> = Vec::new();
        let mut trigger_evaluations = HashMap::<String, TriggerEvaluation>::new();

        for node_id in &batch {
            if status_map.contains_key(node_id) {
                continue;
            }
            if branch_pruned_nodes.contains(node_id) {
                skipped.push((
                    node_id.clone(),
                    "branch_decision_not_selected".to_string(),
                    Vec::new(),
                ));
                continue;
            }
            if let Some(reason) = plan.filter_reasons.get(node_id) {
                skipped.push((node_id.clone(), reason.clone(), Vec::new()));
                continue;
            }
            if matches!(options.failure_propagation, crate::FailurePropagationMode::IsolateBranch) {
                if let Some(blocking_nodes) = isolated_branch_failures.get(node_id) {
                    skipped.push((
                        node_id.clone(),
                        "isolated_branch_failure".to_string(),
                        blocking_nodes.iter().cloned().collect(),
                    ));
                    continue;
                }
            }
            if cancel.load(Ordering::SeqCst) {
                skipped.push((node_id.clone(), "cancelled".to_string(), Vec::new()));
                continue;
            }
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?
                .clone();
            if let Some(limit) = options.run_timeout_ms {
                if start.elapsed() >= Duration::from_millis(limit) {
                    run_timed_out = true;
                    preflight_failures.push((
                        node_id.clone(),
                        node,
                        FailureInfo::new(
                            FailureClass::Timeout,
                            "Timeout",
                            "RUN_TIMEOUT",
                            "run timeout exceeded before node start",
                            Some(serde_json::json!({ "run_timeout_ms": limit })),
                        ),
                        "TimeoutExceeded".to_string(),
                        None,
                    ));
                    continue;
                }
            }

            let dependencies = upstream_nodes(&dep_map, node_id);
            let parent_status_map = dependencies
                .iter()
                .filter_map(|dependency| {
                    status_map.get(dependency).cloned().map(|status| (dependency.clone(), status))
                })
                .collect::<HashMap<_, _>>();
            if node.semantic_kind == SemanticNodeKind::Reduce
                && parent_status_map.len() == dependencies.len()
            {
                let decision =
                    match reduce_preflight_decision(&node, &dependencies, &parent_status_map) {
                        Ok(decision) => decision,
                        Err(failure) => {
                            preflight_failures.push((
                                node_id.clone(),
                                node,
                                failure,
                                "DependencyFailed".to_string(),
                                None,
                            ));
                            continue;
                        }
                    };
                match decision.action {
                    ReducePreflightAction::Run => {
                        trigger_evaluations
                            .insert(node_id.clone(), decision.trigger_evaluation.clone());
                    }
                    ReducePreflightAction::Skip => {
                        skipped.push((
                            node_id.clone(),
                            "empty_reduce_collection".to_string(),
                            Vec::new(),
                        ));
                        continue;
                    }
                    ReducePreflightAction::Fail(failure) => {
                        engine_record::append_indexed_event(
                            &mut run_log,
                            &mut run_log_index,
                            serde_json::json!({
                                "event": "node_blocked",
                                "ts": ctx.clock.now_unix_ms(),
                                "node_id": node_id,
                                "reason": "blocked_by_reduce_policy",
                                "blocking_nodes": dependencies,
                                "trigger_rule": decision.trigger_evaluation.trigger_rule,
                            }),
                        )?;
                        preflight_failures.push((
                            node_id.clone(),
                            node,
                            failure,
                            "DependencyFailed".to_string(),
                            Some(decision.trigger_evaluation),
                        ));
                        continue;
                    }
                }
            } else if !dependencies.is_empty() && parent_status_map.len() == dependencies.len() {
                let parent_statuses = dependencies
                    .iter()
                    .filter_map(|dependency| parent_status_map.get(dependency).cloned())
                    .collect::<Vec<_>>();
                let trigger_evaluation =
                    trigger_evaluation_for_dependencies(&node, &dependencies, &parent_statuses);
                trigger_evaluations.insert(node_id.clone(), trigger_evaluation.clone());
                if !trigger_evaluation.satisfied {
                    let trigger_rule = node.trigger_rule.clone();
                    let failure = dependency_trigger_failure(&node, &trigger_evaluation);
                    preflight_failures.push((
                        node_id.clone(),
                        node,
                        failure,
                        "DependencyFailed".to_string(),
                        Some(trigger_evaluation),
                    ));
                    engine_record::append_indexed_event(
                        &mut run_log,
                        &mut run_log_index,
                        serde_json::json!({
                            "event": "node_blocked",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "reason": "blocked_by_trigger_rule",
                            "blocking_nodes": dependencies,
                            "trigger_rule": trigger_rule,
                        }),
                    )?;
                    continue;
                }
            }
            let resolved_params = ctx.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
            let mut node = node;
            if matches!(options.run_timeout_behavior, crate::RunTimeoutBehavior::CancelRunning) {
                if let Some(limit_ms) = options.run_timeout_ms {
                    let elapsed_ms = start.elapsed().as_millis() as u64;
                    let remaining_ms = limit_ms.saturating_sub(elapsed_ms);
                    if remaining_ms == 0 {
                        run_timed_out = true;
                        preflight_failures.push((
                            node_id.clone(),
                            node,
                            FailureInfo::new(
                                FailureClass::Timeout,
                                "Timeout",
                                "RUN_TIMEOUT",
                                "run timeout exceeded before node start",
                                Some(serde_json::json!({ "run_timeout_ms": limit_ms })),
                            ),
                            "TimeoutExceeded".to_string(),
                            None,
                        ));
                        continue;
                    }
                    let resolved_timeout_ms =
                        resolved_params.get("timeout_ms").and_then(|value| value.as_u64());
                    let effective_timeout_ms = node
                        .timeout_ms
                        .or(resolved_timeout_ms)
                        .map_or(remaining_ms, |timeout_ms| timeout_ms.min(remaining_ms));
                    node.timeout_ms = Some(effective_timeout_ms);
                }
            }

            if node.retry.max_attempts > 0
                && (node.effects.contains(&Effect::Clock)
                    || node.effects.contains(&Effect::Network))
                && !graph.inputs.contains_key("random_seed")
                && !graph.nondeterminism_allowed
            {
                return Err(RuntimeError::Executor(
                    "retry not allowed for nondeterministic node".to_string(),
                ));
            }
            if options.policy.deny_network && node.effects.contains(&Effect::Network) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "network",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo::new(
                        FailureClass::Policy,
                        "Policy",
                        "POLICY_DENIED",
                        "network effect denied by policy",
                        Some(serde_json::json!({ "effect": "network" })),
                    ),
                    "PolicyDenied".to_string(),
                    None,
                ));
                continue;
            }
            if options.policy.deny_env && node.effects.contains(&Effect::Env) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "env",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo::new(
                        FailureClass::Policy,
                        "Policy",
                        "POLICY_DENIED",
                        "env effect denied by policy",
                        Some(serde_json::json!({ "effect": "env" })),
                    ),
                    "PolicyDenied".to_string(),
                    None,
                ));
                continue;
            }
            if options.policy.deny_clock && node.effects.contains(&Effect::Clock) {
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "policy_denied",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node.id,
                        "reason": "clock",
                    }),
                )?;
                preflight_failures.push((
                    node_id.clone(),
                    node,
                    FailureInfo::new(
                        FailureClass::Policy,
                        "Policy",
                        "POLICY_DENIED",
                        "clock effect denied by policy",
                        Some(serde_json::json!({ "effect": "clock" })),
                    ),
                    "PolicyDenied".to_string(),
                    None,
                ));
                continue;
            }
            let adapter = runtime.adapter_for_kind(&node.kind)?;
            let required = adapter.required_effects();
            let declared = EffectSet::from_effects(&node.effects);
            if required.filesystem && !declared.filesystem
                || required.env && !declared.env
                || required.network && !declared.network
                || required.clock && !declared.clock
            {
                return Err(RuntimeError::Executor("missing required effects".to_string()));
            }

            let adapter_id = adapter.id();
            let adapter_schema = adapter.produces_outputs_schema_version();
            let inputs_index = sacred_execution::run_materialize_inputs(
                &ctx,
                graph,
                &node,
                options.materialize_inputs,
                &status_map,
            )?;
            let base_fp = base_fps.get(&node.id).cloned().unwrap_or_default();
            let node_fp = node_fingerprint_with_inputs(&base_fp, &inputs_index)?;
            set_node_fingerprint(&ctx, &node.id, node_fp.clone());
            if !explicit_rerun_target_set.contains(node_id) {
                let cache_read = sacred_execution::run_cache_lookup(
                    &options,
                    &node,
                    &node_fp,
                    &ctx,
                    Arc::clone(&ctx.fs),
                    &adapter_id.id,
                    &adapter_id.version,
                    &adapter_schema,
                )?;
                let hit = cache_read.hit;
                let cache_proof = crate::cache_hit_proof(cache_read)?;
                if let Some(proof) = cache_proof.clone() {
                    if !hit {
                        cache_proofs.insert(node_id.clone(), proof);
                    }
                }
                if hit {
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "cache_hit",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "cache_hit",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                    }));
                    let proof = cache_proof.ok_or_else(|| {
                        RuntimeError::Executor("cache hit missing verification proof".to_string())
                    })?;
                    cached.push((node_id.clone(), node, proof));
                    continue;
                }
                crate::append_event(
                    &mut run_log,
                    serde_json::json!({
                        "event": "cache_miss",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                    }),
                )?;
                run_log_index.push(serde_json::json!({
                    "event": "cache_miss",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }));
            }

            to_start.push((node_id.clone(), node, resolved_params));
        }

        skipped.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, reason, blocking_nodes) in &skipped {
            record_skipped_node(
                runtime,
                &ctx,
                graph,
                &mut run_log,
                &mut run_log_index,
                &mut failure_propagation_records,
                &mut status_map,
                &options,
                node_id,
                reason,
                blocking_nodes,
                started_unix_ms,
                &lifecycle_timestamps,
            )?;
        }
        preflight_failures.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, node, failure, transition_cause, trigger_evaluation) in &preflight_failures {
            sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
            status_map.insert(node_id.clone(), NodeStatus::Failed);
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let adapter_hash =
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
            let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
                started_unix_ms,
                lifecycle_timestamps.get(node_id),
                lifecycle_terminal_state(&NodeStatus::Failed, Some(failure), None),
                lifecycle_terminal_cause(&NodeStatus::Failed, Some(failure), None),
                started,
            )?;
            sacred_execution::run_write_trace(
                &ctx,
                graph,
                node_id,
                NodeStatus::Failed,
                Some(failure.clone()),
                Vec::new(),
                started,
                started,
                1,
                None,
                &aid,
                &aver,
                &aschema,
                None,
                adapter_hash,
                trigger_evaluation.clone(),
                None,
                None,
                Some(transition_cause.clone()),
                Some(lifecycle_state),
                lifecycle_transitions,
                Some(ReplayProvenance {
                    node_action: "reexecuted".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            let retry_decision = if failure.operator_class() == FailureClass::Policy {
                crate::evaluate_retry_decision(
                    node_id,
                    &crate::build_retry_policy(node),
                    1,
                    &crate::retry_observation_from_failure(failure),
                )
            } else {
                crate::RetryDecision {
                    retryable: false,
                    retry_allowed: false,
                    reason: "pre_execution_failures_are_non_retryable".to_string(),
                    matched_exit_code: None,
                }
            };
            crate::write_attempt_events(
                &ctx,
                node_id,
                &[crate::AttemptEvent {
                    attempt: 1,
                    started_unix_ms: started,
                    finished_unix_ms: started,
                    status: NodeStatus::Failed,
                    stdout_path: None,
                    stderr_path: None,
                    failure: Some(failure.clone()),
                    scheduled_backoff_ms: None,
                    retry_decision: Some(retry_decision.clone()),
                }],
            )?;
            failure_propagation_records.push(failure_propagation_record(
                node_id,
                "failed",
                crate::failure_propagation_cause(Some(failure)),
                Some(crate::failure_mode_name(&options.failure_propagation)),
                trigger_evaluation
                    .as_ref()
                    .map(|evaluation| {
                        evaluation
                            .parent_statuses
                            .iter()
                            .filter(|parent| {
                                !matches!(parent.status.as_str(), "success" | "cached")
                            })
                            .map(|parent| parent.node_id.clone())
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default(),
            ));
            if matches!(options.failure_propagation, crate::FailurePropagationMode::IsolateBranch) {
                record_isolated_branch_descendants(graph, node_id, &mut isolated_branch_failures);
            }
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_finished",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": "failed",
                    "reason": crate::failure_propagation_cause(Some(failure)),
                    "failure_code": failure.code,
                    "retry_reason": retry_decision.reason.clone(),
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_finished",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "status": "failed",
                "reason": crate::failure_propagation_cause(Some(failure)),
                "failure_code": failure.code,
                "retry_reason": retry_decision.reason.clone(),
            }));
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_retry_exhausted",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "attempt": 1,
                    "status": "failed",
                    "failure_code": failure.code,
                    "retry_reason": retry_decision.reason.clone(),
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_retry_exhausted",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "attempt": 1,
                "status": "failed",
                "failure_code": failure.code,
                "retry_reason": retry_decision.reason.clone(),
            }));
        }

        let mut started_ids: Vec<String> = Vec::new();
        let actual_started_ids =
            to_start.iter().map(|(node_id, _, _)| node_id.clone()).collect::<Vec<_>>();
        for (node_id, _, _) in &cached {
            started_ids.push(node_id.clone());
        }
        for node_id in &actual_started_ids {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _, _, _) in &preflight_failures {
            started_ids.push(node_id.clone());
        }
        started_ids.sort();
        let schedule_reason = if forced_batch { "ready" } else { "budget_available" };
        let scheduled_event_unix_ms = ctx.clock.now_unix_ms();
        for node_id in &started_ids {
            scheduler_hook.on_node_scheduled(node_id);
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_scheduled",
                    "ts": scheduled_event_unix_ms,
                    "node_id": node_id,
                    "reason": schedule_reason,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_scheduled",
                "ts": scheduled_event_unix_ms,
                "node_id": node_id,
                "reason": schedule_reason,
            }));
        }

        let checkpoint = ExecutionCheckpoint {
            loop_index,
            ready_queue_depth: ready_queue.len(),
            ready_queue: ready_queue.snapshot_sorted(),
            inflight: actual_started_ids.clone(),
            scheduled: started_ids.clone(),
            blocked_by_budget: blocked_by_budget.clone(),
            blocked_reasons: blocked_reasons.clone(),
            completed_statuses: status_map
                .iter()
                .map(|(node_id, status)| match status {
                    NodeStatus::Success => (node_id.clone(), "success".to_string()),
                    NodeStatus::Failed => (node_id.clone(), "failed".to_string()),
                    NodeStatus::Skipped => (node_id.clone(), "skipped".to_string()),
                    NodeStatus::Cached => (node_id.clone(), "cached".to_string()),
                    NodeStatus::Cancelled => (node_id.clone(), "cancelled".to_string()),
                })
                .collect(),
            failure_propagation_mode: crate::failure_mode_name(&options.failure_propagation)
                .to_string(),
            dependency_closure_enabled: options.partial_rerun_dependency_closure,
            generated_unix_ms: ctx.clock.now_unix_ms(),
        };
        let checkpoint_path = ctx.run_dir.staging_path().join("scheduler.checkpoint.json");
        let _ = ctx
            .fs
            .write(&checkpoint_path, &serde_json::to_vec_pretty(&checkpoint).unwrap_or_default());
        let replay_state = crate::replay_scheduler_checkpoint(&plan, &checkpoint)
            .map_err(RuntimeError::Executor)?;
        let replay_violations = crate::scheduler_invariant_violations(&replay_state);
        if !replay_violations.is_empty() {
            write_scheduler_invariant_bundle(&ctx, loop_index, &checkpoint, &replay_violations)?;
            return Err(RuntimeError::Executor(
                "scheduler state invariants violated after checkpoint replay".to_string(),
            ));
        }
        let started_event_unix_ms = ctx.clock.now_unix_ms();
        for node_id in &actual_started_ids {
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_started",
                    "ts": started_event_unix_ms,
                    "node_id": node_id,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_started",
                "ts": started_event_unix_ms,
                "node_id": node_id,
            }));
        }

        let mut branch_pruned = BTreeSet::new();
        for (node_id, node, cache_proof) in &cached {
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let adapter_hash =
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
            let started = ctx.clock.now_unix_ms();
            let branch_resolution = resolve_branch_decision(&ctx, node);
            let (status, failure, branch_decision, transition_cause) = match branch_resolution {
                Ok(Some(selection)) => {
                    engine_record::append_indexed_event(
                        &mut run_log,
                        &mut run_log_index,
                        serde_json::json!({
                            "event": "branch_decision_selected",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "decision": selection.decision,
                            "used_default": selection.used_default,
                        }),
                    )?;
                    for pruned in branch_nodes_to_skip(graph, node_id, &selection.decision) {
                        branch_pruned_nodes.insert(pruned.clone());
                        branch_pruned.insert(pruned);
                    }
                    (
                        NodeStatus::Cached,
                        None,
                        Some(selection.decision),
                        Some("CachedReuse".to_string()),
                    )
                }
                Ok(None) => (NodeStatus::Cached, None, None, Some("CachedReuse".to_string())),
                Err(failure) => (
                    NodeStatus::Failed,
                    Some(failure.clone()),
                    None,
                    Some(crate::transition_cause_for_failure(Some(&failure)).to_string()),
                ),
            };
            let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
                started_unix_ms,
                lifecycle_timestamps.get(node_id),
                lifecycle_terminal_state(&status, failure.as_ref(), None),
                lifecycle_terminal_cause(&status, failure.as_ref(), None),
                started,
            )?;
            sacred_execution::guard_terminal_node_status(&status)?;
            status_map.insert(node_id.clone(), status.clone());
            sacred_execution::run_write_trace(
                &ctx,
                graph,
                node_id,
                status.clone(),
                failure.clone(),
                Vec::new(),
                started,
                started,
                1,
                Some(cache_proof.clone()),
                &aid,
                &aver,
                &aschema,
                None,
                adapter_hash,
                trigger_evaluations.get(node_id).cloned(),
                branch_decision,
                None,
                transition_cause,
                Some(lifecycle_state),
                lifecycle_transitions,
                Some(ReplayProvenance {
                    node_action: "reused".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            engine_record::append_indexed_event(
                &mut run_log,
                &mut run_log_index,
                serde_json::json!({
                    "event": "node_finished",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": crate::status_string(&status),
                    "reason": failure
                        .as_ref()
                        .map(|failure| crate::failure_propagation_cause(Some(failure))),
                    "failure_code": failure.as_ref().map(|failure| failure.code.clone()),
                }),
            )?;
            if status == NodeStatus::Failed {
                failure_propagation_records.push(failure_propagation_record(
                    node_id,
                    "failed",
                    crate::failure_propagation_cause(failure.as_ref()),
                    Some(crate::failure_mode_name(&options.failure_propagation)),
                    Vec::new(),
                ));
                if matches!(
                    options.failure_propagation,
                    crate::FailurePropagationMode::IsolateBranch
                ) {
                    record_isolated_branch_descendants(
                        graph,
                        node_id,
                        &mut isolated_branch_failures,
                    );
                }
            } else {
                let node_fp = node_fingerprint_from_ctx(&ctx, &node.id);
                sacred_execution::run_cache_write(
                    &options,
                    node,
                    &node_fp,
                    &ctx,
                    Arc::clone(&ctx.fs),
                    &aid,
                    &aver,
                    &aschema,
                )?;
            }
        }

        let mut submitted_nodes = HashMap::<String, Node>::new();
        for (node_id, node, params) in &to_start {
            let adapter = runtime.adapter_for_kind(&node.kind)?;
            let ctx_clone = RunContext {
                run_dir: Arc::clone(&ctx.run_dir),
                replay_source_run_dir: ctx.replay_source_run_dir.clone(),
                graph_fingerprint: ctx.graph_fingerprint.clone(),
                node_definition_fingerprints: Arc::clone(&ctx.node_definition_fingerprints),
                declared_environment_fingerprints: Arc::clone(
                    &ctx.declared_environment_fingerprints,
                ),
                params_fingerprints: Arc::clone(&ctx.params_fingerprints),
                command_fingerprints: Arc::clone(&ctx.command_fingerprints),
                planner_contract_version: ctx.planner_contract_version.clone(),
                execution_fingerprint: ctx.execution_fingerprint.clone(),
                evidence_fingerprint: ctx.evidence_fingerprint.clone(),
                execution_contract_fingerprint: ctx.execution_contract_fingerprint.clone(),
                resolved_params: ctx.resolved_params.clone(),
                effective_cache_dir: ctx.effective_cache_dir.clone(),
                fs: Arc::clone(&ctx.fs),
                clock: Arc::clone(&ctx.clock),
                store: ctx.store.clone(),
                policy: ctx.policy.clone(),
                absolute_path_policy: ctx.absolute_path_policy,
                cancellation_requested: Arc::clone(&ctx.cancellation_requested),
            };
            let node_id_clone = node_id.clone();
            let node_for_thread = node.clone();
            let params_for_thread = params.clone();
            let graph_for_thread = graph.clone();
            let retry = node.retry.clone();
            local_workers
                .submit(
                    node_id_clone.clone(),
                    Box::new(move || {
                        let started = ctx_clone.clock.now_unix_ms();
                        let result = sacred_execution::run_retry_logic(
                            adapter.as_ref(),
                            &graph_for_thread,
                            &node_for_thread,
                            &params_for_thread,
                            &ctx_clone,
                            &retry,
                        );
                        let finished = ctx_clone.clock.now_unix_ms();
                        crate::LocalWorkerExecution {
                            started_unix_ms: started,
                            finished_unix_ms: finished,
                            result,
                        }
                    }),
                )
                .map_err(RuntimeError::Executor)?;
            submitted_nodes.insert(node_id_clone, node.clone());
        }
        if cancel.load(Ordering::SeqCst) {
            local_workers.request_cancellation();
        }

        type ResultItem = (String, Node, u128, u128, Result<NodeResult, RuntimeError>);
        let mut results: Vec<ResultItem> = Vec::new();
        for _ in 0..actual_started_ids.len() {
            if cancel.load(Ordering::SeqCst) {
                local_workers.request_cancellation();
            }
            let completion = local_workers.wait_for_completion().map_err(RuntimeError::Executor)?;
            let node = submitted_nodes.remove(&completion.node_id).ok_or_else(|| {
                RuntimeError::Executor("missing worker completion node".to_string())
            })?;
            results.push((
                completion.node_id,
                node,
                completion.started_unix_ms,
                completion.finished_unix_ms,
                completion.result,
            ));
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, node, started, finished, res) in results {
            if let Some(timestamps) = lifecycle_timestamps.get_mut(&node_id) {
                remember_first_timestamp(&mut timestamps.running_unix_ms, started);
            }
            match res {
                Ok(mut result) => {
                    if matches!(
                        options.run_timeout_behavior,
                        crate::RunTimeoutBehavior::CancelRunning
                    ) && result.failure.as_ref().map(|failure| failure.code.as_str())
                        == Some("EXEC_TIMEOUT")
                        && options.run_timeout_ms.is_some_and(|limit_ms| {
                            start.elapsed() >= Duration::from_millis(limit_ms)
                        })
                    {
                        run_timed_out = true;
                    }
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    let adapter_hash =
                        runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
                    let cache_proof = cache_proofs.get(&node_id).cloned();
                    for attempt in &result.attempt_events {
                        crate::append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_attempt_started",
                                "ts": attempt.started_unix_ms,
                                "node_id": node_id,
                                "attempt": attempt.attempt,
                            }),
                        )?;
                        run_log_index.push(serde_json::json!({
                            "event": "node_attempt_started",
                            "ts": attempt.started_unix_ms,
                            "node_id": node_id,
                            "attempt": attempt.attempt,
                        }));
                        crate::append_event(
                            &mut run_log,
                            serde_json::json!({
                                "event": "node_attempt_finished",
                                "ts": attempt.finished_unix_ms,
                                "node_id": node_id,
                                "attempt": attempt.attempt,
                                "status": crate::status_string(&attempt.status),
                            }),
                        )?;
                        run_log_index.push(serde_json::json!({
                            "event": "node_attempt_finished",
                            "ts": attempt.finished_unix_ms,
                            "node_id": node_id,
                            "attempt": attempt.attempt,
                            "status": crate::status_string(&attempt.status),
                        }));
                        if let Some(backoff_ms) = attempt.scheduled_backoff_ms {
                            crate::append_event(
                                &mut run_log,
                                serde_json::json!({
                                    "event": "node_retry_scheduled",
                                    "ts": attempt.finished_unix_ms,
                                    "node_id": node_id,
                                    "attempt": attempt.attempt,
                                    "next_attempt": attempt.attempt + 1,
                                    "backoff_ms": backoff_ms,
                                    "retry_reason": attempt
                                        .retry_decision
                                        .as_ref()
                                        .map(|decision| decision.reason.clone()),
                                    "matched_exit_code": attempt
                                        .retry_decision
                                        .as_ref()
                                        .and_then(|decision| decision.matched_exit_code),
                                }),
                            )?;
                            run_log_index.push(serde_json::json!({
                                "event": "node_retry_scheduled",
                                "ts": attempt.finished_unix_ms,
                                "node_id": node_id,
                                "attempt": attempt.attempt,
                                "next_attempt": attempt.attempt + 1,
                                "backoff_ms": backoff_ms,
                                "retry_reason": attempt
                                    .retry_decision
                                    .as_ref()
                                    .map(|decision| decision.reason.clone()),
                                "matched_exit_code": attempt
                                    .retry_decision
                                    .as_ref()
                                    .and_then(|decision| decision.matched_exit_code),
                            }));
                        }
                    }
                    if result.status == NodeStatus::Failed {
                        if let Some(final_attempt) = result.attempt_events.last() {
                            crate::append_event(
                                &mut run_log,
                                serde_json::json!({
                                    "event": "node_retry_exhausted",
                                    "ts": final_attempt.finished_unix_ms,
                                    "node_id": node_id,
                                    "attempt": final_attempt.attempt,
                                    "status": crate::status_string(&final_attempt.status),
                                    "failure_code": final_attempt
                                        .failure
                                        .as_ref()
                                        .map(|failure| failure.code.clone()),
                                    "retry_reason": final_attempt
                                        .retry_decision
                                        .as_ref()
                                        .map(|decision| decision.reason.clone()),
                                }),
                            )?;
                            run_log_index.push(serde_json::json!({
                                "event": "node_retry_exhausted",
                                "ts": final_attempt.finished_unix_ms,
                                "node_id": node_id,
                                "attempt": final_attempt.attempt,
                                "status": crate::status_string(&final_attempt.status),
                                "failure_code": final_attempt
                                    .failure
                                    .as_ref()
                                    .map(|failure| failure.code.clone()),
                                "retry_reason": final_attempt
                                    .retry_decision
                                    .as_ref()
                                    .map(|decision| decision.reason.clone()),
                            }));
                        }
                    }
                    crate::write_attempt_events(&ctx, &node_id, &result.attempt_events)?;
                    let branch_decision = match resolve_branch_decision(&ctx, &node) {
                        Ok(Some(selection)) => {
                            engine_record::append_indexed_event(
                                &mut run_log,
                                &mut run_log_index,
                                serde_json::json!({
                                            "event": "branch_decision_selected",
                                            "ts": ctx.clock.now_unix_ms(),
                                            "node_id": node_id,
                                            "decision": selection.decision,
                                    "used_default": selection.used_default,
                                }),
                            )?;
                            for pruned in branch_nodes_to_skip(graph, &node_id, &selection.decision)
                            {
                                branch_pruned_nodes.insert(pruned.clone());
                                branch_pruned.insert(pruned);
                            }
                            Some(selection.decision)
                        }
                        Ok(None) => None,
                        Err(failure) => {
                            result.status = NodeStatus::Failed;
                            result.failure = Some(failure);
                            None
                        }
                    };
                    sacred_execution::guard_terminal_node_status(&result.status)?;
                    let trace_failure = result.failure.clone();
                    let replay_action = match result.status {
                        NodeStatus::Cached => ReplayNodeAction::Reused,
                        NodeStatus::Skipped => ReplayNodeAction::Skipped,
                        _ => ReplayNodeAction::Reexecuted,
                    };
                    let replay_event = match replay_action {
                        ReplayNodeAction::Reused => "replay_reused",
                        ReplayNodeAction::Reexecuted => "replay_reexecuted",
                        ReplayNodeAction::Skipped => "replay_reused",
                        ReplayNodeAction::Restored => "replay_reused",
                    };
                    run_log_index.push(serde_json::json!({
                        "event": replay_event,
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                    }));
                    let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
                        started_unix_ms,
                        lifecycle_timestamps.get(&node_id),
                        lifecycle_terminal_state(&result.status, result.failure.as_ref(), None),
                        lifecycle_terminal_cause(&result.status, result.failure.as_ref(), None),
                        finished,
                    )?;
                    sacred_execution::run_write_trace(
                        &ctx,
                        graph,
                        &node_id,
                        result.status.clone(),
                        trace_failure,
                        result.output_evidence.clone(),
                        started,
                        finished,
                        result.attempts,
                        cache_proof,
                        &aid,
                        &aver,
                        &aschema,
                        result.container_meta.clone(),
                        adapter_hash,
                        trigger_evaluations.get(&node_id).cloned(),
                        branch_decision,
                        None,
                        Some(
                            if matches!(result.status, NodeStatus::Failed | NodeStatus::Cancelled) {
                                crate::transition_cause_for_failure(result.failure.as_ref())
                            } else {
                                crate::transition_cause_for_status(&result.status)
                            }
                            .to_string(),
                        ),
                        Some(lifecycle_state),
                        lifecycle_transitions,
                        Some(ReplayProvenance {
                            node_action: match replay_action {
                                ReplayNodeAction::Reexecuted => "reexecuted",
                                ReplayNodeAction::Reused => "reused",
                                ReplayNodeAction::Skipped => "skipped",
                                ReplayNodeAction::Restored => "restored",
                            }
                            .to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "node_finished",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "status": crate::status_string(&result.status),
                            "reason": result
                                .failure
                                .as_ref()
                                .map(|failure| crate::failure_propagation_cause(Some(failure))),
                            "failure_code": result
                                .failure
                                .as_ref()
                                .map(|failure| failure.code.clone()),
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": crate::status_string(&result.status),
                        "reason": result
                            .failure
                            .as_ref()
                            .map(|failure| crate::failure_propagation_cause(Some(failure))),
                        "failure_code": result
                            .failure
                            .as_ref()
                            .map(|failure| failure.code.clone()),
                    }));
                    if result.status == NodeStatus::Failed {
                        status_map.insert(node_id.clone(), NodeStatus::Failed);
                        failure_propagation_records.push(failure_propagation_record(
                            &node_id,
                            "failed",
                            crate::failure_propagation_cause(result.failure.as_ref()),
                            Some(crate::failure_mode_name(&options.failure_propagation)),
                            Vec::new(),
                        ));
                        if matches!(
                            options.failure_propagation,
                            crate::FailurePropagationMode::IsolateBranch
                        ) {
                            record_isolated_branch_descendants(
                                graph,
                                &node_id,
                                &mut isolated_branch_failures,
                            );
                        }
                    } else if result.status == NodeStatus::Cancelled {
                        status_map.insert(node_id.clone(), NodeStatus::Cancelled);
                        failure_propagation_records.push(failure_propagation_record(
                            &node_id,
                            "cancelled",
                            "cancel_requested",
                            Some(crate::failure_mode_name(&options.failure_propagation)),
                            Vec::new(),
                        ));
                    } else {
                        status_map.insert(node_id.clone(), result.status.clone());
                        let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                        let aschema = runtime.adapter_schema_for_kind(&node.kind);
                        let node_fp = node_fingerprint_from_ctx(&ctx, &node.id);
                        sacred_execution::run_cache_write(
                            &options,
                            &node,
                            &node_fp,
                            &ctx,
                            Arc::clone(&ctx.fs),
                            &aid,
                            &aver,
                            &aschema,
                        )?;
                    }
                    let output_bytes =
                        match ctx.fs.metadata(&ctx.run_dir.node_outputs_dir(&node_id)) {
                            Ok(meta) => meta.len(),
                            Err(_) => 0,
                        };
                    node_metric_rows.push(NodeMetrics {
                        node_id: node_id.clone(),
                        queue_delay_ms: 0,
                        execution_time_ms: finished.saturating_sub(started),
                        retries: result.attempts.saturating_sub(1),
                        output_bytes,
                        cache_status: crate::status_string(&result.status),
                        effect_usage: node
                            .effects
                            .iter()
                            .map(|e| format!("{e:?}").to_lowercase())
                            .collect(),
                    });
                }
                Err(err) => {
                    sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
                    let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                    let aschema = runtime.adapter_schema_for_kind(&node.kind);
                    status_map.insert(node_id.clone(), NodeStatus::Failed);
                    let cache_proof = cache_proofs.get(&node_id).cloned();
                    let adapter_hash =
                        runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash());
                    let failure = FailureInfo::new(
                        FailureClass::Execution,
                        "Execution",
                        "INTERNAL",
                        err.to_string(),
                        None,
                    );
                    let (lifecycle_state, lifecycle_transitions) = build_lifecycle_trace(
                        started_unix_ms,
                        lifecycle_timestamps.get(&node_id),
                        lifecycle_terminal_state(&NodeStatus::Failed, Some(&failure), None),
                        lifecycle_terminal_cause(&NodeStatus::Failed, Some(&failure), None),
                        finished,
                    )?;
                    sacred_execution::run_write_trace(
                        &ctx,
                        graph,
                        &node_id,
                        NodeStatus::Failed,
                        Some(failure.clone()),
                        Vec::new(),
                        started,
                        finished,
                        1,
                        cache_proof,
                        &aid,
                        &aver,
                        &aschema,
                        None,
                        adapter_hash,
                        trigger_evaluations.get(&node_id).cloned(),
                        None,
                        None,
                        Some(crate::transition_cause_for_failure(Some(&failure)).to_string()),
                        Some(lifecycle_state),
                        lifecycle_transitions,
                        Some(ReplayProvenance {
                            node_action: "reexecuted".to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    failure_propagation_records.push(failure_propagation_record(
                        &node_id,
                        "failed",
                        crate::failure_propagation_cause(Some(&failure)),
                        Some(crate::failure_mode_name(&options.failure_propagation)),
                        Vec::new(),
                    ));
                    if matches!(
                        options.failure_propagation,
                        crate::FailurePropagationMode::IsolateBranch
                    ) {
                        record_isolated_branch_descendants(
                            graph,
                            &node_id,
                            &mut isolated_branch_failures,
                        );
                    }
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "node_finished",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "status": "failed",
                            "reason": crate::failure_propagation_cause(Some(&failure)),
                            "failure_code": failure.code,
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": "failed",
                        "reason": crate::failure_propagation_cause(Some(&failure)),
                        "failure_code": failure.code,
                    }));
                    node_metric_rows.push(NodeMetrics {
                        node_id: node_id.clone(),
                        queue_delay_ms: 0,
                        execution_time_ms: finished.saturating_sub(started),
                        retries: 0,
                        output_bytes: 0,
                        cache_status: if failure.code == "EXEC_CANCELLED" {
                            "cancelled".to_string()
                        } else {
                            "failed".to_string()
                        },
                        effect_usage: node
                            .effects
                            .iter()
                            .map(|e| format!("{e:?}").to_lowercase())
                            .collect(),
                    });
                }
            }
        }

        let mut completed_node_ids = batch.clone();
        let mut branch_pruned_ids = branch_pruned.into_iter().collect::<Vec<_>>();
        branch_pruned_ids.sort();
        for node_id in &branch_pruned_ids {
            record_skipped_node(
                runtime,
                &ctx,
                graph,
                &mut run_log,
                &mut run_log_index,
                &mut failure_propagation_records,
                &mut status_map,
                &options,
                node_id,
                "branch_decision_not_selected",
                &[],
                started_unix_ms,
                &lifecycle_timestamps,
            )?;
            completed_node_ids.push(node_id.clone());
        }

        for node_id in completed_node_ids {
            for newly_ready in dependency_counter.mark_completed(&node_id) {
                if status_map.contains_key(&newly_ready) {
                    continue;
                }
                scheduler_hook.on_node_eligible(&newly_ready);
                let ready_unix_ms = ctx.clock.now_unix_ms();
                if let Some(timestamps) = lifecycle_timestamps.get_mut(&newly_ready) {
                    remember_first_timestamp(&mut timestamps.eligible_unix_ms, ready_unix_ms);
                }
                engine_record::append_indexed_event(
                    &mut run_log,
                    &mut run_log_index,
                    serde_json::json!({
                        "event": "node_ready",
                        "ts": ready_unix_ms,
                        "node_id": newly_ready,
                        "reason": {
                            "code": "dependencies_satisfied",
                            "upstreams": upstream_nodes(&dep_map, &newly_ready),
                            "trigger_rule": trigger_rule_value(graph, &newly_ready),
                            "released_by": node_id,
                        },
                    }),
                )?;
                ready_queue.insert(newly_ready);
            }
        }

        if matches!(options.failure_propagation, crate::FailurePropagationMode::FailFast)
            && status_map.values().any(|s| *s == NodeStatus::Failed)
        {
            fail_fast_aborted = true;
            break;
        }
    }

    if run_timed_out {
        write_incomplete_run_marker(
            ctx.run_dir.staging_path(),
            "run timed out; partial outputs require resume or repair before completion",
        )
        .map_err(|err| {
            RuntimeError::Executor(format!("incomplete run marker write failed: {err}"))
        })?;
        for node in &graph.nodes {
            if status_map.contains_key(&node.id) {
                continue;
            }
            let failure = FailureInfo::new(
                FailureClass::Timeout,
                "Timeout",
                "RUN_TIMEOUT",
                "run timeout exceeded before node completion",
                options.run_timeout_ms.map(|limit| serde_json::json!({ "run_timeout_ms": limit })),
            );
            let transition_cause = crate::transition_cause_for_failure(Some(&failure));
            record_failed_node(
                runtime,
                &ctx,
                graph,
                &mut run_log,
                &mut run_log_index,
                &mut failure_propagation_records,
                &mut status_map,
                &options,
                node,
                failure,
                transition_cause,
                started_unix_ms,
                &lifecycle_timestamps,
                None,
            )?;
        }
    }

    if fail_fast_aborted {
        for node in &graph.nodes {
            if status_map.contains_key(&node.id) {
                continue;
            }
            let failure = FailureInfo::new(
                FailureClass::Execution,
                "Execution",
                "RUN_ABORTED",
                "run aborted after fail-fast trigger",
                None,
            );
            let transition_cause = crate::transition_cause_for_failure(Some(&failure));
            record_failed_node(
                runtime,
                &ctx,
                graph,
                &mut run_log,
                &mut run_log_index,
                &mut failure_propagation_records,
                &mut status_map,
                &options,
                node,
                failure,
                transition_cause,
                started_unix_ms,
                &lifecycle_timestamps,
                None,
            )?;
        }
    }

    if cancel.load(Ordering::SeqCst) {
        let stop_request = stop_request_watcher.observed_request();
        if let Some(request) = stop_request.as_ref() {
            engine_record::append_indexed_event(
                &mut run_log,
                &mut run_log_index,
                serde_json::json!({
                    "event": "run_cancel_requested",
                    "ts": request.requested_unix_ms,
                    "run_id": manifest.run_id.clone(),
                    "source": request.source.clone(),
                }),
            )?;
        }
        run_audit_events.push(serde_json::json!({
            "action": "cancel",
            "ts": stop_request
                .as_ref()
                .map(|request| request.requested_unix_ms)
                .unwrap_or_else(|| ctx.clock.now_unix_ms()),
            "run_id": manifest.run_id.clone(),
            "source": stop_request
                .as_ref()
                .map(|request| request.source.clone())
                .unwrap_or_else(|| "signal".to_string()),
            "reason": stop_request.as_ref().and_then(|request| request.reason.clone()),
        }));
        manifest.run_cancellation_cause = Some(if stop_request.is_some() {
            "operator_request".to_string()
        } else {
            "operator_interrupt".to_string()
        });
        for node in &graph.nodes {
            if !status_map.contains_key(&node.id) {
                record_skipped_node(
                    runtime,
                    &ctx,
                    graph,
                    &mut run_log,
                    &mut run_log_index,
                    &mut failure_propagation_records,
                    &mut status_map,
                    &options,
                    &node.id,
                    "cancelled",
                    &[],
                    started_unix_ms,
                    &lifecycle_timestamps,
                )?;
            }
        }
    }

    let finished_unix_ms = ctx.clock.now_unix_ms();
    let memory_before_materialization = current_process_memory_bytes().unwrap_or(0);
    if cancel.load(Ordering::SeqCst) {
        manifest.status = "cancelled".to_string();
    } else if run_timed_out {
        manifest.status = "timed_out".to_string();
    } else if status_map.values().any(|s| *s == NodeStatus::Failed) {
        manifest.status = "failed".to_string();
    }
    manifest.finished_unix_ms = finished_unix_ms;
    manifest.node_counts = sacred_execution::count_terminal_nodes(&status_map);
    let trace_statuses: Vec<NodeStatus> = status_map.values().cloned().collect();
    let invariant_counts = crate::invariants::RunNodeCounts {
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
        cancelled: manifest.node_counts.cancelled,
    };
    if !crate::invariants::run_summary_invariant_ok(invariant_counts, &trace_statuses) {
        return Err(RuntimeError::Executor(
            "run summary invariant violated: manifest totals do not match trace totals".to_string(),
        ));
    }
    manifest.run_summary = Some(engine_finalize::summarize_counts(&manifest.node_counts));
    manifest.outputs = collect_outputs_summary(ctx.fs.as_ref(), &ctx.run_dir)?;
    let memory_after_materialization = current_process_memory_bytes().unwrap_or(0);
    let run_index = build_run_outputs_index(&ctx.run_dir, &manifest.outputs)?;
    let lineage_edges = manifest
        .outputs
        .iter()
        .map(|out| bijux_dag_artifacts::lineage::ArtifactLineageEdge {
            artifact_id: format!("{}:{}", out.node_id, out.name),
            producer_node_id: out.node_id.clone(),
            upstream_artifact_ids: dep_map
                .get(&out.node_id)
                .map(|deps| deps.iter().map(|d| format!("{d}:*")).collect())
                .unwrap_or_default(),
        })
        .collect();
    let lineage_snapshot = bijux_dag_artifacts::lineage::ArtifactLineageSnapshot {
        schema_version: "v0.1".to_string(),
        edges: lineage_edges,
    };
    let lineage_snapshot_path = ctx.run_dir.staging_path().join("lineage.snapshot.json");
    if let Some(parent) = lineage_snapshot_path.parent() {
        ctx.fs.create_dir_all(parent)?;
    }
    let lineage_snapshot_payload = bijux_dag_artifacts::lineage::serialize_lineage_snapshot(
        &lineage_snapshot,
    )
    .map_err(|err| RuntimeError::Executor(format!("lineage snapshot write failed: {err}")))?;
    ctx.fs
        .write(&lineage_snapshot_path, &lineage_snapshot_payload)
        .map_err(|err| RuntimeError::Executor(format!("lineage snapshot write failed: {err}")))?;
    let lineage_visualization_path =
        ctx.run_dir.staging_path().join("observability.lineage-visualization.json");
    if let Some(parent) = lineage_visualization_path.parent() {
        ctx.fs.create_dir_all(parent)?;
    }
    let lineage_visualization =
        bijux_dag_artifacts::lineage::build_lineage_visualization(&lineage_snapshot);
    let lineage_visualization_payload =
        bijux_dag_artifacts::lineage::serialize_lineage_visualization(&lineage_visualization)
            .map_err(|err| {
                RuntimeError::Executor(format!("lineage visualization write failed: {err}"))
            })?;
    ctx.fs.write(&lineage_visualization_path, &lineage_visualization_payload).map_err(|err| {
        RuntimeError::Executor(format!("lineage visualization write failed: {err}"))
    })?;
    write_run_outputs_index(ctx.run_dir.staging_path().join("outputs"), &run_index)?;
    run_dir.write_manifest(&manifest)?;
    crate::append_event(
        &mut run_log,
        serde_json::json!({
            "event": "run_finished",
            "ts": finished_unix_ms,
            "status": manifest.status,
        }),
    )?;
    run_log_index.push(serde_json::json!({
        "event": "run_finished",
        "ts": finished_unix_ms,
        "status": manifest.status,
    }));
    run_audit_events.push(serde_json::json!({
        "action": "finish",
        "ts": finished_unix_ms,
        "run_id": manifest.run_id.clone(),
        "status": manifest.status.clone(),
    }));
    let mut structured_events: Vec<EventRecord> = Vec::new();
    for entry in &run_log_index {
        let name = entry.get("event").and_then(|v| v.as_str()).unwrap_or("unknown");
        let unix_ms = entry.get("ts").and_then(|v| v.as_u64()).unwrap_or(0) as u128;
        let node_id = entry.get("node_id").and_then(|v| v.as_str()).map(ToString::to_string);
        let details = entry.clone();
        structured_events.push(EventRecord {
            category: category_from_runtime_event_name(name),
            name: name.to_string(),
            unix_ms,
            node_id,
            run_id: Some(manifest.run_id.clone()),
            details,
        });
    }
    let structured_events = canonicalize_event_records(&structured_events);
    let cache_hits = engine_metrics::count_cache_hits(&status_map);
    let run_metrics = engine_metrics::build_run_metrics(
        &manifest.node_counts,
        graph.nodes.len(),
        &options,
        finished_unix_ms,
        started_unix_ms,
        cache_hits,
        manifest.outputs.len(),
    );
    let scheduler_metrics = engine_metrics::build_scheduler_metrics(
        &manifest.node_counts,
        &run_log_index,
        &options,
        &failure_propagation_records,
    );
    for row in node_metric_rows {
        metrics_registry.record_node(row);
    }
    metrics_registry.record_run(run_metrics);
    metrics_registry.record_scheduler(scheduler_metrics);
    let timeline = reconstruct_timeline_from_events(&structured_events);
    let timeline_path = ctx.run_dir.staging_path().join("observability.timeline.json");
    if let Some(parent) = timeline_path.parent() {
        ctx.fs.create_dir_all(parent)?;
    }
    let timeline_payload = serialize_timeline_export(&timeline)
        .map_err(|err| RuntimeError::Executor(format!("timeline export write failed: {err}")))?;
    ctx.fs
        .write(&timeline_path, &timeline_payload)
        .map_err(|err| RuntimeError::Executor(format!("timeline export write failed: {err}")))?;
    let root_causes =
        build_run_failure_summary(ctx.fs.as_ref(), ctx.run_dir.staging_path(), graph)?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.root-causes.json"),
        &serde_json::to_vec_pretty(&root_causes)?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.events.json"),
        &serde_json::to_vec_pretty(&structured_events)?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.metrics.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "node": metrics_registry.node_metrics,
            "run": metrics_registry.run_metrics,
            "scheduler": metrics_registry.scheduler_metrics,
            "memory": {
                "before_materialization_bytes": memory_before_materialization,
                "after_materialization_bytes": memory_after_materialization
            }
        }))?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.graph-visualization.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({
            "nodes": graph.nodes.iter().map(|n| {
                serde_json::json!({
                    "id": n.id,
                    "status": status_map.get(&n.id).map(crate::status_string).unwrap_or_else(|| "unknown".to_string()),
                })
            }).collect::<Vec<_>>(),
            "edges": graph.edges.iter().map(|e| {
                serde_json::json!({"from": e.from.node_id, "to": e.to.node_id})
            }).collect::<Vec<_>>(),
            "lineage_snapshot": "lineage.snapshot.json",
            "timeline": "observability.timeline.json"
        }))?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("run-log.index.json"),
        &serde_json::to_vec_pretty(
            &structured_events.iter().map(|event| event.details.clone()).collect::<Vec<_>>(),
        )?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("run.audit.json"),
        &serde_json::to_vec_pretty(&run_audit_events)?,
    )?;
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("failure-propagation.json"),
        &serde_json::to_vec_pretty(&failure_propagation_records)?,
    )?;
    write_run_schema_index(
        ctx.run_dir.staging_path().join("run.schema.json"),
        &RunDirSchemaIndex::default(),
    )
    .map_err(|err| RuntimeError::Executor(format!("run schema index write failed: {err}")))?;
    let finalization_mode =
        if run_timed_out { RunFinalizationMode::Incomplete } else { RunFinalizationMode::Complete };
    finalize_run_manifest_with_mode(ctx.run_dir.staging_path(), finalization_mode).map_err(
        |err| RuntimeError::Executor(format!("run finalization marker write failed: {err}")),
    )?;

    let final_path = run_dir.finalize()?;
    if let Some(latest) = options.latest_symlink {
        match runtime.fs.remove_file(&latest) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(RuntimeError::Io(err)),
        }
        runtime.fs.symlink(&final_path, &latest)?;
    }
    Ok(final_path)
}
