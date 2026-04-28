use crate::commands::{DagCli, ScheduleCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{
    dry_run_schedule, materialize_next_runs, validate_schedule_registry, ScheduleRegistry,
};
use serde_json::json;

fn parse_schedule_registry(path: &std::path::Path) -> Result<ScheduleRegistry, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

pub(crate) fn handle_schedule_command(
    cli: &DagCli,
    command: &ScheduleCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ScheduleCommands::Validate { registry } => {
            let registry = parse_schedule_registry(registry)?;
            match validate_schedule_registry(&registry) {
                Ok(()) => {
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.schedule.validate",
                            true,
                            json!({ "schedule_count": registry.definitions.len() }),
                            Vec::new(),
                            ExitCode::SUCCESS,
                        );
                    }
                    println!("schedule registry: ok ({})", registry.definitions.len());
                    Ok(ExitCode::SUCCESS)
                }
                Err(error) => {
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.schedule.validate",
                            false,
                            json!({}),
                            vec![json!({
                                "id": "schedule_registry_invalid",
                                "severity": "error",
                                "message": error,
                            })],
                            ExitCode::from(3),
                        );
                    }
                    println!("{error}");
                    Err(ExitCode::from(3))
                }
            }
        }
        ScheduleCommands::Preview { registry, now_unix_ms, next_runs } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let previews = registry
                .definitions
                .iter()
                .map(|definition| {
                    json!({
                        "schedule_id": definition.id,
                        "preview": dry_run_schedule(definition, *now_unix_ms),
                        "materialized_runs": materialize_next_runs(definition, *now_unix_ms, *next_runs),
                    })
                })
                .collect::<Vec<_>>();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.preview",
                    true,
                    json!({ "previews": previews }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            for preview in previews {
                println!("{}", serde_json::to_string_pretty(&preview).unwrap());
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
#[path = "schedule_routes_tests.rs"]
mod schedule_routes_tests;
