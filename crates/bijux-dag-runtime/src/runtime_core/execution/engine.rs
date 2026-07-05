use crate::{
    bind_path_variables_in_value, build_run_outputs_index, cache_dir_from_env, cache_mode_string,
    category_from_runtime_event_name, collect_outputs_summary, current_process_memory_bytes,
    node_fingerprint_from_ctx, node_fingerprint_with_inputs, registered_adapters, sacred_execution,
    set_node_fingerprint, summarize_failure_root_causes, write_timeline_export, CacheProof,
    EffectSet, EventRecord, ExecutionCheckpoint, InMemoryMetricsRegistry, MetricsRegistry,
    NodeMetrics, NodePathBindings, NodeResult, NodeStatus, ReplayNodeAction, RunAttempt,
    RunContext, RunId, RunSnapshot, Runtime, RuntimeConfig, RuntimeError, SchedulerEventHook,
    TimelineEntry, TimelineExport,
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
    finalize_run_manifest, write_incomplete_run_marker, write_provenance, write_run_outputs_index,
    write_run_schema_index, FailureInfo, Manifest, NodeCounts, Provenance, ReplayProvenance,
    RunDir, RunDirLayout, RunDirSchemaIndex, RunMetadata, TriggerEvaluation, TriggerParentStatus,
};
use bijux_dag_core::{
    evaluate_trigger_rule, Effect, Graph, Node, NodeKind, SemanticNodeKind, TriggerRule,
    UpstreamTerminalOutcome, SPEC_VERSION,
};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::{atomic::Ordering, Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct BranchResolution {
    decision: String,
    used_default: bool,
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
        return Err(FailureInfo {
            kind: "Execution".to_string(),
            code: "BRANCH_OUTPUT_MISSING".to_string(),
            message: format!(
                "branch node {} is missing declared decision output {}",
                node.id, branch.decision_output
            ),
            details: Some(serde_json::json!({
                "node_id": node.id,
                "decision_output": branch.decision_output,
            })),
        });
    };
    let output_path = ctx.run_dir.node_outputs_dir(&node.id).join(&output.path);
    let raw = ctx.fs.read_to_string(&output_path).map_err(|_| FailureInfo {
        kind: "Execution".to_string(),
        code: "BRANCH_DECISION_UNREADABLE".to_string(),
        message: format!("branch decision output for {} could not be read", node.id),
        details: Some(serde_json::json!({
            "node_id": node.id,
            "path": output.path,
        })),
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
    Err(FailureInfo {
        kind: "Execution".to_string(),
        code: "INVALID_BRANCH_DECISION".to_string(),
        message: format!("branch node {} produced undeclared decision {}", node.id, decision),
        details: Some(serde_json::json!({
            "node_id": node.id,
            "produced_decision": decision,
            "declared_decisions": branch.decisions,
        })),
    })
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
        && (!options.selectors.include.is_empty() || !options.downstream_selection_roots.is_empty())
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
    if !options.downstream_selection_roots.is_empty() {
        selected.extend(
            crate::compute_downstream_run_closure(graph, &options.downstream_selection_roots)
                .into_iter(),
        );
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

fn upstream_terminal_outcome(status: &NodeStatus) -> UpstreamTerminalOutcome {
    match status {
        NodeStatus::Success => UpstreamTerminalOutcome::Success,
        NodeStatus::Cached => UpstreamTerminalOutcome::Cached,
        NodeStatus::Failed => UpstreamTerminalOutcome::Failed,
        NodeStatus::Skipped => UpstreamTerminalOutcome::Skipped,
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
    FailureInfo {
        kind: "Dependency".to_string(),
        code: "UPSTREAM_FAILED".to_string(),
        message: format!(
            "upstream dependencies did not satisfy trigger rule {:?}",
            node.trigger_rule
        ),
        details: Some(serde_json::json!({
            "parent_statuses": trigger_evaluation.parent_statuses,
            "trigger_rule": trigger_evaluation.trigger_rule,
            "reason": trigger_evaluation.reason,
        })),
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
    failure_propagation_records: &mut Vec<serde_json::Value>,
    status_map: &mut HashMap<String, NodeStatus>,
    options: &RuntimeConfig,
    node_id: &str,
    reason: &str,
) -> Result<(), RuntimeError> {
    if status_map.contains_key(node_id) {
        return Ok(());
    }
    sacred_execution::guard_terminal_node_status(&NodeStatus::Skipped)?;
    status_map.insert(node_id.to_string(), NodeStatus::Skipped);
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
    sacred_execution::run_write_trace(
        ctx,
        graph,
        node_id,
        NodeStatus::Skipped,
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
        Some(bijux_dag_artifacts::SkipReason { reason: reason.to_string() }),
        Some(crate::transition_cause_for_skip_reason(reason).to_string()),
        Some(ReplayProvenance {
            node_action: "skipped".to_string(),
            source_run_id: options.parent_run_id.clone(),
        }),
    )?;
    failure_propagation_records.push(serde_json::json!({
        "node_id": node_id,
        "status": "skipped",
        "cause": crate::transition_cause_for_skip_reason(reason).to_lowercase(),
    }));
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
            "reason": reason,
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
    let run_dir = if let Some(ref run_id) = options.run_id {
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

    let run_id = options.run_id.clone().unwrap_or_else(|| {
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
        node_counts: NodeCounts { success: 0, failed: 0, skipped: 0, cached: 0 },
        policy: bijux_dag_artifacts::PolicyInfo {
            deny_network: options.policy.deny_network,
            deny_env: options.policy.deny_env,
            deny_clock: options.policy.deny_clock,
            clean_env: options.policy.clean_env,
        },
        cache_mode: cache_mode_string(&options.cache_mode),
        cache_dir: effective_cache_dir.as_ref().map(|p| p.display().to_string()),
        run_timeout_ms: options.run_timeout_ms,
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
        },
        time_source: "system_clock".to_string(),
    };
    write_provenance(run_dir.provenance_path(), &prov)?;

    let cancel = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let cancel_flag = cancel.clone();
    let _ = ctrlc::set_handler(move || {
        cancel_flag.store(true, Ordering::SeqCst);
    });

    let run_dir_arc = Arc::new(run_dir.clone());
    let store = crate::store::ArtifactStore::new(Arc::clone(&run_dir_arc), Arc::clone(&runtime.fs));
    let mut run_log = store.open_run_log()?;
    let mut run_log_index: Vec<serde_json::Value> = Vec::new();
    let mut run_audit_events: Vec<serde_json::Value> = Vec::new();
    let mut failure_propagation_records: Vec<serde_json::Value> = Vec::new();
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
    let mut base_fps = HashMap::new();
    for node in &graph.nodes {
        let params = resolved.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        let base_fp = graph.node_fingerprint_with_params(node, &params)?;
        let env_allowlist = crate::effective_env_allowlist(node);
        let declared_env = crate::declared_environment(
            &ambient_env,
            options.policy.clean_env,
            &env_allowlist,
            &[],
        );
        let env_fp = crate::sha256_bytes(&serde_json::to_vec(&declared_env)?);
        base_fps
            .insert(node.id.clone(), crate::sha256_bytes(format!("{base_fp}:{env_fp}").as_bytes()));
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
        graph_fingerprint: Arc::clone(&graph_fingerprint),
        planner_contract_version: plan.planner_contract_version.clone(),
        execution_fingerprint: plan.execution_fingerprint.clone(),
        evidence_fingerprint: plan.evidence_fingerprint.clone(),
        resolved_params,
        effective_cache_dir: effective_cache_dir.clone(),
        fs: Arc::clone(&runtime.fs),
        clock: Arc::clone(&runtime.clock),
        store,
        policy: options.policy.clone(),
        absolute_path_policy: options.absolute_path_policy,
    };
    let requested_selectors = options
        .selectors
        .include
        .iter()
        .map(|selector| crate::requested_selector_label("include", selector))
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
    let run_attempt = RunAttempt {
        attempt_index: 1,
        run_id: RunId::parse(&manifest.run_id).unwrap_or_else(|_| RunId(manifest.run_id.clone())),
        parent_run_id: options.parent_run_id.as_deref().and_then(|v| RunId::parse(v).ok()),
        reason: if options.parent_run_id.is_some() {
            "replay_or_retry".to_string()
        } else {
            "initial_submission".to_string()
        },
    };
    let run_attempts_path = ctx.run_dir.staging_path().join("run.attempts.json");
    ctx.fs.write(&run_attempts_path, &serde_json::to_vec_pretty(&vec![run_attempt])?)?;
    let start = Instant::now();
    let mut status_map: HashMap<String, NodeStatus> = HashMap::new();
    let mut cache_proofs: HashMap<String, CacheProof> = HashMap::new();
    let mut fail_fast_aborted = false;
    let mut run_timed_out = false;
    let dep_map = plan.dep_map.clone();
    let mut dependency_counter = sacred_execution::resolve_dependencies(&plan);
    let mut ready_queue = sacred_execution::ready_queue_from_dependencies(&dependency_counter);
    let initial_ready = ready_queue.snapshot_sorted();
    let mut scheduler = crate::build_scheduler(&options.scheduler_policy);
    let scheduler_hook = crate::NoopSchedulerEventHook;
    let mut loop_index: u64 = 0;
    let mut branch_pruned_nodes = BTreeSet::new();
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
    for node_id in &initial_ready {
        scheduler_hook.on_node_eligible(node_id);
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
        let forced_batch = batch.len() == 1;

        let mut handles = Vec::new();
        let mut skipped: Vec<(String, String)> = Vec::new();
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
                skipped.push((node_id.clone(), "branch_decision_not_selected".to_string()));
                continue;
            }
            if let Some(reason) = plan.filter_reasons.get(node_id) {
                skipped.push((node_id.clone(), reason.clone()));
                continue;
            }
            let node = graph
                .nodes
                .iter()
                .find(|n| n.id == *node_id)
                .ok_or_else(|| RuntimeError::Executor("missing node".to_string()))?
                .clone();
            if cancel.load(Ordering::SeqCst) {
                skipped.push((node_id.clone(), "cancelled".to_string()));
                continue;
            }
            if let Some(limit) = options.run_timeout_ms {
                if start.elapsed() > Duration::from_millis(limit) {
                    run_timed_out = true;
                    preflight_failures.push((
                        node_id.clone(),
                        node,
                        FailureInfo {
                            kind: "Execution".to_string(),
                            code: "RUN_TIMEOUT".to_string(),
                            message: "run timeout exceeded before node start".to_string(),
                            details: Some(serde_json::json!({ "run_timeout_ms": limit })),
                        },
                        "TimeoutExceeded".to_string(),
                        None,
                    ));
                    continue;
                }
            }

            if let Some(deps) = dep_map.get(node_id) {
                let dependencies = deps.iter().cloned().collect::<Vec<_>>();
                let parent_statuses = dependencies
                    .iter()
                    .filter_map(|dependency| status_map.get(dependency).cloned())
                    .collect::<Vec<_>>();
                if parent_statuses.len() == dependencies.len() {
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
            }
            let resolved_params = ctx.resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);

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
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "network effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "network" })),
                    },
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
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "env effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "env" })),
                    },
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
                    FailureInfo {
                        kind: "Policy".to_string(),
                        code: "POLICY_DENIED".to_string(),
                        message: "clock effect denied by policy".to_string(),
                        details: Some(serde_json::json!({ "effect": "clock" })),
                    },
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
                node_id,
                options.materialize_inputs,
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
        for (node_id, reason) in &skipped {
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
                Some(ReplayProvenance {
                    node_action: "reexecuted".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node_id,
                "status": "failed",
                "cause": crate::failure_propagation_cause(Some(failure)),
            }));
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_finished",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "status": "failed",
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_finished",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "status": "failed",
            }));
        }

        let mut started_ids: Vec<String> = Vec::new();
        for (node_id, _, _) in &to_start {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _) in &cached {
            started_ids.push(node_id.clone());
        }
        for (node_id, _, _, _, _) in &preflight_failures {
            started_ids.push(node_id.clone());
        }
        started_ids.sort();
        let schedule_reason = if forced_batch { "ready" } else { "budget_available" };
        for node_id in &started_ids {
            scheduler_hook.on_node_scheduled(node_id);
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_scheduled",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                    "reason": schedule_reason,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_scheduled",
                "ts": ctx.clock.now_unix_ms(),
                "node_id": node_id,
                "reason": schedule_reason,
            }));
        }

        let checkpoint = ExecutionCheckpoint {
            loop_index,
            ready_queue_depth: ready_queue.len(),
            ready_queue: ready_queue.snapshot_sorted(),
            inflight: started_ids.clone(),
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
        for node_id in &started_ids {
            crate::append_event(
                &mut run_log,
                serde_json::json!({
                    "event": "node_started",
                    "ts": ctx.clock.now_unix_ms(),
                    "node_id": node_id,
                }),
            )?;
            run_log_index.push(serde_json::json!({
                "event": "node_started",
                "ts": ctx.clock.now_unix_ms(),
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
                }),
            )?;
            if status == NodeStatus::Failed {
                failure_propagation_records.push(serde_json::json!({
                    "node_id": node_id,
                    "status": "failed",
                    "cause": crate::failure_propagation_cause(failure.as_ref()),
                }));
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

        for (node_id, node, params) in &to_start {
            let adapter = runtime.adapter_for_kind(&node.kind)?;
            let ctx_clone = RunContext {
                run_dir: Arc::clone(&ctx.run_dir),
                graph_fingerprint: ctx.graph_fingerprint.clone(),
                planner_contract_version: ctx.planner_contract_version.clone(),
                execution_fingerprint: ctx.execution_fingerprint.clone(),
                evidence_fingerprint: ctx.evidence_fingerprint.clone(),
                resolved_params: ctx.resolved_params.clone(),
                effective_cache_dir: ctx.effective_cache_dir.clone(),
                fs: Arc::clone(&ctx.fs),
                clock: Arc::clone(&ctx.clock),
                store: ctx.store.clone(),
                policy: ctx.policy.clone(),
                absolute_path_policy: ctx.absolute_path_policy,
            };
            let node_id_clone = node_id.clone();
            let node_for_thread = node.clone();
            let params_for_thread = params.clone();
            let graph_for_thread = graph.clone();
            let retry = node.retry.clone();
            handles.push((
                node_id_clone,
                node.clone(),
                std::thread::spawn(move || {
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
                    (started, finished, result)
                }),
            ));
        }

        type ResultItem = (String, Node, u128, u128, Result<NodeResult, RuntimeError>);
        let mut results: Vec<ResultItem> = Vec::new();
        for (node_id, node, handle) in handles {
            let res = handle.join().unwrap_or_else(|_| {
                (
                    ctx.clock.now_unix_ms(),
                    ctx.clock.now_unix_ms(),
                    Err(RuntimeError::Executor("thread panicked".to_string())),
                )
            });
            results.push((node_id, node, res.0, res.1, res.2));
        }
        results.sort_by(|a, b| a.0.cmp(&b.0));
        for (node_id, node, started, finished, res) in results {
            match res {
                Ok(mut result) => {
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
                            if result.status == NodeStatus::Failed {
                                crate::transition_cause_for_failure(result.failure.as_ref())
                            } else {
                                crate::transition_cause_for_status(&result.status)
                            }
                            .to_string(),
                        ),
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
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": crate::status_string(&result.status),
                    }));
                    if result.status == NodeStatus::Failed {
                        status_map.insert(node_id.clone(), NodeStatus::Failed);
                        failure_propagation_records.push(serde_json::json!({
                            "node_id": node_id,
                            "status": "failed",
                            "cause": crate::failure_propagation_cause(result.failure.as_ref()),
                        }));
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
                    let failure = FailureInfo {
                        kind: "Internal".to_string(),
                        code: "INTERNAL".to_string(),
                        message: err.to_string(),
                        details: None,
                    };
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
                        Some(ReplayProvenance {
                            node_action: "reexecuted".to_string(),
                            source_run_id: options.parent_run_id.clone(),
                        }),
                    )?;
                    failure_propagation_records.push(serde_json::json!({
                        "node_id": node_id,
                        "status": "failed",
                        "cause": crate::failure_propagation_cause(Some(&failure)),
                    }));
                    crate::append_event(
                        &mut run_log,
                        serde_json::json!({
                            "event": "node_finished",
                            "ts": ctx.clock.now_unix_ms(),
                            "node_id": node_id,
                            "status": "failed",
                        }),
                    )?;
                    run_log_index.push(serde_json::json!({
                        "event": "node_finished",
                        "ts": ctx.clock.now_unix_ms(),
                        "node_id": node_id,
                        "status": "failed",
                    }));
                    node_metric_rows.push(NodeMetrics {
                        node_id: node_id.clone(),
                        queue_delay_ms: 0,
                        execution_time_ms: finished.saturating_sub(started),
                        retries: 0,
                        output_bytes: 0,
                        cache_status: "failed".to_string(),
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
            )?;
            completed_node_ids.push(node_id.clone());
        }

        for node_id in completed_node_ids {
            for newly_ready in dependency_counter.mark_completed(&node_id) {
                if status_map.contains_key(&newly_ready) {
                    continue;
                }
                scheduler_hook.on_node_eligible(&newly_ready);
                engine_record::append_indexed_event(
                    &mut run_log,
                    &mut run_log_index,
                    serde_json::json!({
                        "event": "node_ready",
                        "ts": ctx.clock.now_unix_ms(),
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
        for node in &graph.nodes {
            if status_map.contains_key(&node.id) {
                continue;
            }
            sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
            status_map.insert(node.id.clone(), NodeStatus::Failed);
            let failure = FailureInfo {
                kind: "Execution".to_string(),
                code: "RUN_TIMEOUT".to_string(),
                message: "run timeout exceeded before node completion".to_string(),
                details: options
                    .run_timeout_ms
                    .map(|limit| serde_json::json!({ "run_timeout_ms": limit })),
            };
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let started = ctx.clock.now_unix_ms();
            sacred_execution::run_write_trace(
                &ctx,
                graph,
                &node.id,
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
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash()),
                None,
                None,
                None,
                Some(crate::transition_cause_for_failure(Some(&failure)).to_string()),
                Some(ReplayProvenance {
                    node_action: "skipped".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node.id,
                "status": "failed",
                "cause": crate::failure_propagation_cause(Some(&failure)),
            }));
        }
    }

    if fail_fast_aborted {
        for node in &graph.nodes {
            if status_map.contains_key(&node.id) {
                continue;
            }
            sacred_execution::guard_terminal_node_status(&NodeStatus::Failed)?;
            status_map.insert(node.id.clone(), NodeStatus::Failed);
            let failure = FailureInfo {
                kind: "Execution".to_string(),
                code: "RUN_ABORTED".to_string(),
                message: "run aborted after fail-fast trigger".to_string(),
                details: None,
            };
            let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
            let aschema = runtime.adapter_schema_for_kind(&node.kind);
            let started = ctx.clock.now_unix_ms();
            sacred_execution::run_write_trace(
                &ctx,
                graph,
                &node.id,
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
                runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash()),
                None,
                None,
                None,
                Some(crate::transition_cause_for_failure(Some(&failure)).to_string()),
                Some(ReplayProvenance {
                    node_action: "skipped".to_string(),
                    source_run_id: options.parent_run_id.clone(),
                }),
            )?;
            failure_propagation_records.push(serde_json::json!({
                "node_id": node.id,
                "status": "failed",
                "cause": crate::failure_propagation_cause(Some(&failure)),
            }));
        }
    }

    if cancel.load(Ordering::SeqCst) {
        run_audit_events.push(serde_json::json!({
            "action": "cancel",
            "ts": ctx.clock.now_unix_ms(),
            "run_id": manifest.run_id.clone(),
        }));
        for node in &graph.nodes {
            if !status_map.contains_key(&node.id) {
                status_map.insert(node.id.clone(), NodeStatus::Skipped);
                let (aid, aver) = runtime.adapter_meta_for_kind(&node.kind);
                let aschema = runtime.adapter_schema_for_kind(&node.kind);
                let started = ctx.clock.now_unix_ms();
                sacred_execution::run_write_trace(
                    &ctx,
                    graph,
                    &node.id,
                    NodeStatus::Skipped,
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
                    runtime.adapter_for_kind(&node.kind).ok().and_then(|a| a.binary_hash()),
                    None,
                    None,
                    Some(bijux_dag_artifacts::SkipReason { reason: "cancelled".to_string() }),
                    Some("CancelRequested".to_string()),
                    Some(ReplayProvenance {
                        node_action: "skipped".to_string(),
                        source_run_id: options.parent_run_id.clone(),
                    }),
                )?;
                failure_propagation_records.push(serde_json::json!({
                    "node_id": node.id,
                    "status": "skipped",
                    "cause": "cancel_requested",
                }));
            }
        }
    }

    let finished_unix_ms = ctx.clock.now_unix_ms();
    let memory_before_materialization = current_process_memory_bytes().unwrap_or(0);
    if cancel.load(Ordering::SeqCst) {
        manifest.status = "cancelled".to_string();
    } else if run_timed_out {
        manifest.status = "failed".to_string();
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
    bijux_dag_artifacts::lineage::write_lineage_snapshot(
        ctx.run_dir.staging_path().join("lineage.snapshot.json"),
        &lineage_snapshot,
    )
    .map_err(|err| RuntimeError::Executor(format!("lineage snapshot write failed: {err}")))?;
    bijux_dag_artifacts::lineage::export_lineage_visualization(
        ctx.run_dir.staging_path().join("observability.lineage-visualization.json"),
        &lineage_snapshot,
    )
    .map_err(|err| RuntimeError::Executor(format!("lineage visualization write failed: {err}")))?;
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
    let timeline = TimelineExport {
        schema_version: "v0.1".to_string(),
        entries: structured_events
            .iter()
            .map(|event| TimelineEntry {
                unix_ms: event.unix_ms,
                category: format!("{:?}", event.category).to_lowercase(),
                label: event.name.clone(),
                node_id: event.node_id.clone(),
            })
            .collect(),
    };
    write_timeline_export(
        ctx.run_dir.staging_path().join("observability.timeline.json"),
        &timeline,
    )
    .map_err(|err| RuntimeError::Executor(format!("timeline export write failed: {err}")))?;
    let root_causes = summarize_failure_root_causes(&structured_events);
    ctx.fs.write(
        &ctx.run_dir.staging_path().join("observability.root-causes.json"),
        &serde_json::to_vec_pretty(&serde_json::json!({ "roots": root_causes }))?,
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
        &serde_json::to_vec_pretty(&run_log_index)?,
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
    finalize_run_manifest(ctx.run_dir.staging_path()).map_err(|err| {
        RuntimeError::Executor(format!("run finalization marker write failed: {err}"))
    })?;

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
