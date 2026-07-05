use crate::execution_plan::ExecutionPlan;
use crate::infrastructure::{
    negotiate_backend_capabilities, BackendCapabilities, BackendCapabilityRequirement,
};
use crate::{
    collect_container_argv_path_usages, collect_container_workdir_usage,
    collect_resolved_path_usages, NodePathBindings, ResolvedPathUsage, RuntimeConfig, SelectorSet,
};
use bijux_dag_artifacts::RunDirLayout;
use bijux_dag_core::{Graph, Node};
use serde::{Deserialize, Serialize};
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
pub struct PlannerResourceEstimate {
    pub total_cpu: u64,
    pub total_mem_mb: u64,
    pub max_parallelism_hint: usize,
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
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlannerPlanDiff {
    pub changed_order_nodes: Vec<String>,
    pub changed_filter_reasons: Vec<String>,
    pub changed_annotations: Vec<String>,
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
    pub resource_estimate: PlannerResourceEstimate,
    pub priority_inheritance: Vec<PlannerPriorityInheritance>,
    pub plan_fingerprint: String,
    pub path_previews: Option<Vec<PlannerNodePathPreview>>,
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
    let mut annotations = annotate_plan(&normalized_graph, &plan, selector_set);
    plan = apply_optimizer_rules(normalized_graph.clone(), plan, &mut annotations, guardrails);
    let resource_estimate = estimate_resources(&plan.nodes);
    let priority_inheritance = inherit_priority(&plan.nodes);
    let plan_fingerprint = fingerprint_plan(&plan, &annotations)?;
    let path_previews = build_path_previews(&normalized_graph, options)?;

    Ok(PlannerBuildResult {
        plan,
        phases,
        annotations,
        resource_estimate,
        priority_inheritance,
        plan_fingerprint,
        path_previews,
    })
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
    let before_order: BTreeSet<String> = before.plan.order.iter().cloned().collect();
    let after_order: BTreeSet<String> = after.plan.order.iter().cloned().collect();
    let changed_order_nodes =
        before_order.symmetric_difference(&after_order).cloned().collect::<Vec<_>>();

    let mut changed_filter_reasons = Vec::new();
    for (k, v) in &after.plan.filter_reasons {
        if before.plan.filter_reasons.get(k) != Some(v) {
            changed_filter_reasons.push(format!("{k}:{v}"));
        }
    }

    let before_ann: BTreeMap<String, String> =
        before.annotations.iter().map(|a| (a.node_id.clone(), a.reason.clone())).collect();
    let mut changed_annotations = Vec::new();
    for ann in &after.annotations {
        if before_ann.get(&ann.node_id) != Some(&ann.reason) {
            changed_annotations.push(format!("{}:{}", ann.node_id, ann.reason));
        }
    }
    PlannerPlanDiff { changed_order_nodes, changed_filter_reasons, changed_annotations }
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
) -> Result<Option<Vec<PlannerNodePathPreview>>, String> {
    let Some(run_root) = options.run_root.as_ref() else {
        return Ok(None);
    };
    let layout = RunDirLayout::preview(run_root, options.run_id.as_deref())
        .map_err(|error| error.to_string())?;
    let resolved_graph = graph.resolve_graph().map_err(|error| error.to_string())?;
    let effective_cache_dir = options.cache_dir.clone().or_else(crate::cache_dir_from_env);
    let mut previews = Vec::with_capacity(graph.nodes.len());
    for node in &graph.nodes {
        let (execution_surface, variable_bindings, resolved_paths) =
            preview_node_paths(
                node,
                &layout,
                options,
                effective_cache_dir.as_deref(),
                &resolved_graph.resolved_params,
            )?;
        previews.push(PlannerNodePathPreview {
            node_id: node.id.clone(),
            execution_surface,
            variable_bindings,
            resolved_paths,
        });
    }
    Ok(Some(previews))
}

fn preview_node_paths(
    node: &Node,
    layout: &RunDirLayout,
    options: &RuntimeConfig,
    effective_cache_dir: Option<&std::path::Path>,
    resolved_params: &BTreeMap<String, serde_json::Value>,
) -> Result<(String, NodePathBindings, Vec<ResolvedPathUsage>), String> {
    if node.kind == bijux_dag_core::NodeKind::Container {
        let variable_bindings = NodePathBindings::for_container();
        let mut resolved_paths = Vec::new();
        if let Some(spec) = &node.container {
            resolved_paths.extend(collect_container_argv_path_usages(
                &spec.argv,
                &variable_bindings,
            )?);
            if let Some(workdir_usage) = collect_container_workdir_usage(
                spec.workdir.as_deref(),
                &variable_bindings,
                options.absolute_path_policy,
            )? {
                resolved_paths.push(workdir_usage);
            }
        }
        return Ok(("container".to_string(), variable_bindings, resolved_paths));
    }

    let variable_bindings = NodePathBindings::for_host(layout, &node.id, effective_cache_dir);
    let resolved = resolved_params.get(&node.id).cloned().unwrap_or(serde_json::Value::Null);
    let resolved_paths = collect_resolved_path_usages(&resolved, &variable_bindings)?;
    Ok(("host".to_string(), variable_bindings, resolved_paths))
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
    graph
        .nodes
        .iter()
        .map(|node| {
            let selected = !plan.filter_reasons.contains_key(&node.id);
            let reason = if let Some(filter_reason) = plan.filter_reasons.get(&node.id) {
                filter_reason.clone()
            } else if !selector_set.include.is_empty() {
                "selected_by_include_selector".to_string()
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

fn estimate_resources(nodes: &[Node]) -> PlannerResourceEstimate {
    let total_cpu =
        nodes.iter().map(|n| n.resources.as_ref().map(|r| r.cpu as u64).unwrap_or(1)).sum();
    let total_mem_mb =
        nodes.iter().map(|n| n.resources.as_ref().map(|r| r.mem_mb as u64).unwrap_or(256)).sum();
    PlannerResourceEstimate { total_cpu, total_mem_mb, max_parallelism_hint: nodes.len().max(1) }
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
            env_allowlist_required: !node.env_allowlist.is_empty(),
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
