use crate::commands::{DagCli, PlanCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_runtime::{
    build_backfill_plan, build_planner_analysis, compute_partial_run_closure, diff_plans,
    explain_plan, PlannerBuildResult, PlannerGuardrails, RuntimeConfig,
};
use serde_json::json;

fn default_analysis_runtime_config() -> RuntimeConfig {
    RuntimeConfig::default()
}

fn build_default_planner_analysis(
    graph: &bijux_dag_core::Graph,
) -> Result<PlannerBuildResult, String> {
    let config = default_analysis_runtime_config();
    build_planner_analysis(
        graph,
        &config,
        &config.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
}

fn concise_plan_lines(result: &PlannerBuildResult) -> Vec<String> {
    result
        .annotations
        .iter()
        .map(|annotation| {
            let queue_hint = annotation
                .queue_hint
                .as_deref()
                .unwrap_or("default");
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

pub(crate) fn handle_plan_command(
    cli: &DagCli,
    command: &PlanCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PlanCommands::Explain { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let result = build_default_planner_analysis(&graph).map_err(|_| ExitCode::from(3))?;
            let report = explain_plan(&result);
            let lines = concise_plan_lines(&result);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.explain",
                    true,
                    json!({
                        "plan_fingerprint": report.plan_fingerprint,
                        "phases": report.phases,
                        "ordering": result.plan.order,
                        "resource_estimate": result.resource_estimate,
                        "priority_inheritance": result.priority_inheritance,
                        "optimization_notes": report.optimization_notes,
                        "nodes": report.annotations,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for line in lines {
                println!("{line}");
            }
            Ok(ExitCode::SUCCESS)
        }
        PlanCommands::Diagnostics { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            match build_default_planner_analysis(&graph) {
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
                build_default_planner_analysis(&before_graph).map_err(|_| ExitCode::from(3))?;

            let after_input = read_file(after)?;
            let after_graph = parse_graph(&after_input)?;
            let after_result =
                build_default_planner_analysis(&after_graph).map_err(|_| ExitCode::from(3))?;

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
        PlanCommands::Closure { dag, select } => {
            if select.is_empty() {
                return Err(ExitCode::from(2));
            }
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let result = build_default_planner_analysis(&graph).map_err(|_| ExitCode::from(3))?;
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
