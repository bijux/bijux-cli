use crate::commands::{DagCli, PlanCommands};
use crate::{
    emit_json, lower_graph_to_execution_plan, parse_graph, planner_diagnostics_from_error,
    read_file, ExitCode, PlanOptions,
};
use serde_json::json;

fn concise_plan_lines(plan: &bijux_dag_core::ExecutionPlan) -> Vec<String> {
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
    lines
}

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
            let lines = concise_plan_lines(&plan);
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

#[cfg(test)]
mod tests {
    use super::handle_plan_command;
    use crate::commands::{Commands, DagCli, PlanCommands};
    use crate::ExitCode;
    use std::fs;
    use std::path::PathBuf;

    fn quiet_json_cli() -> DagCli {
        DagCli {
            json: true,
            quiet: true,
            command: Commands::Version,
        }
    }

    fn write_graph_fixture() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"plan-routes","owners":[],"tags":[]},
              "nodes":[
                {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
                {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
              ],
              "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
            }"#,
        )
        .expect("write graph");
        (dir, dag)
    }

    fn write_invalid_graph_fixture() -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph-invalid.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"bad-plan","owners":[],"tags":[]},
              "nodes":[
                {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/out"}],"params":{"value":"1"}},
                {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"same/out"}],"params":{"value":"2"}}
              ],
              "edges":[]
            }"#,
        )
        .expect("write invalid graph");
        (dir, dag)
    }

    #[test]
    fn plan_explain_success_path_returns_success() {
        let (_tmp, dag) = write_graph_fixture();
        let cli = quiet_json_cli();
        let code = handle_plan_command(&cli, &PlanCommands::Explain { dag }).expect("plan explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn plan_diagnostics_success_path_returns_success() {
        let (_tmp, dag) = write_graph_fixture();
        let cli = quiet_json_cli();
        let code =
            handle_plan_command(&cli, &PlanCommands::Diagnostics { dag }).expect("plan diagnostics");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn plan_command_rejects_malformed_input_without_panic() {
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| {
            handle_plan_command(
                &cli,
                &PlanCommands::Explain {
                    dag: PathBuf::from("/no/such/graph.json"),
                },
            )
        });
        assert!(result.is_ok(), "plan route should not panic on malformed input");
        assert!(result.expect("result").is_err());
    }

    #[test]
    fn plan_diagnostics_rejects_malformed_input_without_panic() {
        let cli = quiet_json_cli();
        let result = std::panic::catch_unwind(|| {
            handle_plan_command(
                &cli,
                &PlanCommands::Diagnostics {
                    dag: PathBuf::from("/no/such/graph.json"),
                },
            )
        });
        assert!(result.is_ok(), "plan diagnostics should not panic on malformed input");
        assert!(result.expect("result").is_err());
    }

    #[test]
    fn plan_concise_human_snapshot_is_stable() {
        let expected = "a: included as graph root (no dependencies)\n\
b: included because it depends on a";
        let (_tmp, dag) = write_graph_fixture();
        let raw = fs::read_to_string(dag).expect("read graph");
        let graph = crate::parse_graph(&raw).expect("graph");
        let plan = crate::lower_graph_to_execution_plan(&graph, crate::PlanOptions::default())
            .expect("plan");
        let rendered = super::concise_plan_lines(&plan).join("\n");
        assert_eq!(rendered, expected);
    }

    #[test]
    fn plan_routes_support_replay_and_imported_bundle_shaped_graphs() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"imported-replay","owners":[],"tags":["imported","replay"]},
              "nodes":[{"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"out"}],"params":{"value":"seed"}}],
              "edges":[]
            }"#,
        )
        .expect("write graph");
        let cli = quiet_json_cli();
        let code = handle_plan_command(&cli, &PlanCommands::Explain { dag }).expect("plan explain");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn plan_diagnostics_error_flow_is_stable_for_invalid_graph() {
        let (_tmp, dag) = write_invalid_graph_fixture();
        let cli = quiet_json_cli();
        let code = handle_plan_command(&cli, &PlanCommands::Diagnostics { dag }).unwrap_err();
        assert_eq!(code, ExitCode::from(3));
    }

    #[test]
    fn plan_explain_dump_flow_is_stable_for_valid_graph() {
        let (_tmp, dag) = write_graph_fixture();
        let raw = fs::read_to_string(dag).expect("read graph");
        let graph = crate::parse_graph(&raw).expect("graph");
        let plan = crate::lower_graph_to_execution_plan(&graph, crate::PlanOptions::default())
            .expect("plan");
        assert!(!plan.ordering.is_empty(), "plan ordering should not be empty");
    }
}
