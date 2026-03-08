use crate::commands::{DagCli, PlanCommands};
use crate::{
    emit_json, lower_graph_to_execution_plan, parse_graph, planner_diagnostics_from_error,
    read_file, ExitCode, PlanOptions,
};
use serde_json::json;

pub(crate) fn handle_plan_command(
    cli: &DagCli,
    command: &PlanCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        PlanCommands::Explain { dag } => {
            let input = read_file(dag)?;
            let graph = parse_graph(&input)?;
            let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default())
                .map_err(|_| ExitCode::from(3))?;
            let mut lines = Vec::new();
            for node in &plan.nodes {
                if node.deps.is_empty() {
                    lines.push(format!(
                        "{}: included as graph root (no dependencies)",
                        node.id
                    ));
                } else {
                    lines.push(format!(
                        "{}: included because it depends on {}",
                        node.id,
                        node.deps.join(", ")
                    ));
                }
            }
            if cli.json {
                return emit_json(
                    cli,
                    "dag.plan.explain",
                    true,
                    json!({
                        "graph_fingerprint": plan.graph_fingerprint,
                        "planner_fingerprint": plan.planner_fingerprint,
                        "ordering": plan.ordering,
                        "nodes": plan.nodes.iter().map(|node| {
                            json!({
                                "node_id": node.id,
                                "reason": if node.deps.is_empty() {
                                    "graph root (no dependencies)"
                                } else {
                                    "dependency closure"
                                },
                                "dependencies": node.deps,
                            })
                        }).collect::<Vec<_>>()
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
            match lower_graph_to_execution_plan(&graph, PlanOptions::default()) {
                Ok(plan) => {
                    let payload =
                        serde_json::to_value(&plan.diagnostics).map_err(|_| ExitCode::from(3))?;
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.plan.diagnostics",
                            true,
                            json!({"diagnostics": payload}),
                            Vec::new(),
                            ExitCode::SUCCESS,
                        );
                    }
                    if plan.diagnostics.is_empty() {
                        println!("planner diagnostics: none");
                    } else {
                        println!("{}", serde_json::to_string_pretty(&payload).unwrap());
                    }
                    Ok(ExitCode::SUCCESS)
                }
                Err(error) => {
                    let diagnostics = planner_diagnostics_from_error(&error);
                    let payload =
                        serde_json::to_value(&diagnostics).map_err(|_| ExitCode::from(3))?;
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
    }
}
