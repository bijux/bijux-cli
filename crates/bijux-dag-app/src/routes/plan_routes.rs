use crate::commands::{AbsolutePathPolicyArg, DagCli, PlanCommands};
use crate::graph::selection::selection_summary_from_planner;
use crate::graph_helpers::{
    parse_selectors, resolve_downstream_run_selection, resolve_upstream_run_selection,
    validate_partial_selection_surface,
};
use crate::routes::preconditions::require_safe_path;
use crate::routes::resource_capacity_args::parse_resource_capacities;
use crate::{emit_json, load_graphs_or_emit, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::RunDirLayout;
use bijux_dag_runtime::{
    build_backfill_plan, build_planner_analysis, compare_plan_equivalence,
    compute_partial_run_closure, diff_plans, explain_plan, AbsolutePathPolicy, PlannerBuildResult,
    PlannerEquivalenceReport, PlannerGuardrails, PlannerPlanDiff, RuntimeConfig, SelectorSet,
};
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub(crate) struct PlanPreviewConfig {
    pub run_root: Option<PathBuf>,
    pub run_id: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicy,
    pub jobs: usize,
    pub cpu_budget: Option<u32>,
    pub memory_budget_mb: Option<u32>,
    pub gpu_device_budget: Option<u32>,
    pub named_resource_capacities: std::collections::BTreeMap<String, u32>,
    pub upstream_selection_targets: Vec<String>,
    pub downstream_selection_roots: Vec<String>,
    pub selectors: SelectorSet,
    pub dependency_closure: bool,
}

impl Default for PlanPreviewConfig {
    fn default() -> Self {
        Self {
            run_root: None,
            run_id: None,
            cache_dir: None,
            absolute_path_policy: AbsolutePathPolicy::AllowLiteral,
            jobs: 1,
            cpu_budget: None,
            memory_budget_mb: None,
            gpu_device_budget: None,
            named_resource_capacities: std::collections::BTreeMap::new(),
            upstream_selection_targets: Vec::new(),
            downstream_selection_roots: Vec::new(),
            selectors: SelectorSet::default(),
            dependency_closure: false,
        }
    }
}

impl From<AbsolutePathPolicyArg> for AbsolutePathPolicy {
    fn from(value: AbsolutePathPolicyArg) -> Self {
        match value {
            AbsolutePathPolicyArg::AllowLiteral => Self::AllowLiteral,
            AbsolutePathPolicyArg::DenyLiteral => Self::DenyLiteral,
        }
    }
}

pub(crate) fn default_analysis_runtime_config(preview: &PlanPreviewConfig) -> RuntimeConfig {
    RuntimeConfig {
        jobs: preview.jobs,
        cpu_budget: preview.cpu_budget,
        memory_budget_mb: preview.memory_budget_mb,
        gpu_device_budget: preview.gpu_device_budget,
        named_resource_capacities: preview.named_resource_capacities.clone(),
        scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
            max_parallelism: preview.jobs.max(1),
            ..bijux_dag_runtime::SchedulerPolicy::default()
        },
        run_root: preview.run_root.clone(),
        run_id: preview.run_id.clone(),
        cache_dir: preview.cache_dir.clone(),
        absolute_path_policy: preview.absolute_path_policy,
        upstream_selection_targets: preview.upstream_selection_targets.clone(),
        downstream_selection_roots: preview.downstream_selection_roots.clone(),
        selectors: preview.selectors.clone(),
        partial_rerun_dependency_closure: preview.dependency_closure,
        ..RuntimeConfig::default()
    }
}

pub(crate) fn resolve_plan_preview_layout(
    run_root: Option<&Path>,
    run_id: Option<&str>,
) -> Result<Option<RunDirLayout>, ExitCode> {
    let Some(run_root) = run_root else {
        return Ok(None);
    };
    require_safe_path(run_root)?;
    RunDirLayout::preview(run_root, run_id).map(Some).map_err(|_| ExitCode::from(2))
}

pub(crate) fn build_default_planner_analysis(
    graph: &bijux_dag_core::Graph,
    preview: &PlanPreviewConfig,
) -> Result<PlannerBuildResult, String> {
    let config = default_analysis_runtime_config(preview);
    build_planner_analysis(
        graph,
        &config,
        &config.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
}

pub(crate) fn concise_plan_lines(result: &PlannerBuildResult) -> Vec<String> {
    result
        .annotations
        .iter()
        .map(|annotation| {
            let queue_hint = annotation.queue_hint.as_deref().unwrap_or("default");
            format!(
                "{}: {:?} via {} ({}, queue={})",
                annotation.node_id,
                annotation.replay_action,
                annotation.reason,
                if annotation.selected { "selected" } else { "filtered" },
                queue_hint
            )
        })
        .collect()
}

pub(crate) fn plan_explain_payload(
    result: &PlannerBuildResult,
    preview_layout: Option<&RunDirLayout>,
    absolute_path_policy: AbsolutePathPolicy,
) -> serde_json::Value {
    let report = explain_plan(result);
    let selection = selection_summary_from_planner(result);
    json!({
        "planner_contract_version": result.plan.planner_contract_version,
        "plan_fingerprint": report.plan_fingerprint,
        "run_layout": preview_layout,
        "absolute_path_policy": absolute_path_policy,
        "selection": {
            "requested_selectors": selection.requested_selectors,
            "upstream_targets": selection.upstream_targets,
            "downstream_roots": selection.downstream_roots,
            "dependency_closure_enabled": selection.dependency_closure_enabled,
            "selected_nodes": selection.selected_nodes,
            "omitted_nodes": selection.omitted_nodes,
        },
        "phases": report.phases,
        "ordering": result.plan.order,
        "execution_cost_estimate": result.execution_cost_estimate,
        "priority_inheritance": result.priority_inheritance,
        "optimization_notes": report.optimization_notes,
        "nodes": report.annotations,
        "planned_nodes": result.plan.planned_nodes,
        "planned_edges": result.plan.planned_dependencies,
        "branch_paths": result.plan.branch_paths,
        "diagnostics": result.plan.diagnostics,
        "path_previews": result.path_previews,
    })
}

pub(crate) fn plan_diff_changed(diff: &PlannerPlanDiff) -> bool {
    diff.graph_fingerprint_changed
}

pub(crate) fn plan_diff_payload(
    before_result: &PlannerBuildResult,
    after_result: &PlannerBuildResult,
    diff: &PlannerPlanDiff,
) -> serde_json::Value {
    json!({
        "changed": plan_diff_changed(diff),
        "before_plan_fingerprint": before_result.plan_fingerprint,
        "after_plan_fingerprint": after_result.plan_fingerprint,
        "before_execution_cost_estimate": before_result.execution_cost_estimate,
        "after_execution_cost_estimate": after_result.execution_cost_estimate,
        "diff": diff,
    })
}

pub(crate) fn plan_equivalence_payload(
    before_result: &PlannerBuildResult,
    after_result: &PlannerBuildResult,
    report: &PlannerEquivalenceReport,
) -> serde_json::Value {
    json!({
        "equivalent": report.equivalent,
        "report": report,
        "before_plan_fingerprint": before_result.plan_fingerprint,
        "after_plan_fingerprint": after_result.plan_fingerprint,
        "before_execution_cost_estimate": before_result.execution_cost_estimate,
        "after_execution_cost_estimate": after_result.execution_cost_estimate,
    })
}

fn equivalence_class_label(report: &PlannerEquivalenceReport) -> &'static str {
    match report.equivalence_class {
        bijux_dag_runtime::PlannerEquivalenceClass::StrictEquivalent => "strict_equivalent",
        bijux_dag_runtime::PlannerEquivalenceClass::MetadataDriftEquivalent => {
            "metadata_drift_equivalent"
        }
        bijux_dag_runtime::PlannerEquivalenceClass::NotEquivalent => "not_equivalent",
    }
}

pub(crate) fn handle_plan_command(
    cli: &DagCli,
    command: &PlanCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PlanCommands::Explain {
            dags,
            out,
            run_id,
            cache_dir,
            absolute_path_policy,
            jobs,
            cpu_budget,
            memory_budget_mb,
            gpu_device_budget,
            resource_capacity,
            from_node,
            to_node,
            select,
            exclude,
            dependency_closure,
        } => {
            let graph = load_graphs_or_emit(cli, "dag.plan.explain", dags)?;
            validate_partial_selection_surface(
                from_node,
                to_node,
                select,
                exclude,
                *dependency_closure,
            )?;
            let (upstream_selection_targets, _) = resolve_upstream_run_selection(&graph, to_node)?;
            let (downstream_selection_roots, _) =
                resolve_downstream_run_selection(&graph, from_node)?;
            let selectors =
                if upstream_selection_targets.is_empty() && downstream_selection_roots.is_empty() {
                    parse_selectors(select, exclude)?
                } else {
                    SelectorSet::default()
                };
            let named_resource_capacities = parse_resource_capacities(resource_capacity)?;
            let preview_layout = resolve_plan_preview_layout(out.as_deref(), run_id.as_deref())?;
            let preview = PlanPreviewConfig {
                run_root: out.clone(),
                run_id: preview_layout.as_ref().map(|layout| layout.run_id.clone()),
                cache_dir: cache_dir.clone(),
                absolute_path_policy: (*absolute_path_policy).into(),
                jobs: *jobs,
                cpu_budget: *cpu_budget,
                memory_budget_mb: *memory_budget_mb,
                gpu_device_budget: *gpu_device_budget,
                named_resource_capacities,
                upstream_selection_targets,
                downstream_selection_roots,
                selectors,
                dependency_closure: *dependency_closure,
            };
            let result =
                build_default_planner_analysis(&graph, &preview).map_err(|_| ExitCode::from(3))?;
            let lines = concise_plan_lines(&result);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.explain",
                    true,
                    plan_explain_payload(
                        &result,
                        preview_layout.as_ref(),
                        preview.absolute_path_policy,
                    ),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for line in lines {
                println!("{line}");
            }
            Ok(ExitCode::SUCCESS)
        }
        PlanCommands::Diagnostics { dags } => {
            let graph = load_graphs_or_emit(cli, "dag.plan.diagnostics", dags)?;
            match build_default_planner_analysis(&graph, &PlanPreviewConfig::default()) {
                Ok(result) => {
                    let payload = serde_json::to_value(&result.plan.diagnostics)
                        .map_err(|_| ExitCode::from(3))?;
                    let has_diagnostics = !result.plan.diagnostics.is_empty();
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.plan.diagnostics",
                            !has_diagnostics,
                            json!({
                                "diagnostics": payload,
                                "plan_fingerprint": result.plan_fingerprint,
                                "execution_cost_estimate": result.execution_cost_estimate,
                                "priority_inheritance": result.priority_inheritance,
                            }),
                            Vec::new(),
                            if has_diagnostics { ExitCode::from(3) } else { ExitCode::SUCCESS },
                        );
                    }
                    if !has_diagnostics {
                        println!("planner diagnostics: none");
                    } else {
                        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                        return Err(ExitCode::from(3));
                    }
                    Ok(ExitCode::SUCCESS)
                }
                Err(error) => {
                    let payload = json!([{
                        "id": "planner_analysis_failed",
                        "severity": "error",
                        "message": error,
                        "node_id": serde_json::Value::Null,
                    }]);
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.plan.diagnostics",
                            false,
                            json!({"diagnostics": payload}),
                            Vec::new(),
                            ExitCode::from(3),
                        );
                    }
                    println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                    Err(ExitCode::from(3))
                }
            }
        }
        PlanCommands::Diff { before, after } => {
            let before_input = read_file(before)?;
            let before_graph = parse_graph(&before_input)?;
            let before_result =
                build_default_planner_analysis(&before_graph, &PlanPreviewConfig::default())
                    .map_err(|_| ExitCode::from(3))?;

            let after_input = read_file(after)?;
            let after_graph = parse_graph(&after_input)?;
            let after_result =
                build_default_planner_analysis(&after_graph, &PlanPreviewConfig::default())
                    .map_err(|_| ExitCode::from(3))?;

            let diff = diff_plans(&before_result, &after_result);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.diff",
                    true,
                    plan_diff_payload(&before_result, &after_result, &diff),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if !plan_diff_changed(&diff) {
                println!("plan diff: no semantic planner differences");
            } else {
                if diff.metadata_only_changed {
                    println!("metadata_only_changed: true");
                }
                if diff.execution_affecting_changed {
                    println!("execution_affecting_changed: true");
                }
                if !diff.added_nodes.is_empty() {
                    println!("added_nodes: {}", diff.added_nodes.join(", "));
                }
                if !diff.removed_nodes.is_empty() {
                    println!("removed_nodes: {}", diff.removed_nodes.join(", "));
                }
                if !diff.changed_params.is_empty() {
                    println!("changed_params: {}", diff.changed_params.join(", "));
                }
                if !diff.changed_outputs.is_empty() {
                    println!("changed_outputs: {}", diff.changed_outputs.join(", "));
                }
                if !diff.changed_resources.is_empty() {
                    println!("changed_resources: {}", diff.changed_resources.join(", "));
                }
                if !diff.changed_retry_timeout.is_empty() {
                    println!("changed_retry_timeout: {}", diff.changed_retry_timeout.join(", "));
                }
                if !diff.added_dependencies.is_empty() {
                    println!("added_dependencies: {}", diff.added_dependencies.join(", "));
                }
                if !diff.removed_dependencies.is_empty() {
                    println!("removed_dependencies: {}", diff.removed_dependencies.join(", "));
                }
                if !diff.changed_metadata.is_empty() {
                    println!("changed_metadata: {}", diff.changed_metadata.join(", "));
                }
            }
            Ok(ExitCode::SUCCESS)
        }
        PlanCommands::Equivalence { before, after } => {
            let before_input = read_file(before)?;
            let before_graph = parse_graph(&before_input)?;
            let before_result =
                build_default_planner_analysis(&before_graph, &PlanPreviewConfig::default())
                    .map_err(|_| ExitCode::from(3))?;

            let after_input = read_file(after)?;
            let after_graph = parse_graph(&after_input)?;
            let after_result =
                build_default_planner_analysis(&after_graph, &PlanPreviewConfig::default())
                    .map_err(|_| ExitCode::from(3))?;

            let report = compare_plan_equivalence(&before_result, &after_result);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.equivalence",
                    true,
                    plan_equivalence_payload(&before_result, &after_result, &report),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("equivalent: {}", report.equivalent);
            println!("equivalence_class: {}", equivalence_class_label(&report));
            println!("summary: {}", report.summary);
            if !report.ignored_non_execution_drift.is_empty() {
                println!(
                    "ignored_non_execution_drift: {}",
                    report.ignored_non_execution_drift.join(", ")
                );
            }
            if !report.non_equivalence_causes.is_empty() {
                println!("non_equivalence_causes: {}", report.non_equivalence_causes.join(", "));
            }
            Ok(ExitCode::SUCCESS)
        }
        PlanCommands::Closure { dags, select } => {
            if select.is_empty() {
                return Err(ExitCode::from(2));
            }
            let graph = load_graphs_or_emit(cli, "dag.plan.closure", dags)?;
            let result = build_default_planner_analysis(&graph, &PlanPreviewConfig::default())
                .map_err(|_| ExitCode::from(3))?;
            let closure =
                compute_partial_run_closure(&result.plan, select).into_iter().collect::<Vec<_>>();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.closure",
                    true,
                    json!({
                        "selected_nodes": select,
                        "closure": closure,
                        "plan_fingerprint": result.plan_fingerprint,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for node_id in &closure {
                println!("{node_id}");
            }
            Ok(ExitCode::SUCCESS)
        }
        PlanCommands::Backfill { window_start_unix_ms, window_end_unix_ms, partition_key } => {
            let plan = build_backfill_plan(
                *window_start_unix_ms,
                *window_end_unix_ms,
                partition_key.clone(),
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.backfill",
                    true,
                    serde_json::to_value(&plan).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&plan).unwrap());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
#[path = "plan_routes_tests.rs"]
mod plan_routes_tests;
