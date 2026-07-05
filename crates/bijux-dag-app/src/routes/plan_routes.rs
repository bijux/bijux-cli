use crate::commands::{AbsolutePathPolicyArg, DagCli, PlanCommands};
use crate::routes::preconditions::require_safe_path;
use crate::{emit_json, load_graphs_or_emit, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::RunDirLayout;
use bijux_dag_runtime::{
    build_backfill_plan, build_planner_analysis, compute_partial_run_closure, diff_plans,
    explain_plan, AbsolutePathPolicy, PlannerBuildResult, PlannerGuardrails, RuntimeConfig,
};
use serde_json::json;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub(crate) struct PlanPreviewConfig {
    pub run_root: Option<PathBuf>,
    pub run_id: Option<String>,
    pub cache_dir: Option<PathBuf>,
    pub absolute_path_policy: AbsolutePathPolicy,
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
        run_root: preview.run_root.clone(),
        run_id: preview.run_id.clone(),
        cache_dir: preview.cache_dir.clone(),
        absolute_path_policy: preview.absolute_path_policy,
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
    json!({
        "planner_contract_version": result.plan.planner_contract_version,
        "plan_fingerprint": report.plan_fingerprint,
        "run_layout": preview_layout,
        "absolute_path_policy": absolute_path_policy,
        "phases": report.phases,
        "ordering": result.plan.order,
        "resource_estimate": result.resource_estimate,
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

pub(crate) fn handle_plan_command(
    cli: &DagCli,
    command: &PlanCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PlanCommands::Explain { dags, out, run_id, cache_dir, absolute_path_policy } => {
            let graph = load_graphs_or_emit(cli, "dag.plan.explain", dags)?;
            let preview_layout = resolve_plan_preview_layout(out.as_deref(), run_id.as_deref())?;
            let preview = PlanPreviewConfig {
                run_root: out.clone(),
                run_id: preview_layout.as_ref().map(|layout| layout.run_id.clone()),
                cache_dir: cache_dir.clone(),
                absolute_path_policy: (*absolute_path_policy).into(),
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
                                "resource_estimate": result.resource_estimate,
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
            let changed = !diff.changed_order_nodes.is_empty()
                || !diff.changed_filter_reasons.is_empty()
                || !diff.changed_annotations.is_empty();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.diff",
                    true,
                    json!({
                        "changed": changed,
                        "before_plan_fingerprint": before_result.plan_fingerprint,
                        "after_plan_fingerprint": after_result.plan_fingerprint,
                        "before_resource_estimate": before_result.resource_estimate,
                        "after_resource_estimate": after_result.resource_estimate,
                        "diff": diff,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            if !changed {
                println!("plan diff: no semantic planner differences");
            } else {
                if !diff.changed_order_nodes.is_empty() {
                    println!("changed_order_nodes: {}", diff.changed_order_nodes.join(", "));
                }
                if !diff.changed_filter_reasons.is_empty() {
                    println!("changed_filter_reasons: {}", diff.changed_filter_reasons.join(", "));
                }
                if !diff.changed_annotations.is_empty() {
                    println!("changed_annotations: {}", diff.changed_annotations.join(", "));
                }
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
