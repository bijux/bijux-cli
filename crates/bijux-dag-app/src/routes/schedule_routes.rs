use crate::commands::{
    DagCli, ScheduleBackfillCommands, ScheduleCommands, ScheduleControlCommands,
    ScheduleQueueCommands,
};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{
    advance_backfill_operation, apply_backfill_throttling, apply_submission_status_updates,
    build_schedule_override_status, build_schedule_queue_state, cancel_backfill_operation,
    compile_backfill_operation, compile_submission_request, deduplicate_trigger_events,
    detect_cron_conflicts, dispatch_schedule_queue_runs, dry_run_schedule,
    evaluate_schedule_submissions_with_overrides, evaluate_sla_metrics, materialize_next_runs,
    pause_backfill_operation, pause_schedule, resume_backfill_operation, resume_schedule,
    retry_failed_backfill_runs, summarize_backfill_operation, validate_schedule_registry,
    weighted_priority_tie_break_order, BackfillAdvanceRequest, BackfillOperation,
    BackfillThrottlingPolicy, PriorityClass, ScheduleEvaluationInputs, ScheduleOverrideState,
    SchedulePriorityDispatchPolicy, ScheduleRegistry, ScheduleSubmissionLedger,
    ScheduleSubmissionStatusUpdateBatch, ScheduledSubmission, WeightedPriorityPolicy,
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

fn write_pretty_json<T: serde::Serialize>(
    path: &std::path::Path,
    value: &T,
) -> Result<(), ExitCode> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| ExitCode::from(3))?;
    std::fs::write(path, bytes).map_err(|_| ExitCode::from(3))
}

fn parse_json_file_or_default<T>(path: &std::path::Path) -> Result<T, ExitCode>
where
    T: serde::de::DeserializeOwned + Default,
{
    if path.exists() {
        parse_json_file(path)
    } else {
        Ok(T::default())
    }
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
        ScheduleCommands::Submit { registry, inputs, ledger, overrides, out } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let inputs: ScheduleEvaluationInputs = parse_json_file(inputs)?;
            let existing_ledger = if let Some(ledger_path) = ledger {
                parse_json_file(ledger_path)?
            } else {
                ScheduleSubmissionLedger::default()
            };
            let overrides = if let Some(overrides_path) = overrides {
                parse_json_file_or_default(overrides_path)?
            } else {
                ScheduleOverrideState::default()
            };
            let report = evaluate_schedule_submissions_with_overrides(
                &registry,
                &inputs,
                &existing_ledger,
                &overrides,
            );
            let updated_ledger =
                ScheduleSubmissionLedger { entries: report.recorded_submissions.clone() };
            if let Some(out_path) = out {
                write_pretty_json(out_path, &updated_ledger)?;
            }
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.submit",
                    true,
                    json!({
                        "generated_requests": report.generated_requests,
                        "recorded_submissions": report.recorded_submissions,
                        "duplicate_suppressions": report.duplicate_suppressions,
                        "paused_suppressions": report.paused_suppressions,
                        "audits": report.audits,
                        "written_ledger": out,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("generated_submissions={}", report.generated_requests.len());
            println!("duplicate_suppressions={}", report.duplicate_suppressions.len());
            println!("paused_suppressions={}", report.paused_suppressions.len());
            if let Some(out_path) = out {
                println!("written_ledger={}", out_path.display());
            }
            Ok(ExitCode::SUCCESS)
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
            let request = match compile_submission_request(definition, *requested_unix_ms) {
                Ok(request) => request,
                Err(error) => {
                    if cli.json {
                        return emit_json(
                            cli,
                            "dag.schedule.compile",
                            false,
                            json!({}),
                            vec![json!({
                                "id": "schedule_compile_invalid",
                                "severity": "error",
                                "message": error,
                            })],
                            ExitCode::from(3),
                        );
                    }
                    println!("{error}");
                    return Err(ExitCode::from(3));
                }
            };
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
        ScheduleCommands::Queue { command } => handle_schedule_queue_command(cli, command),
        ScheduleCommands::Control { command } => handle_schedule_control_command(cli, command),
        ScheduleCommands::Backfill { command } => handle_schedule_backfill_command(cli, command),
    }
}

fn maybe_write_submission_ledger(
    out: &Option<std::path::PathBuf>,
    ledger: &ScheduleSubmissionLedger,
) -> Result<(), ExitCode> {
    if let Some(path) = out {
        write_pretty_json(path, ledger)?;
    }
    Ok(())
}

fn maybe_write_schedule_overrides(
    out: &Option<std::path::PathBuf>,
    overrides: &ScheduleOverrideState,
) -> Result<(), ExitCode> {
    if let Some(path) = out {
        write_pretty_json(path, overrides)?;
    }
    Ok(())
}

fn parse_backfill_operation(path: &std::path::Path) -> Result<BackfillOperation, ExitCode> {
    parse_json_file(path)
}

fn maybe_write_backfill_operation(
    out: &Option<std::path::PathBuf>,
    operation: &BackfillOperation,
) -> Result<(), ExitCode> {
    if let Some(path) = out {
        write_pretty_json(path, operation)?;
    }
    Ok(())
}

fn maybe_write_backfill_summary(
    out: &Option<std::path::PathBuf>,
    operation: &BackfillOperation,
) -> Result<(), ExitCode> {
    if let Some(path) = out {
        let summary = summarize_backfill_operation(operation);
        write_pretty_json(path, &summary)?;
    }
    Ok(())
}

fn handle_schedule_queue_command(
    cli: &DagCli,
    command: &ScheduleQueueCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ScheduleQueueCommands::Status { registry, ledger, out } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let ledger = if let Some(ledger_path) = ledger {
                parse_json_file(ledger_path)?
            } else {
                ScheduleSubmissionLedger::default()
            };
            let queue_state =
                build_schedule_queue_state(&registry, &ledger).map_err(|_| ExitCode::from(3))?;
            if let Some(path) = out {
                write_pretty_json(path, &queue_state)?;
            }
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.queue.status",
                    true,
                    json!({
                        "queue_state": queue_state,
                        "written_state": out,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&queue_state).unwrap());
            if let Some(path) = out {
                println!("written_queue_state={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        ScheduleQueueCommands::Dispatch { ledger, max_dispatches, policy, out } => {
            let mut ledger: ScheduleSubmissionLedger = parse_json_file(ledger)?;
            let policy = if let Some(policy_path) = policy {
                parse_json_file(policy_path)?
            } else {
                SchedulePriorityDispatchPolicy::default()
            };
            let report = dispatch_schedule_queue_runs(&mut ledger, *max_dispatches, &policy);
            maybe_write_submission_ledger(out, &ledger)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.queue.dispatch",
                    true,
                    json!({
                        "dispatch_report": report,
                        "updated_ledger": ledger,
                        "written_ledger": out,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if let Some(path) = out {
                println!("written_ledger={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        ScheduleQueueCommands::Update { ledger, updates, out } => {
            let mut ledger: ScheduleSubmissionLedger = parse_json_file(ledger)?;
            let updates: ScheduleSubmissionStatusUpdateBatch = parse_json_file(updates)?;
            apply_submission_status_updates(&mut ledger, &updates.updates)
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_submission_ledger(out, &ledger)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.queue.update",
                    true,
                    json!({
                        "updated_ledger": ledger,
                        "written_ledger": out,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&ledger).unwrap());
            if let Some(path) = out {
                println!("written_ledger={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn handle_schedule_control_command(
    cli: &DagCli,
    command: &ScheduleControlCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ScheduleControlCommands::Status { registry, overrides, out } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let overrides = if let Some(overrides_path) = overrides {
                parse_json_file_or_default(overrides_path)?
            } else {
                ScheduleOverrideState::default()
            };
            let status = build_schedule_override_status(&registry, &overrides);
            if let Some(path) = out {
                write_pretty_json(path, &status)?;
            }
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.control.status",
                    true,
                    json!({
                        "schedule_status": status,
                        "written_status": out,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&status).unwrap());
            if let Some(path) = out {
                println!("written_status={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        ScheduleControlCommands::Pause {
            overrides,
            schedule_id,
            operator,
            at_unix_ms,
            reason,
            out,
        } => {
            let mut overrides: ScheduleOverrideState = parse_json_file_or_default(overrides)?;
            pause_schedule(&mut overrides, schedule_id, operator, *at_unix_ms, reason.clone())
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_schedule_overrides(out, &overrides)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.control.pause",
                    true,
                    serde_json::to_value(&overrides).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&overrides).unwrap());
            if let Some(path) = out {
                println!("written_overrides={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
        ScheduleControlCommands::Resume {
            overrides,
            schedule_id,
            operator,
            at_unix_ms,
            reason,
            out,
        } => {
            let mut overrides: ScheduleOverrideState = parse_json_file_or_default(overrides)?;
            resume_schedule(&mut overrides, schedule_id, operator, *at_unix_ms, reason.clone())
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_schedule_overrides(out, &overrides)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.control.resume",
                    true,
                    serde_json::to_value(&overrides).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&overrides).unwrap());
            if let Some(path) = out {
                println!("written_overrides={}", path.display());
            }
            Ok(ExitCode::SUCCESS)
        }
    }
}

fn handle_schedule_backfill_command(
    cli: &DagCli,
    command: &ScheduleBackfillCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        ScheduleBackfillCommands::Plan {
            registry,
            schedule_id,
            planned_unix_ms,
            backfill_id,
            out,
        } => {
            let registry = parse_schedule_registry(registry)?;
            validate_schedule_registry(&registry).map_err(|_| ExitCode::from(3))?;
            let definition = registry
                .definitions
                .iter()
                .find(|definition| definition.id == *schedule_id)
                .ok_or_else(|| ExitCode::from(3))?;
            let operation =
                compile_backfill_operation(definition, backfill_id.as_deref(), *planned_unix_ms)
                    .map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.plan",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Status { state } => {
            let operation = parse_backfill_operation(state)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.status",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Summary { state, out } => {
            let operation = parse_backfill_operation(state)?;
            let summary = summarize_backfill_operation(&operation);
            maybe_write_backfill_summary(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.summary",
                    true,
                    serde_json::to_value(&summary).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&summary).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Advance { state, request, out } => {
            let operation = parse_backfill_operation(state)?;
            let request: BackfillAdvanceRequest = parse_json_file(request)?;
            let report =
                advance_backfill_operation(&operation, &request).map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &report.operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.advance",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Pause { state, at_unix_ms, reason, out } => {
            let mut operation = parse_backfill_operation(state)?;
            pause_backfill_operation(&mut operation, *at_unix_ms, reason.clone())
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.pause",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Resume { state, at_unix_ms, out } => {
            let mut operation = parse_backfill_operation(state)?;
            resume_backfill_operation(&mut operation, *at_unix_ms)
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.resume",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::RetryFailed { state, at_unix_ms, out } => {
            let mut operation = parse_backfill_operation(state)?;
            retry_failed_backfill_runs(&mut operation, *at_unix_ms)
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.retry-failed",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        ScheduleBackfillCommands::Cancel { state, at_unix_ms, reason, out } => {
            let mut operation = parse_backfill_operation(state)?;
            cancel_backfill_operation(&mut operation, *at_unix_ms, reason.clone())
                .map_err(|_| ExitCode::from(3))?;
            maybe_write_backfill_operation(out, &operation)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.schedule.backfill.cancel",
                    true,
                    serde_json::to_value(&operation).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&operation).unwrap());
            Ok(ExitCode::SUCCESS)
        }
    }
}

#[cfg(test)]
#[path = "schedule_routes_tests.rs"]
mod schedule_routes_tests;
