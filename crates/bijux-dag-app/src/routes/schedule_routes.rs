use crate::commands::{DagCli, ScheduleCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{
    apply_backfill_throttling, compile_submission_request, deduplicate_trigger_events,
    detect_cron_conflicts, dry_run_schedule, evaluate_sla_metrics, materialize_next_runs,
    validate_schedule_registry, weighted_priority_tie_break_order, BackfillThrottlingPolicy,
    PriorityClass, ScheduleRegistry, ScheduledSubmission, WeightedPriorityPolicy,
};
use serde_json::json;
use std::collections::BTreeMap;

#[derive(Debug, serde::Deserialize)]
struct ScheduleOrderingSimulation {
    submissions: Vec<ScheduledSubmission>,
    priorities: BTreeMap<String, PriorityClass>,
    policy: WeightedPriorityPolicy,
}

#[derive(Debug, serde::Deserialize)]
struct BackfillThrottlingSimulation {
    pending_backfill_runs: usize,
    pending_live_runs: usize,
    policy: BackfillThrottlingPolicy,
}

#[derive(Debug, serde::Deserialize)]
struct TriggerDedupSimulation {
    events: Vec<String>,
}

#[derive(Debug, serde::Deserialize)]
struct SlaSample {
    observed_ms: u64,
    expected_ms: u64,
}

#[derive(Debug, serde::Deserialize)]
struct SlaSimulation {
    start_samples: Vec<SlaSample>,
    finish_samples: Vec<SlaSample>,
    queue_saturation_count: u64,
    fairness_drift_count: u64,
}

fn parse_schedule_registry(path: &std::path::Path) -> Result<ScheduleRegistry, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T, ExitCode> {
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
        ScheduleCommands::Compile { registry, schedule_id, requested_unix_ms } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let definition = registry
                .definitions
                .iter()
                .find(|definition| definition.id == *schedule_id)
                .ok_or_else(|| ExitCode::from(3))?;
            let request = compile_submission_request(definition, *requested_unix_ms);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.compile",
                    true,
                    serde_json::to_value(&request).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&request).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleCommands::Audit { registry, now_unix_ms, next_runs } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let conflicts = detect_cron_conflicts(&registry.definitions);
            let materialized = registry
                .definitions
                .iter()
                .map(|definition| materialize_next_runs(definition, *now_unix_ms, *next_runs))
                .collect::<Vec<_>>();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.audit",
                    true,
                    json!({
                        "conflicts": conflicts,
                        "materialized_runs": materialized,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "conflicts": conflicts,
                    "materialized_runs": materialized,
                }))
                .unwrap()
            );
            Ok(ExitCode::SUCCESS)
        }
        ScheduleCommands::Dedup { events } => {
            let simulation: TriggerDedupSimulation = parse_json_file(events)?;
            let decisions = deduplicate_trigger_events(&simulation.events);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.dedup",
                    true,
                    json!({ "decisions": decisions }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&decisions).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleCommands::Sla { simulation } => {
            let simulation: SlaSimulation = parse_json_file(simulation)?;
            let start_samples = simulation
                .start_samples
                .iter()
                .map(|sample| (sample.observed_ms as u128, sample.expected_ms as u128))
                .collect::<Vec<_>>();
            let finish_samples = simulation
                .finish_samples
                .iter()
                .map(|sample| (sample.observed_ms as u128, sample.expected_ms as u128))
                .collect::<Vec<_>>();
            let metrics = evaluate_sla_metrics(
                &start_samples,
                &finish_samples,
                simulation.queue_saturation_count,
                simulation.fairness_drift_count,
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.sla",
                    true,
                    serde_json::to_value(&metrics).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&metrics).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleCommands::Order { simulation } => {
            let simulation: ScheduleOrderingSimulation = parse_json_file(simulation)?;
            let ordered = weighted_priority_tie_break_order(
                simulation.submissions,
                &simulation.priorities,
                &simulation.policy,
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.order",
                    true,
                    json!({ "ordered": ordered }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&ordered).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleCommands::Throttle { simulation } => {
            let simulation: BackfillThrottlingSimulation = parse_json_file(simulation)?;
            let (allowed_backfill_runs, pending_live_runs) = apply_backfill_throttling(
                simulation.pending_backfill_runs,
                simulation.pending_live_runs,
                &simulation.policy,
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.throttle",
                    true,
                    json!({
                        "allowed_backfill_runs": allowed_backfill_runs,
                        "pending_live_runs": pending_live_runs,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!(
                "allowed_backfill_runs={} pending_live_runs={}",
                allowed_backfill_runs, pending_live_runs
            );
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
#[path = "schedule_routes_tests.rs"]
mod schedule_routes_tests;
