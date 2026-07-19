use crate::execution_plan::ExecutionPlan;
use crate::infrastructure::{
    negotiate_backend_capabilities, BackendCapabilities, BackendCapabilityRequirement,
};
use crate::{
    collect_container_argv_path_usages, collect_container_workdir_usage,
    collect_resolved_path_usages, resolve_container_argv, NodePathBindings, ReadyQueue,
    ResolvedPathUsage, RuntimeConfig, Selector, SelectorSet,
};
use bijux_dag_artifacts::RunDirLayout;
use bijux_dag_core::{resources, Graph, Node};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlannerPhase {
    Normalize,
    Validate,
    Bind,
    Optimize,
    ScheduleReadyTransform,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PlannerNodeAction {
    Execute,
    Restore,
    Verify,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerNodeAnnotation {
    pub node_id: String,
    pub selected: bool,
    pub reason: String,
    pub replay_action: PlannerNodeAction,
    pub locality_hint: Option<String>,
    pub queue_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerExecutionDemand {
    pub cpu_cores_total: u64,
    pub memory_mb_total: u64,
    pub gpu_devices_total: u64,
    pub cpu_cores_peak_parallel: u64,
    pub memory_mb_peak_parallel: u64,
    pub gpu_devices_peak_parallel: u64,
    pub named_resources_total: BTreeMap<String, u64>,
    pub named_resources_peak_parallel: BTreeMap<String, u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerSchedulingBound {
    DependencyBound,
    ResourceBound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerBlockedNodeEstimate {
    pub node_id: String,
    pub blocked_by: Vec<String>,
    pub blocked_waves: usize,
    pub blocked_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerResourceBottleneck {
    pub resource: String,
    pub blocking_events: usize,
    pub blocked_node_ids: Vec<String>,
    pub blocked_duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerSchedulingSimulation {
    pub scheduled_waves: usize,
    pub projected_makespan_ms: u64,
    pub resource_delay_ms: u64,
    pub run_bound: PlannerSchedulingBound,
    pub bottlenecks: Vec<PlannerResourceBottleneck>,
    pub blocked_nodes: Vec<PlannerBlockedNodeEstimate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerCacheExposure {
    pub cacheable_nodes: usize,
    pub non_cacheable_nodes: usize,
    pub non_cacheable_node_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerTimeoutExposure {
    pub timed_nodes: usize,
    pub timed_node_ids: Vec<String>,
    pub max_timeout_ms: Option<u64>,
    pub total_timeout_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerRetryExposure {
    pub retrying_nodes: usize,
    pub retrying_node_ids: Vec<String>,
    pub max_attempts: u32,
    pub max_backoff_ms: u64,
    pub total_retry_attempts: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerDurationSource {
    EstimatedDuration,
    UnitFallback,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerCriticalPathNode {
    pub node_id: String,
    pub duration_ms: u64,
    pub duration_source: PlannerDurationSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerCriticalPathEstimate {
    pub node_ids: Vec<String>,
    pub total_duration_ms: u64,
    pub estimated_duration_nodes: usize,
    pub unit_duration_fallback_nodes: usize,
    pub nodes: Vec<PlannerCriticalPathNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerExecutionCostEstimate {
    pub node_count: usize,
    pub root_nodes: Vec<String>,
    pub critical_path_length: usize,
    pub critical_path: PlannerCriticalPathEstimate,
    pub max_parallelism: usize,
    pub demand: PlannerExecutionDemand,
    pub scheduling_simulation: PlannerSchedulingSimulation,
    pub cache_exposure: PlannerCacheExposure,
    pub timeout_exposure: PlannerTimeoutExposure,
    pub retry_exposure: PlannerRetryExposure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerPriorityInheritance {
    pub node_id: String,
    pub inherited_priority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerBackfillPlan {
    pub window_start_unix_ms: u128,
    pub window_end_unix_ms: u128,
    pub partition_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerExplainReport {
    pub plan_fingerprint: String,
    pub phases: Vec<PlannerPhase>,
    pub annotations: Vec<PlannerNodeAnnotation>,
    pub optimization_notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerNodePathPreview {
    pub node_id: String,
    pub execution_surface: String,
    pub variable_bindings: NodePathBindings,
    pub resolved_paths: Vec<ResolvedPathUsage>,
    pub resolved_argv: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerPlanDiff {
    pub graph_fingerprint_changed: bool,
    pub execution_fingerprint_changed: bool,
    pub metadata_only_changed: bool,
    pub execution_affecting_changed: bool,
    pub added_nodes: Vec<String>,
    pub removed_nodes: Vec<String>,
    pub changed_params: Vec<String>,
    pub changed_outputs: Vec<String>,
    pub changed_resources: Vec<String>,
    pub changed_retry_timeout: Vec<String>,
    pub changed_effects: Vec<String>,
    pub changed_cache: Vec<String>,
    pub changed_env_allowlist: Vec<String>,
    pub changed_trigger_rule: Vec<String>,
    pub changed_branching: Vec<String>,
    pub changed_node_kind: Vec<String>,
    pub added_dependencies: Vec<String>,
    pub removed_dependencies: Vec<String>,
    pub changed_metadata: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PlannerEquivalenceClass {
    StrictEquivalent,
    MetadataDriftEquivalent,
    NotEquivalent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerEquivalenceReport {
    pub equivalent: bool,
    pub equivalence_class: PlannerEquivalenceClass,
    pub before_graph_identity: String,
    pub after_graph_identity: String,
    pub graph_identity_equal: bool,
    pub before_execution_fingerprint: String,
    pub after_execution_fingerprint: String,
    pub execution_fingerprint_equal: bool,
    pub ignored_non_execution_drift: Vec<String>,
    pub non_equivalence_causes: Vec<String>,
    pub summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerGuardrails {
    pub allow_semantic_optimizations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannerBuildResult {
    pub plan: ExecutionPlan,
    pub phases: Vec<PlannerPhase>,
    pub annotations: Vec<PlannerNodeAnnotation>,
    pub execution_cost_estimate: PlannerExecutionCostEstimate,
    pub priority_inheritance: Vec<PlannerPriorityInheritance>,
    pub plan_fingerprint: String,
    pub path_previews: Option<Vec<PlannerNodePathPreview>>,
    #[serde(skip, default = "planner_build_result_graph_placeholder")]
    analysis_graph: Graph,
}

pub fn build_planner_analysis(
    graph: &Graph,
    options: &RuntimeConfig,
    selector_set: &SelectorSet,
    guardrails: &PlannerGuardrails,
) -> Result<PlannerBuildResult, String> {
    let phases = vec![
        PlannerPhase::Normalize,
        PlannerPhase::Validate,
        PlannerPhase::Bind,
        PlannerPhase::Optimize,
        PlannerPhase::ScheduleReadyTransform,
    ];
    let normalized_graph = graph.canonicalize();
    validate_backend_compatibility(&normalized_graph)?;
    validate_impossible_run_requirements(&normalized_graph)?;

    let mut plan = crate::planner::build_plan(&normalized_graph, options);
    if plan.diagnostics.iter().any(|d| d.contains("P4013") || d.contains("P4021")) {
        return Err(
            "planner lowering rejected unsupported runtime capability requirements".to_string()
        );
    }
    let resolved_graph = normalized_graph.resolve_graph().map_err(|error| error.to_string())?;
    validate_command_templates(&normalized_graph, &resolved_graph.resolved_params)?;
    let mut annotations = annotate_plan(&normalized_graph, &plan, selector_set);
    plan = apply_optimizer_rules(normalized_graph.clone(), plan, &mut annotations, guardrails);
    let execution_cost_estimate = estimate_execution_cost(&normalized_graph, &plan, options);
    let priority_inheritance = inherit_priority(&plan.nodes);
    let plan_fingerprint = fingerprint_plan(&plan, &annotations)?;
    let path_previews =
        build_path_previews(&normalized_graph, options, &resolved_graph.resolved_params)?;

    Ok(PlannerBuildResult {
        plan,
        phases,
        annotations,
        execution_cost_estimate,
        priority_inheritance,
        plan_fingerprint,
        path_previews,
        analysis_graph: normalized_graph,
    })
}

fn planner_build_result_graph_placeholder() -> Graph {
    Graph {
        spec: String::new(),
        meta: None,
        inputs: BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes: Vec::new(),
        edges: Vec::new(),
    }
}

fn apply_optimizer_rules(
    graph: Graph,
    mut plan: ExecutionPlan,
    annotations: &mut [PlannerNodeAnnotation],
    guardrails: &PlannerGuardrails,
) -> ExecutionPlan {
    if !guardrails.allow_semantic_optimizations {
        return plan;
    }
    let mut no_op_nodes = BTreeSet::new();
    for node in &graph.nodes {
        if node.tags.iter().any(|t| t == "noop") {
            no_op_nodes.insert(node.id.clone());
        }
    }
    for node_id in no_op_nodes {
        plan.filter_reasons.entry(node_id.clone()).or_insert_with(|| "optimized_noop".to_string());
        if let Some(ann) = annotations.iter_mut().find(|ann| ann.node_id == node_id) {
            ann.selected = false;
            ann.reason = "optimized_noop".to_string();
            ann.replay_action = PlannerNodeAction::Skip;
        }
    }
    plan
}

pub fn fingerprint_plan(
    plan: &ExecutionPlan,
    annotations: &[PlannerNodeAnnotation],
) -> Result<String, String> {
    let payload = serde_json::to_vec(&(plan, annotations)).map_err(|err| err.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(payload);
    Ok(hex::encode(hasher.finalize()))
}

pub fn diff_plans(before: &PlannerBuildResult, after: &PlannerBuildResult) -> PlannerPlanDiff {
    let before_nodes = before
        .analysis_graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();
    let after_nodes = after
        .analysis_graph
        .nodes
        .iter()
        .map(|node| (node.id.clone(), node))
        .collect::<BTreeMap<_, _>>();

    let mut added_nodes = after_nodes
        .keys()
        .filter(|node_id| !before_nodes.contains_key(*node_id))
        .cloned()
        .collect::<Vec<_>>();
    let mut removed_nodes = before_nodes
        .keys()
        .filter(|node_id| !after_nodes.contains_key(*node_id))
        .cloned()
        .collect::<Vec<_>>();
    let mut changed_params = Vec::new();
    let mut changed_outputs = Vec::new();
    let mut changed_resources = Vec::new();
    let mut changed_retry_timeout = Vec::new();
    let mut changed_effects = Vec::new();
    let mut changed_cache = Vec::new();
    let mut changed_env_allowlist = Vec::new();
    let mut changed_trigger_rule = Vec::new();
    let mut changed_branching = Vec::new();
    let mut changed_node_kind = Vec::new();
    let mut changed_metadata = Vec::new();

    for node_id in before_nodes.keys().filter(|node_id| after_nodes.contains_key(*node_id)) {
        let before_node =
            before_nodes.get(node_id).expect("before node must exist for intersection comparison");
        let after_node =
            after_nodes.get(node_id).expect("after node must exist for intersection comparison");

        if !serialized_value_eq(&before_node.params, &after_node.params) {
            changed_params.push(node_id.clone());
        }
        if before_node.outputs != after_node.outputs {
            changed_outputs.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.resources, &after_node.resources) {
            changed_resources.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.retry, &after_node.retry)
            || before_node.timeout_ms != after_node.timeout_ms
        {
            changed_retry_timeout.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.effects, &after_node.effects) {
            changed_effects.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.cache, &after_node.cache) {
            changed_cache.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.env_allowlist, &after_node.env_allowlist) {
            changed_env_allowlist.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.trigger_rule, &after_node.trigger_rule) {
            changed_trigger_rule.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.branch, &after_node.branch) {
            changed_branching.push(node_id.clone());
        }
        if before_node.kind.as_str() != after_node.kind.as_str()
            || !serialized_value_eq(&before_node.semantic_kind, &after_node.semantic_kind)
        {
            changed_node_kind.push(node_id.clone());
        }
        if !serialized_value_eq(&before_node.tags, &after_node.tags) {
            changed_metadata.push(format!("node_tags:{node_id}"));
        }
        if !serialized_value_eq(&before_node.group, &after_node.group) {
            changed_metadata.push(format!("node_group:{node_id}"));
        }
    }

    let before_dependencies =
        before.analysis_graph.edges.iter().map(edge_diff_key).collect::<BTreeSet<_>>();
    let after_dependencies =
        after.analysis_graph.edges.iter().map(edge_diff_key).collect::<BTreeSet<_>>();
    let mut added_dependencies =
        after_dependencies.difference(&before_dependencies).cloned().collect::<Vec<_>>();
    let mut removed_dependencies =
        before_dependencies.difference(&after_dependencies).cloned().collect::<Vec<_>>();

    let graph_fingerprint_changed = before.plan.graph_fingerprint != after.plan.graph_fingerprint;
    let execution_fingerprint_changed =
        before.plan.execution_fingerprint != after.plan.execution_fingerprint;
    if !serialized_value_eq(&before.analysis_graph.meta, &after.analysis_graph.meta) {
        changed_metadata.push("graph_meta".to_string());
    }

    added_nodes.sort();
    removed_nodes.sort();
    changed_params.sort();
    changed_outputs.sort();
    changed_resources.sort();
    changed_retry_timeout.sort();
    changed_effects.sort();
    changed_cache.sort();
    changed_env_allowlist.sort();
    changed_trigger_rule.sort();
    changed_branching.sort();
    changed_node_kind.sort();
    added_dependencies.sort();
    removed_dependencies.sort();
    changed_metadata.sort();
    changed_metadata.dedup();

    let execution_affecting_changed = execution_fingerprint_changed
        || !added_nodes.is_empty()
        || !removed_nodes.is_empty()
        || !changed_params.is_empty()
        || !changed_outputs.is_empty()
        || !changed_resources.is_empty()
        || !changed_retry_timeout.is_empty()
        || !changed_effects.is_empty()
        || !changed_cache.is_empty()
        || !changed_env_allowlist.is_empty()
        || !changed_trigger_rule.is_empty()
        || !changed_branching.is_empty()
        || !changed_node_kind.is_empty()
        || !added_dependencies.is_empty()
        || !removed_dependencies.is_empty();
    let metadata_only_changed = graph_fingerprint_changed && !execution_affecting_changed;

    PlannerPlanDiff {
        graph_fingerprint_changed,
        execution_fingerprint_changed,
        metadata_only_changed,
        execution_affecting_changed,
        added_nodes,
        removed_nodes,
        changed_params,
        changed_outputs,
        changed_resources,
        changed_retry_timeout,
        changed_effects,
        changed_cache,
        changed_env_allowlist,
        changed_trigger_rule,
        changed_branching,
        changed_node_kind,
        added_dependencies,
        removed_dependencies,
        changed_metadata,
    }
}

pub fn compare_plan_equivalence(
    before: &PlannerBuildResult,
    after: &PlannerBuildResult,
) -> PlannerEquivalenceReport {
    let diff = diff_plans(before, after);
    let graph_identity_equal = before.plan.graph_fingerprint == after.plan.graph_fingerprint;
    let execution_fingerprint_equal =
        before.plan.execution_fingerprint == after.plan.execution_fingerprint;
    let execution_equivalent = !diff.execution_affecting_changed;

    let (equivalent, equivalence_class, summary) = if execution_equivalent {
        if graph_identity_equal && execution_fingerprint_equal {
            (
                true,
                PlannerEquivalenceClass::StrictEquivalent,
                "graphs are equivalent under canonical graph identity and execution fingerprint"
                    .to_string(),
            )
        } else {
            (
                true,
                PlannerEquivalenceClass::MetadataDriftEquivalent,
                "graphs remain execution-equivalent after ignoring non-execution metadata drift"
                    .to_string(),
            )
        }
    } else {
        (
            false,
            PlannerEquivalenceClass::NotEquivalent,
            "graphs are not execution-equivalent because execution-affecting planner state changed"
                .to_string(),
        )
    };

    let ignored_non_execution_drift =
        if execution_equivalent { diff.changed_metadata.clone() } else { Vec::new() };
    let non_equivalence_causes =
        if execution_equivalent { Vec::new() } else { equivalence_causes_from_diff(&diff) };

    PlannerEquivalenceReport {
        equivalent,
        equivalence_class,
        before_graph_identity: before.plan.graph_fingerprint.clone(),
        after_graph_identity: after.plan.graph_fingerprint.clone(),
        graph_identity_equal,
        before_execution_fingerprint: before.plan.execution_fingerprint.clone(),
        after_execution_fingerprint: after.plan.execution_fingerprint.clone(),
        execution_fingerprint_equal,
        ignored_non_execution_drift,
        non_equivalence_causes,
        summary,
    }
}

fn edge_diff_key(edge: &bijux_dag_core::Edge) -> String {
    let kind = match edge.kind {
        bijux_dag_core::EdgeKind::Data => "data",
        bijux_dag_core::EdgeKind::Control => "control",
        bijux_dag_core::EdgeKind::Conditional => "conditional",
    };
    let id = edge.id.as_deref().unwrap_or("-");
    let decision = edge.decision.as_deref().unwrap_or("-");
    format!(
        "{kind}:{id}:{decision}:{}:{}->{}:{}",
        edge.from.node_id, edge.from.port, edge.to.node_id, edge.to.port
    )
}

fn serialized_value_eq<T: Serialize>(left: &T, right: &T) -> bool {
    serde_json::to_value(left).expect("planner diff comparison should serialize left operand")
        == serde_json::to_value(right)
            .expect("planner diff comparison should serialize right operand")
}

fn equivalence_causes_from_diff(diff: &PlannerPlanDiff) -> Vec<String> {
    let mut causes = Vec::new();
    causes.extend(diff.added_nodes.iter().map(|node_id| format!("added_node:{node_id}")));
    causes.extend(diff.removed_nodes.iter().map(|node_id| format!("removed_node:{node_id}")));
    causes.extend(diff.changed_params.iter().map(|node_id| format!("changed_params:{node_id}")));
    causes.extend(diff.changed_outputs.iter().map(|node_id| format!("changed_outputs:{node_id}")));
    causes.extend(
        diff.changed_resources.iter().map(|node_id| format!("changed_resources:{node_id}")),
    );
    causes.extend(
        diff.changed_retry_timeout.iter().map(|node_id| format!("changed_retry_timeout:{node_id}")),
    );
    causes.extend(diff.changed_effects.iter().map(|node_id| format!("changed_effects:{node_id}")));
    causes.extend(diff.changed_cache.iter().map(|node_id| format!("changed_cache:{node_id}")));
    causes.extend(
        diff.changed_env_allowlist.iter().map(|node_id| format!("changed_env_allowlist:{node_id}")),
    );
    causes.extend(
        diff.changed_trigger_rule.iter().map(|node_id| format!("changed_trigger_rule:{node_id}")),
    );
    causes.extend(
        diff.changed_branching.iter().map(|node_id| format!("changed_branching:{node_id}")),
    );
    causes.extend(
        diff.changed_node_kind.iter().map(|node_id| format!("changed_node_kind:{node_id}")),
    );
    causes.extend(
        diff.added_dependencies.iter().map(|dependency| format!("added_dependency:{dependency}")),
    );
    causes.extend(
        diff.removed_dependencies
            .iter()
            .map(|dependency| format!("removed_dependency:{dependency}")),
    );
    causes.sort();
    causes.dedup();
    causes
}

pub fn explain_plan(result: &PlannerBuildResult) -> PlannerExplainReport {
    PlannerExplainReport {
        plan_fingerprint: result.plan_fingerprint.clone(),
        phases: result.phases.clone(),
        annotations: result.annotations.clone(),
        optimization_notes: result
            .annotations
            .iter()
            .filter(|a| a.reason.starts_with("optimized"))
            .map(|a| format!("{} -> {}", a.node_id, a.reason))
            .collect(),
    }
}

fn build_path_previews(
    graph: &Graph,
    options: &RuntimeConfig,
    resolved_params: &BTreeMap<String, serde_json::Value>,
) -> Result<Option<Vec<PlannerNodePathPreview>>, String> {
    let Some(run_root) = options.run_root.as_ref() else {
        return Ok(None);
    };
    let layout = RunDirLayout::preview(run_root, options.run_id.as_deref())
        .map_err(|error| error.to_string())?;
    let effective_cache_dir = options.cache_dir.clone().or_else(crate::cache_dir_from_env);
    let mut previews = Vec::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        let (execution_surface, variable_bindings, resolved_paths, resolved_argv) =
            preview_node_paths(
                graph,
                node,
                &layout,
                options,
                effective_cache_dir.as_deref(),
                resolved_params,
            )?;
        previews.push(PlannerNodePathPreview {
            node_id: node.id.clone(),
            execution_surface,
            variable_bindings,
            resolved_paths,
            resolved_argv,
        });
    }
    Ok(Some(previews))
}

fn validate_command_templates(
    graph: &Graph,
    resolved_params: &BTreeMap<String, serde_json::Value>,
) -> Result<(), String> {
    for node in &graph.nodes {
        let Some(spec) = node.container.as_ref() else {
            continue;
        };
        let resolved = resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
        bijux_dag_core::resolve::resolve_command_argv_templates(graph, node, &spec.argv, &resolved)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn preview_node_paths(
    graph: &Graph,
    node: &Node,
    layout: &RunDirLayout,
    options: &RuntimeConfig,
    effective_cache_dir: Option<&std::path::Path>,
    resolved_params: &BTreeMap<String, serde_json::Value>,
) -> Result<(String, NodePathBindings, Vec<ResolvedPathUsage>, Option<Vec<String>>), String> {
    let resolved = resolved_params.get(&node.id).cloned().unwrap_or(Value::Null);
    if node.kind == bijux_dag_core::NodeKind::Container {
        let variable_bindings = NodePathBindings::for_container();
        let mut resolved_paths = Vec::new();
        let mut resolved_argv = None;
        if let Some(spec) = &node.container {
            let stable_argv = bijux_dag_core::resolve::resolve_command_argv_templates(
                graph, node, &spec.argv, &resolved,
            )
            .map_err(|error| error.to_string())?;
            resolved_paths
                .extend(collect_container_argv_path_usages(&stable_argv, &variable_bindings)?);
            resolved_argv = Some(resolve_container_argv(&stable_argv, &variable_bindings)?);
            if let Some(workdir_usage) = collect_container_workdir_usage(
                spec.workdir.as_deref(),
                &variable_bindings,
                options.absolute_path_policy,
            )? {
                resolved_paths.push(workdir_usage);
            }
        }
        return Ok(("container".to_string(), variable_bindings, resolved_paths, resolved_argv));
    }

    let variable_bindings = NodePathBindings::for_host(layout, &node.id, effective_cache_dir);
    let bound = crate::bind_path_variables_in_value(&resolved, &variable_bindings)?;
    let resolved_paths = collect_resolved_path_usages(&resolved, &variable_bindings)?;
    let resolved_argv = bound
        .get("argv")
        .and_then(Value::as_array)
        .map(|argv| {
            argv.iter()
                .map(|entry| {
                    entry
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "argv must resolve to strings".to_string())
                })
                .collect::<Result<Vec<_>, _>>()
        })
        .transpose()?;
    Ok(("host".to_string(), variable_bindings, resolved_paths, resolved_argv))
}

pub fn compute_partial_run_closure(
    plan: &ExecutionPlan,
    selected_nodes: &[String],
) -> BTreeSet<String> {
    fn expand(
        node_id: &str,
        dep_map: &HashMap<String, BTreeSet<String>>,
        keep: &mut BTreeSet<String>,
    ) {
        if !keep.insert(node_id.to_string()) {
            return;
        }
        if let Some(deps) = dep_map.get(node_id) {
            for dep in deps {
                expand(dep, dep_map, keep);
            }
        }
    }
    let mut keep = BTreeSet::new();
    for selected in selected_nodes {
        expand(selected, &plan.dep_map, &mut keep);
    }
    keep
}

pub fn compute_downstream_run_closure(
    graph: &Graph,
    selected_nodes: &[String],
) -> BTreeSet<String> {
    fn expand(
        node_id: &str,
        adjacency: &HashMap<String, BTreeSet<String>>,
        keep: &mut BTreeSet<String>,
    ) {
        if !keep.insert(node_id.to_string()) {
            return;
        }
        if let Some(children) = adjacency.get(node_id) {
            for child in children {
                expand(child, adjacency, keep);
            }
        }
    }

    let mut adjacency = HashMap::<String, BTreeSet<String>>::new();
    for edge in &graph.edges {
        adjacency.entry(edge.from.node_id.clone()).or_default().insert(edge.to.node_id.clone());
    }

    let mut keep = BTreeSet::new();
    for selected in selected_nodes {
        expand(selected, &adjacency, &mut keep);
    }
    keep
}

pub fn compute_upstream_run_closure(graph: &Graph, selected_nodes: &[String]) -> BTreeSet<String> {
    fn expand(
        node_id: &str,
        dep_map: &HashMap<String, BTreeSet<String>>,
        keep: &mut BTreeSet<String>,
    ) {
        if !keep.insert(node_id.to_string()) {
            return;
        }
        if let Some(deps) = dep_map.get(node_id) {
            for dep in deps {
                expand(dep, dep_map, keep);
            }
        }
    }

    let mut dep_map = HashMap::<String, BTreeSet<String>>::new();
    for edge in &graph.edges {
        dep_map.entry(edge.to.node_id.clone()).or_default().insert(edge.from.node_id.clone());
    }

    let mut keep = BTreeSet::new();
    for selected in selected_nodes {
        expand(selected, &dep_map, &mut keep);
    }
    keep
}

pub fn build_replay_plan_annotations(plan: &ExecutionPlan) -> Vec<PlannerNodeAnnotation> {
    plan.nodes
        .iter()
        .map(|node| {
            let replay_action = if plan.filter_reasons.contains_key(&node.id) {
                PlannerNodeAction::Skip
            } else {
                PlannerNodeAction::Execute
            };
            PlannerNodeAnnotation {
                node_id: node.id.clone(),
                selected: !plan.filter_reasons.contains_key(&node.id),
                reason: "replay_plan".to_string(),
                replay_action,
                locality_hint: Some("local-cache-first".to_string()),
                queue_hint: Some("default".to_string()),
            }
        })
        .collect()
}

pub fn build_backfill_plan(
    window_start_unix_ms: u128,
    window_end_unix_ms: u128,
    partition_keys: Vec<String>,
) -> PlannerBackfillPlan {
    PlannerBackfillPlan { window_start_unix_ms, window_end_unix_ms, partition_keys }
}

fn annotate_plan(
    graph: &Graph,
    plan: &ExecutionPlan,
    selector_set: &SelectorSet,
) -> Vec<PlannerNodeAnnotation> {
    let upstream_target_labels = plan
        .requested_selectors
        .iter()
        .filter_map(|value| value.strip_prefix("to-node:"))
        .collect::<BTreeSet<_>>();
    let downstream_root_labels = plan
        .requested_selectors
        .iter()
        .filter_map(|value| value.strip_prefix("from-node:"))
        .collect::<BTreeSet<_>>();
    graph
        .nodes
        .iter()
        .map(|node| {
            let selected = !plan.filter_reasons.contains_key(&node.id);
            let reason = if let Some(filter_reason) = plan.filter_reasons.get(&node.id) {
                filter_reason.clone()
            } else if upstream_target_labels.contains(node.id.as_str()) {
                "selected_by_to_node".to_string()
            } else if !upstream_target_labels.is_empty() {
                "selected_by_upstream_closure".to_string()
            } else if downstream_root_labels.contains(node.id.as_str()) {
                "selected_by_from_node".to_string()
            } else if !downstream_root_labels.is_empty() {
                "selected_by_downstream_closure".to_string()
            } else if !selector_set.include.is_empty()
                && selector_set.include.iter().any(|selector| selector_matches(node, selector))
            {
                "selected_by_include_selector".to_string()
            } else if !selector_set.include.is_empty() {
                "selected_by_dependency_closure".to_string()
            } else {
                "selected_by_default".to_string()
            };
            PlannerNodeAnnotation {
                node_id: node.id.clone(),
                selected,
                reason,
                replay_action: if selected {
                    PlannerNodeAction::Execute
                } else {
                    PlannerNodeAction::Skip
                },
                locality_hint: Some("artifact-locality-preferred".to_string()),
                queue_hint: node.group.clone().or_else(|| Some("default".to_string())),
            }
        })
        .collect()
}

fn selector_matches(node: &Node, selector: &Selector) -> bool {
    match selector {
        Selector::Id(id) => node.id == *id,
        Selector::IdPrefix(prefix) => node.id.starts_with(prefix),
        Selector::Tag(tag) => node.tags.iter().any(|candidate| candidate == tag),
        Selector::Kind(kind) => node.kind.as_str() == kind,
    }
}

fn estimate_execution_cost(
    graph: &Graph,
    plan: &ExecutionPlan,
    options: &RuntimeConfig,
) -> PlannerExecutionCostEstimate {
    let selected_nodes = plan
        .nodes
        .iter()
        .filter(|node| !plan.filter_reasons.contains_key(&node.id))
        .collect::<Vec<_>>();
    let selected_node_ids =
        selected_nodes.iter().map(|node| node.id.clone()).collect::<BTreeSet<_>>();
    let selected_dependencies = plan
        .planned_dependencies
        .iter()
        .filter(|edge| {
            selected_node_ids.contains(&edge.from) && selected_node_ids.contains(&edge.to)
        })
        .collect::<Vec<_>>();
    let order = plan
        .order
        .iter()
        .filter(|node_id| selected_node_ids.contains(*node_id))
        .cloned()
        .collect::<Vec<_>>();

    let mut indegree =
        selected_nodes.iter().map(|node| (node.id.clone(), 0usize)).collect::<HashMap<_, _>>();
    let mut adjacency = selected_nodes
        .iter()
        .map(|node| (node.id.clone(), Vec::<String>::new()))
        .collect::<HashMap<_, _>>();
    for edge in &selected_dependencies {
        adjacency.entry(edge.from.clone()).or_default().push(edge.to.clone());
        *indegree.entry(edge.to.clone()).or_default() += 1;
    }
    for children in adjacency.values_mut() {
        children.sort();
    }

    let mut root_nodes = indegree
        .iter()
        .filter_map(|(node_id, count)| (*count == 0).then_some(node_id.clone()))
        .collect::<Vec<_>>();
    root_nodes.sort();

    let mut cpu_cores_total = 0u64;
    let mut memory_mb_total = 0u64;
    let mut gpu_devices_total = 0u64;
    let mut named_resources_total = BTreeMap::<String, u64>::new();
    let mut cacheable_nodes = 0usize;
    let mut non_cacheable_node_ids = Vec::new();
    let mut timed_node_ids = Vec::new();
    let mut max_timeout_ms = None::<u64>;
    let mut total_timeout_ms = 0u64;
    let mut retrying_node_ids = Vec::new();
    let mut max_attempts = 0u32;
    let mut max_backoff_ms = 0u64;
    let mut total_retry_attempts = 0u64;

    for node in &selected_nodes {
        let (cpu, memory, gpu) = node_resource_demand(node);
        cpu_cores_total += cpu;
        memory_mb_total += memory;
        gpu_devices_total += gpu;
        accumulate_named_resource_demand(
            &mut named_resources_total,
            &node_named_resource_demand(node),
        );
        if node.cache.enabled {
            cacheable_nodes += 1;
        } else {
            non_cacheable_node_ids.push(node.id.clone());
        }
        if let Some(timeout_ms) = node.timeout_ms {
            timed_node_ids.push(node.id.clone());
            total_timeout_ms += timeout_ms;
            max_timeout_ms =
                Some(max_timeout_ms.map_or(timeout_ms, |current| current.max(timeout_ms)));
        }
        if node.retry.max_attempts > 0 {
            retrying_node_ids.push(node.id.clone());
            max_attempts = max_attempts.max(node.retry.max_attempts);
            max_backoff_ms = max_backoff_ms.max(node.retry.backoff_ms);
            total_retry_attempts += u64::from(node.retry.max_attempts);
        }
    }

    non_cacheable_node_ids.sort();
    timed_node_ids.sort();
    retrying_node_ids.sort();

    let critical_path = critical_path_estimate(&order, &selected_dependencies, &selected_nodes);
    let critical_path_length = critical_path.node_ids.len();
    let (
        max_parallelism,
        cpu_cores_peak_parallel,
        memory_mb_peak_parallel,
        gpu_devices_peak_parallel,
        named_resources_peak_parallel,
    ) = parallelism_profile(&selected_nodes, &indegree, &adjacency);
    let scheduling_simulation = scheduling_simulation(
        graph,
        options,
        &selected_nodes,
        &indegree,
        &adjacency,
        critical_path.total_duration_ms,
    );

    PlannerExecutionCostEstimate {
        node_count: selected_nodes.len(),
        root_nodes,
        critical_path_length,
        critical_path,
        max_parallelism,
        demand: PlannerExecutionDemand {
            cpu_cores_total,
            memory_mb_total,
            gpu_devices_total,
            cpu_cores_peak_parallel,
            memory_mb_peak_parallel,
            gpu_devices_peak_parallel,
            named_resources_total,
            named_resources_peak_parallel,
        },
        scheduling_simulation,
        cache_exposure: PlannerCacheExposure {
            cacheable_nodes,
            non_cacheable_nodes: non_cacheable_node_ids.len(),
            non_cacheable_node_ids,
        },
        timeout_exposure: PlannerTimeoutExposure {
            timed_nodes: timed_node_ids.len(),
            timed_node_ids,
            max_timeout_ms,
            total_timeout_ms,
        },
        retry_exposure: PlannerRetryExposure {
            retrying_nodes: retrying_node_ids.len(),
            retrying_node_ids,
            max_attempts,
            max_backoff_ms,
            total_retry_attempts,
        },
    }
}

fn node_resource_demand(node: &Node) -> (u64, u64, u64) {
    let cpu = node.resources.as_ref().map(|resources| resources.cpu as u64).unwrap_or(1);
    let memory = node.resources.as_ref().map(|resources| resources.mem_mb as u64).unwrap_or(256);
    let gpu = u64::from(resources::node_gpu_devices(node));
    (cpu, memory, gpu)
}

fn node_named_resource_demand(node: &Node) -> BTreeMap<String, u64> {
    resources::node_named_resources(node)
        .into_iter()
        .map(|(name, amount)| (name, u64::from(amount)))
        .collect()
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannerNodeDurationEstimate {
    duration_ms: u64,
    duration_source: PlannerDurationSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PlannerCriticalPathState {
    node_ids: Vec<String>,
    total_duration_ms: u64,
    estimated_duration_nodes: usize,
    unit_duration_fallback_nodes: usize,
    nodes: Vec<PlannerCriticalPathNode>,
}

fn critical_path_estimate(
    order: &[String],
    dependencies: &[&crate::PlannedDependency],
    nodes: &[&Node],
) -> PlannerCriticalPathEstimate {
    let node_lookup =
        nodes.iter().map(|node| (node.id.clone(), *node)).collect::<HashMap<String, &Node>>();
    let mut parent_map = HashMap::<String, Vec<String>>::new();
    for edge in dependencies {
        parent_map.entry(edge.to.clone()).or_default().push(edge.from.clone());
    }
    for parents in parent_map.values_mut() {
        parents.sort();
    }

    let mut best_path_by_node = HashMap::<String, PlannerCriticalPathState>::new();
    let mut longest_path = None::<PlannerCriticalPathState>;
    for node_id in order {
        let duration_estimate = node_lookup
            .get(node_id)
            .map(|node| node_duration_estimate(node))
            .unwrap_or_else(unit_duration_estimate);
        let parent_state = parent_map.get(node_id).and_then(|parents| {
            parents
                .iter()
                .filter_map(|parent| best_path_by_node.get(parent))
                .max_by(|left, right| compare_critical_path_state(left, right))
                .cloned()
        });
        let next_state =
            append_critical_path_state(parent_state.as_ref(), node_id, &duration_estimate);
        best_path_by_node.insert(node_id.clone(), next_state.clone());
        if longest_path
            .as_ref()
            .is_none_or(|current| compare_critical_path_state(current, &next_state).is_lt())
        {
            longest_path = Some(next_state);
        }
    }

    if let Some(path) = longest_path {
        PlannerCriticalPathEstimate {
            node_ids: path.node_ids,
            total_duration_ms: path.total_duration_ms,
            estimated_duration_nodes: path.estimated_duration_nodes,
            unit_duration_fallback_nodes: path.unit_duration_fallback_nodes,
            nodes: path.nodes,
        }
    } else {
        PlannerCriticalPathEstimate {
            node_ids: Vec::new(),
            total_duration_ms: 0,
            estimated_duration_nodes: 0,
            unit_duration_fallback_nodes: 0,
            nodes: Vec::new(),
        }
    }
}

fn append_critical_path_state(
    parent_state: Option<&PlannerCriticalPathState>,
    node_id: &str,
    duration_estimate: &PlannerNodeDurationEstimate,
) -> PlannerCriticalPathState {
    let mut state = parent_state.cloned().unwrap_or_else(|| PlannerCriticalPathState {
        node_ids: Vec::new(),
        total_duration_ms: 0,
        estimated_duration_nodes: 0,
        unit_duration_fallback_nodes: 0,
        nodes: Vec::new(),
    });
    state.node_ids.push(node_id.to_string());
    state.total_duration_ms += duration_estimate.duration_ms;
    match duration_estimate.duration_source {
        PlannerDurationSource::EstimatedDuration => state.estimated_duration_nodes += 1,
        PlannerDurationSource::UnitFallback => state.unit_duration_fallback_nodes += 1,
    }
    state.nodes.push(PlannerCriticalPathNode {
        node_id: node_id.to_string(),
        duration_ms: duration_estimate.duration_ms,
        duration_source: duration_estimate.duration_source.clone(),
    });
    state
}

fn compare_critical_path_state(
    left: &PlannerCriticalPathState,
    right: &PlannerCriticalPathState,
) -> std::cmp::Ordering {
    left.total_duration_ms
        .cmp(&right.total_duration_ms)
        .then(left.node_ids.len().cmp(&right.node_ids.len()))
        .then_with(|| right.node_ids.cmp(&left.node_ids))
}

fn node_duration_estimate(node: &Node) -> PlannerNodeDurationEstimate {
    param_literal_u64(node, "estimated_duration_ms")
        .filter(|duration_ms| *duration_ms > 0)
        .map(|duration_ms| PlannerNodeDurationEstimate {
            duration_ms,
            duration_source: PlannerDurationSource::EstimatedDuration,
        })
        .unwrap_or_else(unit_duration_estimate)
}

fn unit_duration_estimate() -> PlannerNodeDurationEstimate {
    PlannerNodeDurationEstimate {
        duration_ms: 1,
        duration_source: PlannerDurationSource::UnitFallback,
    }
}

fn param_literal_u64(node: &Node, key: &str) -> Option<u64> {
    match &node.params {
        bijux_dag_core::ParamValue::Object(map) => match map.get(key) {
            Some(bijux_dag_core::ParamValue::Literal(value)) => value.as_u64(),
            _ => None,
        },
        _ => None,
    }
}

fn parallelism_profile(
    nodes: &[&Node],
    indegree: &HashMap<String, usize>,
    adjacency: &HashMap<String, Vec<String>>,
) -> (usize, u64, u64, u64, BTreeMap<String, u64>) {
    if nodes.is_empty() {
        return (0, 0, 0, 0, BTreeMap::new());
    }

    let node_lookup =
        nodes.iter().map(|node| (node.id.clone(), *node)).collect::<HashMap<String, &Node>>();
    let mut remaining_indegree = indegree.clone();
    let mut ready = remaining_indegree
        .iter()
        .filter_map(|(node_id, count)| (*count == 0).then_some(node_id.clone()))
        .collect::<Vec<_>>();
    ready.sort();

    let mut max_parallelism = 0usize;
    let mut cpu_cores_peak_parallel = 0u64;
    let mut memory_mb_peak_parallel = 0u64;
    let mut gpu_devices_peak_parallel = 0u64;
    let mut named_resources_peak_parallel = BTreeMap::<String, u64>::new();

    while !ready.is_empty() {
        max_parallelism = max_parallelism.max(ready.len());
        let mut batch_cpu = 0u64;
        let mut batch_memory = 0u64;
        let mut batch_gpu = 0u64;
        let mut batch_named_resources = BTreeMap::<String, u64>::new();
        let batch = ready.clone();
        let mut next_ready = Vec::new();

        for node_id in &batch {
            let node = node_lookup
                .get(node_id)
                .expect("selected node must be present in execution cost lookup");
            let (cpu, memory, gpu) = node_resource_demand(node);
            batch_cpu += cpu;
            batch_memory += memory;
            batch_gpu += gpu;
            accumulate_named_resource_demand(
                &mut batch_named_resources,
                &node_named_resource_demand(node),
            );
            if let Some(children) = adjacency.get(node_id) {
                for child in children {
                    let count = remaining_indegree
                        .get_mut(child)
                        .expect("child indegree must exist in execution cost profile");
                    *count -= 1;
                    if *count == 0 {
                        next_ready.push(child.clone());
                    }
                }
            }
        }

        cpu_cores_peak_parallel = cpu_cores_peak_parallel.max(batch_cpu);
        memory_mb_peak_parallel = memory_mb_peak_parallel.max(batch_memory);
        gpu_devices_peak_parallel = gpu_devices_peak_parallel.max(batch_gpu);
        maximize_named_resource_demand(&mut named_resources_peak_parallel, &batch_named_resources);
        next_ready.sort();
        next_ready.dedup();
        ready = next_ready;
    }

    (
        max_parallelism,
        cpu_cores_peak_parallel,
        memory_mb_peak_parallel,
        gpu_devices_peak_parallel,
        named_resources_peak_parallel,
    )
}

#[derive(Debug, Clone, Default)]
struct PlannerBlockedNodeAccumulator {
    blocked_reasons: BTreeSet<String>,
    blocked_waves: usize,
    blocked_duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
struct PlannerResourceBottleneckAccumulator {
    blocked_node_ids: BTreeSet<String>,
    blocking_events: usize,
    blocked_duration_ms: u64,
}

fn scheduling_simulation(
    graph: &Graph,
    options: &RuntimeConfig,
    selected_nodes: &[&Node],
    indegree: &HashMap<String, usize>,
    adjacency: &HashMap<String, Vec<String>>,
    topology_critical_path_duration_ms: u64,
) -> PlannerSchedulingSimulation {
    if selected_nodes.is_empty() {
        return PlannerSchedulingSimulation {
            scheduled_waves: 0,
            projected_makespan_ms: 0,
            resource_delay_ms: 0,
            run_bound: PlannerSchedulingBound::DependencyBound,
            bottlenecks: Vec::new(),
            blocked_nodes: Vec::new(),
        };
    }

    let node_lookup = selected_nodes
        .iter()
        .map(|node| (node.id.clone(), *node))
        .collect::<HashMap<String, &Node>>();
    let mut remaining_indegree = indegree.clone();
    let mut ready_queue = ReadyQueue::from_indegree(indegree);
    let mut scheduler = crate::build_scheduler(&options.scheduler_policy);
    let mut scheduled_waves = 0usize;
    let mut projected_makespan_ms = 0u64;
    let mut blocked_nodes = BTreeMap::<String, PlannerBlockedNodeAccumulator>::new();
    let mut bottlenecks = BTreeMap::<String, PlannerResourceBottleneckAccumulator>::new();

    while !ready_queue.is_empty() {
        let decision = scheduler.next_batch(
            graph,
            &mut ready_queue,
            options,
            std::time::Instant::now(),
            false,
        );
        if decision.batch.is_empty() {
            break;
        }
        scheduled_waves += 1;
        let wave_duration_ms = decision
            .batch
            .iter()
            .filter_map(|node_id| node_lookup.get(node_id))
            .map(|node| node_duration_estimate(node).duration_ms)
            .max()
            .unwrap_or(0);
        projected_makespan_ms += wave_duration_ms;

        for node_id in &decision.blocked_by_budget {
            let reason = decision
                .blocked_reasons
                .get(node_id)
                .cloned()
                .unwrap_or_else(|| "blocked_by_policy".to_string());
            let blocked_node = blocked_nodes.entry(node_id.clone()).or_default();
            blocked_node.blocked_reasons.insert(reason.clone());
            blocked_node.blocked_waves += 1;
            blocked_node.blocked_duration_ms += wave_duration_ms;

            let resource = blocked_resource_label(&reason);
            let bottleneck = bottlenecks.entry(resource).or_default();
            bottleneck.blocking_events += 1;
            bottleneck.blocked_duration_ms += wave_duration_ms;
            bottleneck.blocked_node_ids.insert(node_id.clone());
        }

        for node_id in &decision.batch {
            if let Some(children) = adjacency.get(node_id) {
                for child in children {
                    let count = remaining_indegree
                        .get_mut(child)
                        .expect("child indegree must exist in scheduling simulation");
                    *count = count.saturating_sub(1);
                    if *count == 0 {
                        ready_queue.insert(child.clone());
                    }
                }
            }
        }
    }

    let resource_delay_ms =
        projected_makespan_ms.saturating_sub(topology_critical_path_duration_ms);
    let run_bound = if resource_delay_ms > 0 {
        PlannerSchedulingBound::ResourceBound
    } else {
        PlannerSchedulingBound::DependencyBound
    };
    let mut blocked_nodes = blocked_nodes
        .into_iter()
        .map(|(node_id, entry)| PlannerBlockedNodeEstimate {
            node_id,
            blocked_by: entry.blocked_reasons.into_iter().collect(),
            blocked_waves: entry.blocked_waves,
            blocked_duration_ms: entry.blocked_duration_ms,
        })
        .collect::<Vec<_>>();
    blocked_nodes.sort_by(|left, right| left.node_id.cmp(&right.node_id));

    let mut bottlenecks = bottlenecks
        .into_iter()
        .map(|(resource, entry)| PlannerResourceBottleneck {
            resource,
            blocking_events: entry.blocking_events,
            blocked_node_ids: entry.blocked_node_ids.into_iter().collect(),
            blocked_duration_ms: entry.blocked_duration_ms,
        })
        .collect::<Vec<_>>();
    bottlenecks.sort_by(|left, right| {
        right
            .blocked_duration_ms
            .cmp(&left.blocked_duration_ms)
            .then_with(|| left.resource.cmp(&right.resource))
    });

    PlannerSchedulingSimulation {
        scheduled_waves,
        projected_makespan_ms,
        resource_delay_ms,
        run_bound,
        bottlenecks,
        blocked_nodes,
    }
}

fn blocked_resource_label(reason: &str) -> String {
    if let Some(name) = reason.strip_prefix("blocked_by_named_resource:") {
        return format!("named_resource:{name}");
    }
    match reason {
        "blocked_by_parallelism" => "parallelism".to_string(),
        "blocked_by_cpu" => "cpu_cores".to_string(),
        "blocked_by_memory" => "memory_mb".to_string(),
        "blocked_by_gpu" => "gpu_devices".to_string(),
        _ => reason.to_string(),
    }
}

fn accumulate_named_resource_demand(
    totals: &mut BTreeMap<String, u64>,
    demand: &BTreeMap<String, u64>,
) {
    for (name, amount) in demand {
        *totals.entry(name.clone()).or_default() += *amount;
    }
}

fn maximize_named_resource_demand(
    peaks: &mut BTreeMap<String, u64>,
    demand: &BTreeMap<String, u64>,
) {
    for (name, amount) in demand {
        let peak = peaks.entry(name.clone()).or_default();
        *peak = (*peak).max(*amount);
    }
}

fn inherit_priority(nodes: &[Node]) -> Vec<PlannerPriorityInheritance> {
    nodes
        .iter()
        .map(|node| {
            let inherited_priority = if node.tags.iter().any(|t| t == "critical") {
                "critical"
            } else if node.tags.iter().any(|t| t == "batch") {
                "batch"
            } else {
                "standard"
            };
            PlannerPriorityInheritance {
                node_id: node.id.clone(),
                inherited_priority: inherited_priority.to_string(),
            }
        })
        .collect()
}

fn validate_impossible_run_requirements(graph: &Graph) -> Result<(), String> {
    for node in &graph.nodes {
        if let Some(resources) = node.resources.as_ref() {
            if resources.cpu == 0 || resources.mem_mb == 0 {
                return Err(format!(
                    "node '{}' has impossible resource requirements (cpu={}, mem_mb={})",
                    node.id, resources.cpu, resources.mem_mb
                ));
            }
            for (name, amount) in &resources.named_resources {
                if name.trim().is_empty() || *amount == 0 {
                    return Err(format!(
                        "node '{}' has impossible named resource requirement ('{}'={})",
                        node.id, name, amount
                    ));
                }
            }
        }
    }
    Ok(())
}

fn validate_backend_compatibility(graph: &Graph) -> Result<(), String> {
    let local_capabilities = BackendCapabilities {
        supports_container: true,
        supports_network_isolation: true,
        supports_env_allowlist: true,
        supports_artifact_mounts: true,
        supports_remote_logs: false,
        supports_gpu: false,
    };
    for node in &graph.nodes {
        let requires_container = matches!(node.kind, bijux_dag_core::NodeKind::Container);
        let requirements = BackendCapabilityRequirement {
            container_required: requires_container,
            network_isolation_required: false,
            env_allowlist_required: !crate::effective_env_allowlist(node).is_empty(),
            artifact_mount_required: true,
            remote_logs_required: false,
            gpu_required: node.tags.iter().any(|t| t == "gpu"),
        };
        let decision = negotiate_backend_capabilities(&local_capabilities, &requirements);
        if !decision.accepted {
            return Err(format!(
                "node '{}' is incompatible with planner backend capabilities: {}",
                node.id, decision.reason
            ));
        }
    }
    Ok(())
}
