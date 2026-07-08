use crate::commands::{DagCli, RuntimeCommands};
use crate::repair_service::{
    apply_run_repair, plan_run_repair, run_repair_ok, RepairExecutionOptions,
};
use crate::routes::policy_surface::policy_surface_payload;
use crate::routes::preconditions::require_safe_path;
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_artifacts::{NodeTrace, RunOutputsIndex};
use bijux_dag_runtime::simulated_platform::RemoteStatusEvent;
use bijux_dag_runtime::{
    audit_dispatch_discipline, audit_run_event_log, build_cancellation_audit_report,
    build_execution_isolation_report, build_heartbeat_audit_report,
    build_manual_intervention_audit_report, build_pause_resume_audit_report,
    build_retry_decision_report, build_timeout_audit_report, build_transition_audit_report,
    check_run_consistency, detect_stuck_run, evaluate_pause_state,
    execute_remote_payload_in_place, reconcile_orphaned_node, should_quarantine_run,
    validate_and_repair_run_metadata, BatchAttemptState, BatchLifecycleEvent,
    DispatchKeyRecord, InterruptionClass, ManualInterventionRecord, MockRemoteWorker, NodeState,
    NodeTransition, OperatorRetryPolicy, RemoteNodeExecutionPayload, RemoteWorkerExecutor,
    ResumePolicy, RunPausePolicy, RunSnapshot, RunState, RunSummaryV2, RunTransition,
    RuntimeConfig, SchedulerRecoveryRule, StuckRunPolicy, TaskIsolationMode,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct DispatchSimulation {
    #[serde(default)]
    dispatches: Vec<DispatchKeyRecord>,
    #[serde(default)]
    remote_status_events: Vec<RemoteStatusEvent>,
    #[serde(default)]
    batch_events: Vec<BatchLifecycleEvent>,
}

#[derive(Debug, Deserialize)]
struct HeartbeatSimulation {
    heartbeat: bijux_dag_runtime::simulated_platform::WorkerHeartbeat,
    now_unix_ms: u128,
    liveness_policy: bijux_dag_runtime::simulated_platform::LivenessPolicy,
    heartbeat_semantics: bijux_dag_runtime::simulated_platform::HeartbeatSemantics,
    #[serde(default)]
    lease: Option<bijux_dag_runtime::simulated_platform::WorkLease>,
    #[serde(default)]
    lease_semantics: Option<bijux_dag_runtime::simulated_platform::TaskLeaseSemantics>,
}

#[derive(Debug, Deserialize)]
struct CancellationSimulation {
    isolation_mode: TaskIsolationMode,
    issued_unix_ms: u128,
    delivered_unix_ms: u128,
    deadline_ms: u64,
    #[serde(default)]
    batch_state: Option<BatchAttemptState>,
}

#[derive(Debug, Deserialize)]
struct PauseSimulation {
    policy: RunPausePolicy,
    queued_count: usize,
    ready_count: usize,
    running_count: usize,
    interruption_class: InterruptionClass,
    resume_policy: ResumePolicy,
}

#[derive(Debug, Deserialize)]
struct InterventionSimulation {
    record: ManualInterventionRecord,
    policy: OperatorRetryPolicy,
    manual_attempts_so_far: u32,
    #[serde(default)]
    lineage_recorded: bool,
    #[serde(default)]
    required_artifacts: Vec<String>,
    #[serde(default)]
    present_artifacts: Vec<String>,
    #[serde(default)]
    allow_missing_required_artifacts: bool,
}

#[derive(Debug, Deserialize)]
struct TransitionSimulation {
    node_transitions: Vec<NodeTransition>,
    run_transitions: Vec<RunTransition>,
    final_run_state: RunState,
    final_node_states: Vec<NodeState>,
    causal_failure_count: usize,
}

#[derive(Debug, Deserialize)]
struct WorkerRecoverySimulation {
    rule: SchedulerRecoveryRule,
    has_checkpoint: bool,
    side_effect_uncertain: bool,
}

#[derive(Debug, Deserialize)]
struct ControlRecoverySimulation {
    now_unix_ms: u128,
    last_progress_unix_ms: u128,
    last_heartbeat_unix_ms: u128,
    stuck_policy: StuckRunPolicy,
    pause_policy: RunPausePolicy,
    queued_count: usize,
    ready_count: usize,
    running_count: usize,
    summary: RunSummaryV2,
    node_states: Vec<RecoveryNodeStateRecord>,
    artifact_nodes: Vec<String>,
    manifest_exists: bool,
    index_exists: bool,
    allow_repair: bool,
}

#[derive(Debug, Deserialize)]
struct RecoveryNodeStateRecord {
    node_id: String,
    state: NodeState,
}

#[derive(Debug, Serialize)]
struct DurableStateAuditReport {
    run_id: String,
    authoritative_ready: bool,
    durable_components: Vec<String>,
    missing_components: Vec<String>,
    node_trace_count: usize,
    checkpoint_present: bool,
    incomplete_marker_present: bool,
}

#[derive(Debug, Serialize)]
struct WriteDisciplineAuditReport {
    event_count: usize,
    index_entry_count: Option<usize>,
    index_in_sync: Option<bool>,
    duplicate_singleton_events: Vec<String>,
    duplicate_attempt_keys: Vec<String>,
    conflicting_node_terminal_events: Vec<String>,
    duplicate_output_paths: Vec<String>,
    exactly_once_ready: bool,
}

#[derive(Debug, Serialize)]
struct WorkerRecoveryAuditReport {
    orphaned_node_state: NodeState,
    recovery_action: String,
    recovered_state: NodeState,
    checkpoint_resume_possible: bool,
    manual_review_required: bool,
    recommended_action: String,
}

#[derive(Debug, Serialize)]
struct ControlRecoveryAuditReport {
    stuck_detected: bool,
    pause_state: BTreeMap<String, bool>,
    consistency_summary_matches_node_states: bool,
    all_success_nodes_have_artifacts: bool,
    mismatches: Vec<String>,
    quarantine_reason: Option<String>,
    repair_outcome: Value,
    recommended_action: String,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn read_json_value(path: &Path) -> Result<Value, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn parse_node_state_str(status: &str) -> Option<NodeState> {
    match status {
        "success" => Some(NodeState::Success),
        "succeeded" => Some(NodeState::Success),
        "failed" => Some(NodeState::Failed),
        "skipped" => Some(NodeState::Skipped),
        "cached" => Some(NodeState::Cached),
        "cancelled" => Some(NodeState::Cancelled),
        "timed_out" => Some(NodeState::TimedOut),
        "queued" => Some(NodeState::Queued),
        "running" => Some(NodeState::Running),
        "ready" => Some(NodeState::Eligible),
        "eligible" => Some(NodeState::Eligible),
        "pending" => Some(NodeState::Pending),
        _ => None,
    }
}

fn read_node_traces(run_dir: &Path) -> Result<Vec<NodeTrace>, ExitCode> {
    let mut traces = Vec::new();
    let nodes_dir = run_dir.join("nodes");
    if !nodes_dir.exists() {
        return Ok(traces);
    }
    let mut entries: Vec<_> =
        fs::read_dir(nodes_dir).map_err(|_| ExitCode::from(3))?.filter_map(|e| e.ok()).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let trace_path = entry.path().join("trace.json");
        if !trace_path.exists() {
            continue;
        }
        let trace: NodeTrace =
            serde_json::from_str(&read_file(&trace_path)?).map_err(|_| ExitCode::from(3))?;
        traces.push(trace);
    }
    Ok(traces)
}

fn read_run_outputs_index(run_dir: &Path) -> Result<RunOutputsIndex, ExitCode> {
    let raw = read_file(&run_dir.join("outputs").join("index.json"))?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn audit_durable_state(run_dir: &Path) -> Result<DurableStateAuditReport, ExitCode> {
    let manifest_path = run_dir.join("manifest.json");
    let graph_snapshot_path = run_dir.join("graph.snapshot.json");
    let run_snapshot_path = run_dir.join("run.snapshot.json");
    let run_log_path = run_dir.join("run.log.jsonl");
    let run_log_index_path = run_dir.join("run-log.index.json");
    let outputs_index_path = run_dir.join("outputs").join("index.json");
    let lineage_snapshot_path = run_dir.join("lineage.snapshot.json");
    let timeline_path = run_dir.join("observability.timeline.json");
    let checkpoint_path = run_dir.join("scheduler.checkpoint.json");
    let incomplete_marker_path = run_dir.join(".run-incomplete.json");

    let mut durable_components = Vec::new();
    let mut missing_components = Vec::new();
    for (name, path) in [
        ("manifest", manifest_path.as_path()),
        ("graph_snapshot", graph_snapshot_path.as_path()),
        ("run_snapshot", run_snapshot_path.as_path()),
        ("run_event_log", run_log_path.as_path()),
        ("run_event_index", run_log_index_path.as_path()),
        ("outputs_index", outputs_index_path.as_path()),
        ("lineage_snapshot", lineage_snapshot_path.as_path()),
        ("timeline", timeline_path.as_path()),
        ("scheduler_checkpoint", checkpoint_path.as_path()),
    ] {
        if path.exists() {
            durable_components.push(name.to_string());
        } else {
            missing_components.push(name.to_string());
        }
    }
    let run_id = if manifest_path.exists() {
        read_json_value(&manifest_path)?
            .get("run_id")
            .and_then(Value::as_str)
            .unwrap_or("unknown-run")
            .to_string()
    } else if run_snapshot_path.exists() {
        let snapshot: RunSnapshot =
            serde_json::from_str(&read_file(&run_snapshot_path)?).map_err(|_| ExitCode::from(3))?;
        snapshot.run_id.to_string()
    } else {
        run_dir
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("unknown-run")
            .trim_start_matches("run-")
            .to_string()
    };
    let node_trace_count = read_node_traces(run_dir)?.len();
    Ok(DurableStateAuditReport {
        run_id,
        authoritative_ready: missing_components.is_empty(),
        durable_components,
        missing_components,
        node_trace_count,
        checkpoint_present: checkpoint_path.exists(),
        incomplete_marker_present: incomplete_marker_path.exists(),
    })
}

fn audit_write_discipline(run_dir: &Path) -> Result<WriteDisciplineAuditReport, ExitCode> {
    let raw = read_file(&run_dir.join("run.log.jsonl"))?;
    let mut events = Vec::new();
    let mut duplicate_attempt_keys = BTreeSet::new();
    let mut seen_attempt_keys = BTreeSet::new();
    let mut terminal_statuses: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let value: Value = serde_json::from_str(line).map_err(|_| ExitCode::from(3))?;
        let name = value.get("event").and_then(Value::as_str).unwrap_or("unknown").to_string();
        if let (Some(node_id), Some(attempt)) = (
            value.get("node_id").and_then(Value::as_str),
            value.get("attempt").and_then(Value::as_u64),
        ) {
            let key = format!("{name}:{node_id}:{attempt}");
            if !seen_attempt_keys.insert(key.clone()) {
                duplicate_attempt_keys.insert(key);
            }
        }
        if name == "node_finished" {
            if let (Some(node_id), Some(status)) = (
                value.get("node_id").and_then(Value::as_str),
                value.get("status").and_then(Value::as_str),
            ) {
                terminal_statuses
                    .entry(node_id.to_string())
                    .or_default()
                    .insert(status.to_string());
            }
        }
        events.push(value);
    }
    let duplicate_singleton_events = ["run_started", "plan_built", "run_finished"]
        .into_iter()
        .filter(|name| {
            events
                .iter()
                .filter(|value| value.get("event").and_then(Value::as_str) == Some(*name))
                .count()
                > 1
        })
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let conflicting_node_terminal_events = terminal_statuses
        .into_iter()
        .filter(|(_, statuses)| statuses.len() > 1)
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();
    let (index_entry_count, index_in_sync) = match fs::read(run_dir.join("run-log.index.json")) {
        Ok(bytes) => {
            let index: Vec<Value> =
                serde_json::from_slice(&bytes).map_err(|_| ExitCode::from(3))?;
            let in_sync = index.len() == events.len()
                && index.iter().zip(events.iter()).all(|(entry, event)| {
                    entry.get("event").and_then(Value::as_str)
                        == event.get("event").and_then(Value::as_str)
                });
            (Some(index.len()), Some(in_sync))
        }
        Err(_) => (None, None),
    };
    let outputs = read_run_outputs_index(run_dir)?;
    let mut seen_paths = BTreeSet::new();
    let duplicate_output_paths = outputs
        .files
        .iter()
        .filter_map(|file| {
            if !seen_paths.insert(file.path.clone()) {
                Some(file.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let exactly_once_ready = duplicate_singleton_events.is_empty()
        && duplicate_attempt_keys.is_empty()
        && conflicting_node_terminal_events.is_empty()
        && duplicate_output_paths.is_empty()
        && index_in_sync.unwrap_or(false);
    Ok(WriteDisciplineAuditReport {
        event_count: events.len(),
        index_entry_count,
        index_in_sync,
        duplicate_singleton_events,
        duplicate_attempt_keys: duplicate_attempt_keys.into_iter().collect(),
        conflicting_node_terminal_events,
        duplicate_output_paths,
        exactly_once_ready,
    })
}

fn build_worker_recovery_report(
    simulation: &WorkerRecoverySimulation,
) -> WorkerRecoveryAuditReport {
    let recovered_state = reconcile_orphaned_node(&simulation.rule);
    let manual_review_required = simulation.side_effect_uncertain
        && matches!(simulation.rule.action, bijux_dag_runtime::SchedulerRecoveryAction::Requeue);
    let checkpoint_resume_possible = simulation.has_checkpoint
        && matches!(
            simulation.rule.action,
            bijux_dag_runtime::SchedulerRecoveryAction::Reattach
                | bijux_dag_runtime::SchedulerRecoveryAction::Requeue
        );
    let recommended_action = if manual_review_required {
        "hold_for_operator_review".to_string()
    } else if checkpoint_resume_possible {
        "resume_from_checkpoint_or_reattach".to_string()
    } else {
        format!("{:?}", simulation.rule.action).to_lowercase()
    };
    WorkerRecoveryAuditReport {
        orphaned_node_state: simulation.rule.orphaned_node_state.clone(),
        recovery_action: format!("{:?}", simulation.rule.action).to_lowercase(),
        recovered_state,
        checkpoint_resume_possible,
        manual_review_required,
        recommended_action,
    }
}

fn build_control_recovery_report(
    simulation: &ControlRecoverySimulation,
) -> ControlRecoveryAuditReport {
    let stuck_detected = detect_stuck_run(
        simulation.now_unix_ms,
        simulation.last_progress_unix_ms,
        simulation.last_heartbeat_unix_ms,
        &simulation.stuck_policy,
    );
    let pause_state = evaluate_pause_state(
        &simulation.pause_policy,
        simulation.queued_count,
        simulation.ready_count,
        simulation.running_count,
    )
    .into_iter()
    .map(|(key, value)| (key.to_string(), value))
    .collect::<BTreeMap<_, _>>();
    let node_states = simulation
        .node_states
        .iter()
        .map(|entry| (entry.node_id.clone(), entry.state.clone()))
        .collect::<Vec<_>>();
    let consistency =
        check_run_consistency(&node_states, &simulation.artifact_nodes, &simulation.summary);
    let quarantine_reason = should_quarantine_run(&simulation.summary.state, &consistency);
    let repair_outcome = validate_and_repair_run_metadata(
        simulation.manifest_exists,
        simulation.index_exists,
        simulation.allow_repair,
    );
    let recommended_action = if quarantine_reason.is_some() {
        "quarantine_and_repair".to_string()
    } else if stuck_detected {
        "pause_dispatch_and_reconcile".to_string()
    } else {
        "reattach_scheduler_and_continue".to_string()
    };
    ControlRecoveryAuditReport {
        stuck_detected,
        pause_state,
        consistency_summary_matches_node_states: consistency.summary_matches_node_states,
        all_success_nodes_have_artifacts: consistency.all_success_nodes_have_artifacts,
        mismatches: consistency.mismatches,
        quarantine_reason,
        repair_outcome: serde_json::to_value(&repair_outcome).unwrap_or_else(|_| json!({})),
        recommended_action,
    }
}

fn enforce_mark_success_gate(
    simulation: &InterventionSimulation,
    report: &mut bijux_dag_runtime::ManualInterventionAuditReport,
) {
    if report.action != "mark-success" {
        return;
    }

    let required = simulation.required_artifacts.iter().cloned().collect::<BTreeSet<_>>();
    let present = simulation.present_artifacts.iter().cloned().collect::<BTreeSet<_>>();
    let missing = required.difference(&present).cloned().collect::<Vec<_>>();

    if simulation.record.node_id.is_none() {
        report.allowed = false;
        report.notes.push("mark-success requires a target node_id".to_string());
    }
    if !simulation.lineage_recorded {
        report.allowed = false;
        report.notes.push("mark-success requires recorded affected lineage".to_string());
    }
    if !missing.is_empty() && !simulation.allow_missing_required_artifacts {
        report.allowed = false;
        report.notes.push(format!(
            "mark-success cannot fabricate missing required artifacts: {}",
            missing.join(", ")
        ));
    }
}

pub(crate) fn handle_runtime_command(
    cli: &DagCli,
    command: &RuntimeCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        RuntimeCommands::ExecutePayload { payload, result, in_place } => {
            let payload: RemoteNodeExecutionPayload = parse_json_file(payload)?;
            let execution = if *in_place {
                execute_remote_payload_in_place(payload)
            } else {
                MockRemoteWorker.execute_payload(payload)
            }
            .map_err(|_| ExitCode::from(3))?;
            if let Some(parent) = result.parent() {
                fs::create_dir_all(parent).map_err(|_| ExitCode::from(3))?;
            }
            fs::write(result, serde_json::to_vec_pretty(&execution).map_err(|_| ExitCode::from(3))?)
                .map_err(|_| ExitCode::from(3))?;
            Ok(match execution.node_result.status {
                bijux_dag_runtime::NodeStatus::Success | bijux_dag_runtime::NodeStatus::Cached => {
                    ExitCode::SUCCESS
                }
                bijux_dag_runtime::NodeStatus::Cancelled => ExitCode::from(130),
                bijux_dag_runtime::NodeStatus::Skipped
                | bijux_dag_runtime::NodeStatus::Failed => ExitCode::from(3),
            })
        }
        RuntimeCommands::Isolation { dag } => {
            let graph = parse_graph(&read_file(dag)?)?;
            let report = build_execution_isolation_report(&graph, &RuntimeConfig::default())
                .map_err(|_| ExitCode::from(3))?;
            let policy_surface = policy_surface_payload(&graph, &RuntimeConfig::default(), false)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.isolation",
                    true,
                    json!({
                        "execution_isolation": report,
                        "policy_surface": policy_surface,
                    }),
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&json!({
                    "execution_isolation": report,
                    "policy_surface": policy_surface,
                }))
                .unwrap()
            );
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Dispatch { simulation } => {
            let simulation: DispatchSimulation = parse_json_file(simulation)?;
            let report = audit_dispatch_discipline(
                &simulation.dispatches,
                &simulation.remote_status_events,
                &simulation.batch_events,
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.dispatch",
                    report.idempotent_dispatch_guarantee,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if report.idempotent_dispatch_guarantee {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id": "dispatch_discipline_violation",
                            "severity": "error",
                            "message": "duplicate dispatch or duplicate batch delivery detected",
                        })]
                    },
                    if report.idempotent_dispatch_guarantee {
                        ExitCode::SUCCESS
                    } else {
                        ExitCode::from(3)
                    },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if report.idempotent_dispatch_guarantee {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::State { run_dir } => {
            let report = audit_durable_state(run_dir)?;
            let ok = report.missing_components.is_empty();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.state",
                    ok,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"durable_state_missing_components",
                            "severity":"error",
                            "message":"run state is missing durable control-plane artifacts",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::WriteDiscipline { run_dir } => {
            let report = audit_write_discipline(run_dir)?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.write-discipline",
                    report.exactly_once_ready,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if report.exactly_once_ready {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"write_discipline_violation",
                            "severity":"error",
                            "message":"event or artifact writes violate exactly-once discipline",
                        })]
                    },
                    if report.exactly_once_ready { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if report.exactly_once_ready {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::WorkerRecovery { simulation } => {
            let simulation: WorkerRecoverySimulation = parse_json_file(simulation)?;
            let report = build_worker_recovery_report(&simulation);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.worker-recovery",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::ControlRecovery { simulation } => {
            let simulation: ControlRecoverySimulation = parse_json_file(simulation)?;
            let report = build_control_recovery_report(&simulation);
            let ok = report.quarantine_reason.is_none();
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.control-recovery",
                    ok,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"control_recovery_requires_quarantine",
                            "severity":"error",
                            "message":"control-plane recovery found inconsistent run state",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::Repair {
            run_dir,
            apply,
            out,
            run_id,
            jobs,
            materialize_inputs,
            cache,
            cache_dir,
            remote_cache_dir,
        } => {
            if let Some(out) = out.as_ref() {
                require_safe_path(out)?;
            }
            if let Some(cache_dir) = cache_dir.as_ref() {
                require_safe_path(cache_dir)?;
            }
            let report = if *apply {
                apply_run_repair(
                    run_dir,
                    &RepairExecutionOptions {
                        out_dir: out.clone(),
                        run_id: run_id.clone(),
                        jobs: *jobs,
                        materialize_inputs: crate::run_data::map_materialize_mode(
                            *materialize_inputs,
                        ),
                        cache_mode: match cache {
                            crate::commands::CacheModeArg::Off => crate::CacheMode::Off,
                            crate::commands::CacheModeArg::Read => crate::CacheMode::Read,
                            crate::commands::CacheModeArg::Readwrite => crate::CacheMode::ReadWrite,
                        },
                        cache_dir: cache_dir.clone(),
                        remote_cache_dir: remote_cache_dir.clone(),
                    },
                )?
            } else {
                plan_run_repair(run_dir, cache_dir.clone(), remote_cache_dir.clone())?
            };
            let ok = run_repair_ok(&report, *apply);
            let payload = serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.repair",
                    ok,
                    payload,
                    Vec::new(),
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&payload).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::Retry { dag, node_id, attempt, failure_class, exit_code } => {
            let graph = parse_graph(&read_file(dag)?)?;
            let report = build_retry_decision_report(
                &graph,
                &RuntimeConfig::default(),
                node_id,
                *attempt,
                failure_class,
                *exit_code,
            )
            .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.retry",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Timeout {
            dag,
            node_id,
            queue_wait_ms,
            execution_ms,
            total_elapsed_ms,
            heartbeat_gap_ms,
            heartbeat_timeout_ms,
            sla_timeout_ms,
        } => {
            let graph = parse_graph(&read_file(dag)?)?;
            let report = build_timeout_audit_report(
                &graph,
                &RuntimeConfig::default(),
                node_id,
                *queue_wait_ms,
                *execution_ms,
                *total_elapsed_ms,
                *heartbeat_gap_ms,
                *heartbeat_timeout_ms,
                *sla_timeout_ms,
            )
            .map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.timeout",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Heartbeat { simulation } => {
            let simulation: HeartbeatSimulation = parse_json_file(simulation)?;
            let report = build_heartbeat_audit_report(
                &simulation.heartbeat,
                simulation.now_unix_ms,
                &simulation.liveness_policy,
                &simulation.heartbeat_semantics,
                simulation.lease.as_ref(),
                simulation.lease_semantics.as_ref(),
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.heartbeat",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Cancel { simulation } => {
            let simulation: CancellationSimulation = parse_json_file(simulation)?;
            let report = build_cancellation_audit_report(
                simulation.isolation_mode,
                simulation.issued_unix_ms,
                simulation.delivered_unix_ms,
                simulation.deadline_ms,
                simulation.batch_state.as_ref(),
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.cancel",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Pause { simulation } => {
            let simulation: PauseSimulation = parse_json_file(simulation)?;
            let report = build_pause_resume_audit_report(
                &simulation.policy,
                simulation.queued_count,
                simulation.ready_count,
                simulation.running_count,
                &simulation.interruption_class,
                &simulation.resume_policy,
            );
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.pause",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            Ok(ExitCode::SUCCESS)
        }
        RuntimeCommands::Intervention { simulation } => {
            let simulation: InterventionSimulation = parse_json_file(simulation)?;
            let mut report = build_manual_intervention_audit_report(
                &simulation.record,
                &simulation.policy,
                simulation.manual_attempts_so_far,
            );
            enforce_mark_success_gate(&simulation, &mut report);
            let ok = report.allowed;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.intervention",
                    ok,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id": "manual_intervention_rejected",
                            "severity": "error",
                            "message": "manual intervention violates runtime policy",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::Transition { simulation } => {
            let simulation: TransitionSimulation = parse_json_file(simulation)?;
            let report = build_transition_audit_report(
                &simulation.node_transitions,
                &simulation.run_transitions,
                simulation.final_run_state,
                &simulation.final_node_states,
                simulation.causal_failure_count,
            );
            let ok = report.node_transition_errors.is_empty()
                && report.run_transition_errors.is_empty()
                && report.consistency.valid;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.transition",
                    ok,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"state_transition_violation",
                            "severity":"error",
                            "message":"transition trace or final run state is inconsistent",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
        RuntimeCommands::Events { run_dir } => {
            let report = audit_run_event_log(run_dir).map_err(|_| ExitCode::from(3))?;
            let ok = report.malformed_events == 0
                && report.missing_required_events.is_empty()
                && report.singleton_event_violations.is_empty()
                && report.index_in_sync.unwrap_or(true);
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.events",
                    ok,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    if ok {
                        Vec::new()
                    } else {
                        vec![json!({
                            "id":"event_log_audit_failed",
                            "severity":"error",
                            "message":"run event history is incomplete or inconsistent",
                        })]
                    },
                    if ok { ExitCode::SUCCESS } else { ExitCode::from(3) },
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
            if ok {
                Ok(ExitCode::SUCCESS)
            } else {
                Err(ExitCode::from(3))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_runtime_command, parse_node_state_str};
    use crate::commands::{CacheModeArg, Commands, DagCli, MaterializeModeArg, RuntimeCommands};
    use crate::ExitCode;
    use bijux_dag_runtime::NodeState;
    use serde_json::Value;
    use std::fs;
    use std::path::PathBuf;

    fn quiet_json_cli(command: RuntimeCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Runtime { command } }
    }

    fn repair_command(run_dir: PathBuf, apply: bool) -> RuntimeCommands {
        RuntimeCommands::Repair {
            run_dir,
            apply,
            out: None,
            run_id: None,
            jobs: 1,
            materialize_inputs: MaterializeModeArg::Copy,
            cache: CacheModeArg::Off,
            cache_dir: None,
            remote_cache_dir: None,
        }
    }

    #[test]
    fn runtime_routes_support_isolation_report() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"runtime","owners":[],"tags":[]},
              "nodes":[
                {"id":"const1","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
                {"id":"task1","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"effects":["filesystem"],"params":{"argv":["/bin/sh","-c","true"]}}
              ],
              "edges":[{"from":{"node_id":"const1","port":"out"},"to":{"node_id":"task1","port":"in"}}]
            }"#,
        )
        .expect("write dag");

        let cli = quiet_json_cli(RuntimeCommands::Isolation { dag: dag.clone() });
        let code =
            handle_runtime_command(&cli, &RuntimeCommands::Isolation { dag }).expect("isolation");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_reject_duplicate_dispatches() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("dispatch.json");
        fs::write(
            &simulation,
            r#"{
              "dispatches":[
                {"run_id":"run-1","node_id":"node-a"},
                {"run_id":"run-1","node_id":"node-a"}
              ],
              "remote_status_events":[
                {"run_id":"run-1","node_id":"node-a","sequence":1,"status":"running","unix_ms":10}
              ],
              "batch_events":[]
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Dispatch { simulation: simulation.clone() });
        let exit = handle_runtime_command(&cli, &RuntimeCommands::Dispatch { simulation })
            .expect_err("dispatch must fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn runtime_routes_do_not_panic_on_missing_simulation() {
        let cli = quiet_json_cli(RuntimeCommands::Dispatch {
            simulation: PathBuf::from("/missing/dispatch.json"),
        });
        let result = std::panic::catch_unwind(|| {
            let _ = handle_runtime_command(
                &cli,
                &RuntimeCommands::Dispatch { simulation: PathBuf::from("/missing/dispatch.json") },
            );
        });
        assert!(result.is_ok());
    }

    #[test]
    fn runtime_routes_support_retry_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"runtime","owners":[],"tags":[]},
              "nodes":[
                {"id":"const1","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
                {
                  "id":"task1",
                  "kind":"shell",
                  "inputs":["in"],
                  "outputs":[{"name":"out","path":"b/out"}],
                  "retry":{"max_attempts":4,"backoff_ms":25},
                  "effects":["filesystem"],
                  "params":{
                    "argv":["/bin/sh","-c","true"],
                    "retry_backoff_strategy":"exponential",
                    "retry_jitter_ms":5,
                    "retryable_failure_classes":["execution_transient","artifact_transient"]
                  }
                }
              ],
              "edges":[{"from":{"node_id":"const1","port":"out"},"to":{"node_id":"task1","port":"in"}}]
            }"#,
        )
        .expect("write dag");

        let cli = quiet_json_cli(RuntimeCommands::Retry {
            dag: dag.clone(),
            node_id: "task1".to_string(),
            attempt: 2,
            failure_class: "artifact_transient".to_string(),
            exit_code: None,
        });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Retry {
                dag,
                node_id: "task1".to_string(),
                attempt: 2,
                failure_class: "artifact_transient".to_string(),
                exit_code: None,
            },
        )
        .expect("retry");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_timeout_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let dag = dir.path().join("graph.json");
        fs::write(
            &dag,
            r#"{
              "spec":"bijux-dag/v0.1",
              "meta":{"name":"runtime","owners":[],"tags":[]},
              "nodes":[
                {"id":"const1","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
                {
                  "id":"task1",
                  "kind":"shell",
                  "inputs":["in"],
                  "outputs":[{"name":"out","path":"b/out"}],
                  "timeout_ms":40,
                  "effects":["filesystem"],
                  "params":{
                    "argv":["/bin/sh","-c","true"],
                    "queue_timeout_ms":10,
                    "total_budget_timeout_ms":80
                  }
                }
              ],
              "edges":[{"from":{"node_id":"const1","port":"out"},"to":{"node_id":"task1","port":"in"}}]
            }"#,
        )
        .expect("write dag");

        let cli = quiet_json_cli(RuntimeCommands::Timeout {
            dag: dag.clone(),
            node_id: "task1".to_string(),
            queue_wait_ms: Some(15),
            execution_ms: Some(50),
            total_elapsed_ms: Some(90),
            heartbeat_gap_ms: Some(5),
            heartbeat_timeout_ms: Some(20),
            sla_timeout_ms: Some(100),
        });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Timeout {
                dag,
                node_id: "task1".to_string(),
                queue_wait_ms: Some(15),
                execution_ms: Some(50),
                total_elapsed_ms: Some(90),
                heartbeat_gap_ms: Some(5),
                heartbeat_timeout_ms: Some(20),
                sla_timeout_ms: Some(100),
            },
        )
        .expect("timeout");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_heartbeat_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("heartbeat.json");
        fs::write(
            &simulation,
            r#"{
              "heartbeat":{"worker_id":"worker-a","unix_ms":1000,"inflight_nodes":["node-a"]},
              "now_unix_ms":2200,
              "liveness_policy":{"heartbeat_timeout_ms":1500,"grace_retries":2},
              "heartbeat_semantics":{"interval_ms":500,"timeout_ms":2500,"delayed_threshold_ms":1000},
              "lease":{"lease_id":"lease-1","run_id":"run-1","node_id":"node-a","worker_id":"worker-a","expires_unix_ms":1700},
              "lease_semantics":{"lease_duration_ms":2000,"renew_before_expiry_ms":500,"max_renewals":2,"recovery_grace_ms":800}
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Heartbeat { simulation: simulation.clone() });
        let code = handle_runtime_command(&cli, &RuntimeCommands::Heartbeat { simulation })
            .expect("heartbeat");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_cancellation_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("cancel.json");
        fs::write(
            &simulation,
            r#"{
              "isolation_mode":"Container",
              "issued_unix_ms":1000,
              "delivered_unix_ms":1300,
              "deadline_ms":500,
              "batch_state":{
                "metadata":{
                  "scheduler_id":"scheduler",
                  "submission_time_unix_ms":1,
                  "run_id":"run-1",
                  "node_id":"node-a",
                  "attempt_id":"1",
                  "resource_request":"cpu=1",
                  "status_mapping":"sim"
                },
                "events":[{"scheduler_id":"scheduler","status":"submitted","unix_ms":1}],
                "cancelled":false
              }
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Cancel { simulation: simulation.clone() });
        let code =
            handle_runtime_command(&cli, &RuntimeCommands::Cancel { simulation }).expect("cancel");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_pause_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("pause.json");
        fs::write(
            &simulation,
            r#"{
              "policy":{"mode":"PauseAllNewDispatch","preserve_running_nodes":true},
              "queued_count":2,
              "ready_count":1,
              "running_count":1,
              "interruption_class":"WorkerLoss",
              "resume_policy":"VerifyAndContinue"
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Pause { simulation: simulation.clone() });
        let code =
            handle_runtime_command(&cli, &RuntimeCommands::Pause { simulation }).expect("pause");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_manual_intervention_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("intervention.json");
        fs::write(
            &simulation,
            r#"{
              "record":{
                "run_id":"run-1",
                "node_id":"node-a",
                "operator":"operator-a",
                "action":"retry",
                "reason":"transient artifact outage",
                "recorded_unix_ms":123
              },
              "policy":{"max_manual_attempts":2,"require_reason":true,"requires_audit_record":true},
              "manual_attempts_so_far":1
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Intervention { simulation: simulation.clone() });
        let code = handle_runtime_command(&cli, &RuntimeCommands::Intervention { simulation })
            .expect("intervention");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_reject_mark_success_without_lineage_or_artifacts() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("mark-success.json");
        fs::write(
            &simulation,
            r#"{
              "record":{
                "run_id":"run-1",
                "node_id":"node-a",
                "operator":"operator-a",
                "action":"mark-success",
                "reason":"operator override",
                "recorded_unix_ms":123
              },
              "policy":{"max_manual_attempts":2,"require_reason":true,"requires_audit_record":true},
              "manual_attempts_so_far":0,
              "lineage_recorded":false,
              "required_artifacts":["result.json"],
              "present_artifacts":[],
              "allow_missing_required_artifacts":false
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Intervention { simulation: simulation.clone() });
        let exit = handle_runtime_command(&cli, &RuntimeCommands::Intervention { simulation })
            .expect_err("mark-success must fail");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn runtime_routes_reject_inconsistent_transition_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        let simulation = dir.path().join("transition.json");
        fs::write(
            &simulation,
            r#"{
              "node_transitions":[
                {"from":"Pending","to":"Eligible","cause":"SchedulerEligible"},
                {"from":"Eligible","to":"Queued","cause":"SchedulerQueued"},
                {"from":"Queued","to":"Running","cause":"ExecutionStarted"},
                {"from":"Running","to":"Success","cause":"ExecutionSucceeded"}
              ],
              "run_transitions":[{"from":"Running","to":"Failed","cause":"ExecutionFailed"}],
              "final_run_state":"Failed",
              "final_node_states":["Success"],
              "causal_failure_count":0
            }"#,
        )
        .expect("write simulation");

        let cli = quiet_json_cli(RuntimeCommands::Transition { simulation: simulation.clone() });
        let exit = handle_runtime_command(&cli, &RuntimeCommands::Transition { simulation })
            .expect_err("transition");
        assert_eq!(exit, ExitCode::from(3));
    }

    #[test]
    fn runtime_routes_audit_durable_state_and_write_discipline() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::create_dir_all(dir.path().join("outputs")).expect("outputs");
        std::fs::write(
            dir.path().join("manifest.json"),
            r#"{"run_id":"run-1","status":"success","graph_fingerprint":"fp"}"#,
        )
        .expect("manifest");
        std::fs::write(
            dir.path().join("graph.snapshot.json"),
            r#"{"graph":{"nodes":[],"edges":[]},"graph_fingerprint":"fp"}"#,
        )
        .expect("graph snapshot");
        std::fs::write(
            dir.path().join("run.snapshot.json"),
            r#"{
              "run_id":"run-1",
              "graph_snapshot_path":"graph.snapshot.json",
              "planner_config":"{}",
              "scheduler_config":"{}",
              "policy_config":"{}",
              "provenance":"{}",
              "submission_source":"manual",
              "trigger_source":"manual",
              "operator":"ops",
              "labels":[],
              "parent_run_id":null,
              "requested_selectors":[],
              "selected_nodes":[],
              "dependency_closure_enabled":false,
              "replay_source_run_id":null,
              "partial_rerun_contract":null
            }"#,
        )
        .expect("run snapshot");
        std::fs::write(
            dir.path().join("run.log.jsonl"),
            r#"{"event":"run_started","ts":1}
{"event":"plan_built","ts":2}
{"event":"run_finished","ts":3}"#,
        )
        .expect("run log");
        std::fs::write(
            dir.path().join("run-log.index.json"),
            r#"[{"event":"run_started"},{"event":"plan_built"},{"event":"run_finished"}]"#,
        )
        .expect("index");
        std::fs::write(dir.path().join("outputs").join("index.json"), r#"{"files":[]}"#)
            .expect("outputs index");
        std::fs::write(
            dir.path().join("lineage.snapshot.json"),
            r#"{"schema_version":"v0.1","edges":[]}"#,
        )
        .expect("lineage");
        std::fs::write(
            dir.path().join("observability.timeline.json"),
            r#"{"schema_version":"v0.1","entries":[]}"#,
        )
        .expect("timeline");
        std::fs::write(
            dir.path().join("scheduler.checkpoint.json"),
            r#"{"loop_index":1,"ready_queue_depth":0,"ready_queue":[],"inflight":[],"scheduled":[],"blocked_by_budget":[],"blocked_reasons":{},"completed_statuses":{},"failure_propagation_mode":"isolate_branch","dependency_closure_enabled":false,"generated_unix_ms":1}"#,
        )
        .expect("checkpoint");

        let state_cli =
            quiet_json_cli(RuntimeCommands::State { run_dir: dir.path().to_path_buf() });
        let state = handle_runtime_command(
            &state_cli,
            &RuntimeCommands::State { run_dir: dir.path().to_path_buf() },
        )
        .expect("state");
        assert_eq!(state, ExitCode::SUCCESS);

        let discipline_cli =
            quiet_json_cli(RuntimeCommands::WriteDiscipline { run_dir: dir.path().to_path_buf() });
        let discipline = handle_runtime_command(
            &discipline_cli,
            &RuntimeCommands::WriteDiscipline { run_dir: dir.path().to_path_buf() },
        )
        .expect("discipline");
        assert_eq!(discipline, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_support_worker_and_control_recovery_reports() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(
            dir.path().join("worker.json"),
            r#"{
              "rule":{"orphaned_node_state":"Running","action":"Requeue"},
              "has_checkpoint":true,
              "side_effect_uncertain":false
            }"#,
        )
        .expect("worker");
        let worker_cli = quiet_json_cli(RuntimeCommands::WorkerRecovery {
            simulation: dir.path().join("worker.json"),
        });
        let worker = handle_runtime_command(
            &worker_cli,
            &RuntimeCommands::WorkerRecovery { simulation: dir.path().join("worker.json") },
        )
        .expect("worker");
        assert_eq!(worker, ExitCode::SUCCESS);

        std::fs::write(
            dir.path().join("control.json"),
            r#"{
              "now_unix_ms":100,
              "last_progress_unix_ms":95,
              "last_heartbeat_unix_ms":96,
              "stuck_policy":{"max_without_progress_ms":10,"max_without_heartbeat_ms":10},
              "pause_policy":{"mode":"PauseQueuedOnly","preserve_running_nodes":true},
              "queued_count":1,
              "ready_count":0,
              "running_count":1,
              "summary":{"run_id":"run_1","state":"Running","counts":{"success":1,"failed":0,"skipped":0,"cached":0}},
              "node_states":[{"node_id":"n1","state":"Success"}],
              "artifact_nodes":["n1"],
              "manifest_exists":true,
              "index_exists":true,
              "allow_repair":false
            }"#,
        )
        .expect("control");
        let control_cli = quiet_json_cli(RuntimeCommands::ControlRecovery {
            simulation: dir.path().join("control.json"),
        });
        let control = handle_runtime_command(
            &control_cli,
            &RuntimeCommands::ControlRecovery { simulation: dir.path().join("control.json") },
        )
        .expect("control");
        assert_eq!(control, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_can_repair_missing_manifest_and_index() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::create_dir_all(dir.path().join("nodes").join("extract")).expect("nodes");
        std::fs::create_dir_all(dir.path().join("outputs")).expect("outputs");
        std::fs::write(
            dir.path().join("graph.snapshot.json"),
            r#"{"graph":{"nodes":[],"edges":[]},"graph_fingerprint":"fp"}"#,
        )
        .expect("graph snapshot");
        std::fs::write(
            dir.path().join("run.snapshot.json"),
            r#"{
              "run_id":"run-1",
              "graph_snapshot_path":"graph.snapshot.json",
              "planner_config":"{}",
              "scheduler_config":"{}",
              "policy_config":"{}",
              "provenance":"{}",
              "submission_source":"manual",
              "trigger_source":"manual",
              "operator":"ops",
              "labels":["critical"],
              "parent_run_id":null,
              "requested_selectors":[],
              "selected_nodes":["extract"],
              "dependency_closure_enabled":false,
              "replay_source_run_id":null,
              "partial_rerun_contract":null
            }"#,
        )
        .expect("run snapshot");
        std::fs::write(
            dir.path().join("nodes").join("extract").join("trace.json"),
            r#"{
              "node_id":"extract",
              "status":"success",
              "started_unix_ms":1,
              "finished_unix_ms":2,
              "attempt":1,
              "fingerprint":"fp-node",
              "adapter_id":"shell",
              "adapter_version":"v1",
              "adapter_outputs_schema_version":"schema/v1"
            }"#,
        )
        .expect("trace");
        std::fs::write(dir.path().join("run.log.jsonl"), r#"{"event":"run_started","ts":1}"#)
            .expect("run log");
        std::fs::write(dir.path().join("outputs").join("index.json"), r#"{"files":[]}"#)
            .expect("outputs");

        let repair_cli = quiet_json_cli(repair_command(dir.path().to_path_buf(), true));
        let repair =
            handle_runtime_command(&repair_cli, &repair_command(dir.path().to_path_buf(), true))
                .expect("repair");
        assert_eq!(repair, ExitCode::SUCCESS);
        assert!(dir.path().join("manifest.json").exists());
        assert!(dir.path().join("run-log.index.json").exists());
    }

    #[test]
    fn runtime_state_and_repair_surface_report_incomplete_marker() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::create_dir_all(dir.path().join("outputs")).expect("outputs");
        std::fs::write(
            dir.path().join("manifest.json"),
            r#"{"run_id":"run-1","status":"failed","graph_fingerprint":"fp"}"#,
        )
        .expect("manifest");
        std::fs::write(
            dir.path().join("graph.snapshot.json"),
            r#"{"graph":{"nodes":[],"edges":[]},"graph_fingerprint":"fp"}"#,
        )
        .expect("graph snapshot");
        std::fs::write(
            dir.path().join("run.snapshot.json"),
            r#"{
              "run_id":"run-1",
              "graph_snapshot_path":"graph.snapshot.json",
              "planner_config":"{}",
              "scheduler_config":"{}",
              "policy_config":"{}",
              "provenance":"{}",
              "submission_source":"manual",
              "trigger_source":"manual",
              "operator":"ops",
              "labels":[],
              "parent_run_id":null,
              "requested_selectors":[],
              "selected_nodes":[],
              "dependency_closure_enabled":true,
              "replay_source_run_id":null,
              "partial_rerun_contract":null
            }"#,
        )
        .expect("run snapshot");
        std::fs::write(dir.path().join("run.log.jsonl"), "{\"event\":\"run_started\",\"ts\":1}\n")
            .expect("log");
        std::fs::write(dir.path().join("run-log.index.json"), r#"[{"event":"run_started"}]"#)
            .expect("index");
        std::fs::write(dir.path().join("outputs").join("index.json"), r#"{"files":[]}"#)
            .expect("outputs");
        std::fs::write(
            dir.path().join("lineage.snapshot.json"),
            r#"{"schema_version":"v0.1","edges":[]}"#,
        )
        .expect("lineage");
        std::fs::write(
            dir.path().join("observability.timeline.json"),
            r#"{"schema_version":"v0.1","entries":[]}"#,
        )
        .expect("timeline");
        std::fs::write(
            dir.path().join("scheduler.checkpoint.json"),
            r#"{"loop_index":1,"ready_queue_depth":0,"ready_queue":[],"inflight":[],"scheduled":[],"blocked_by_budget":[],"blocked_reasons":{},"completed_statuses":{},"failure_propagation_mode":"isolate_branch","dependency_closure_enabled":true,"generated_unix_ms":1}"#,
        )
        .expect("checkpoint");
        std::fs::write(
            dir.path().join(".run-incomplete.json"),
            r#"{"status":"incomplete","reason":"interrupted"}"#,
        )
        .expect("incomplete marker");

        let state_cli =
            quiet_json_cli(RuntimeCommands::State { run_dir: dir.path().to_path_buf() });
        let state_exit = handle_runtime_command(
            &state_cli,
            &RuntimeCommands::State { run_dir: dir.path().to_path_buf() },
        )
        .expect("state");
        assert_eq!(state_exit, ExitCode::SUCCESS);

        let repair_cli = quiet_json_cli(repair_command(dir.path().to_path_buf(), true));
        let repair_exit =
            handle_runtime_command(&repair_cli, &repair_command(dir.path().to_path_buf(), true))
                .expect("repair");
        assert_eq!(repair_exit, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_repair_rebuilds_cancellation_cause_and_counts_from_traces() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::create_dir_all(dir.path().join("nodes").join("prepare")).expect("prepare dir");
        std::fs::create_dir_all(dir.path().join("nodes").join("execute")).expect("execute dir");
        std::fs::create_dir_all(dir.path().join("outputs")).expect("outputs");
        std::fs::write(
            dir.path().join("graph.snapshot.json"),
            r#"{"graph":{"nodes":[],"edges":[]},"graph_fingerprint":"fp"}"#,
        )
        .expect("graph snapshot");
        std::fs::write(
            dir.path().join("run.snapshot.json"),
            r#"{
              "run_id":"run-cancelled",
              "graph_snapshot_path":"graph.snapshot.json",
              "planner_config":"{}",
              "scheduler_config":"{}",
              "policy_config":"{}",
              "provenance":"{}",
              "submission_source":"manual",
              "trigger_source":"cli",
              "operator":"ops",
              "labels":["cancelled"],
              "parent_run_id":null,
              "requested_selectors":[],
              "selected_nodes":["prepare","execute"],
              "dependency_closure_enabled":false,
              "replay_source_run_id":null,
              "partial_rerun_contract":null
            }"#,
        )
        .expect("run snapshot");
        std::fs::write(
            dir.path().join("nodes").join("prepare").join("trace.json"),
            r#"{
              "node_id":"prepare",
              "status":"success",
              "started_unix_ms":1,
              "finished_unix_ms":2,
              "attempt":1,
              "fingerprint":"fp-prepare",
              "adapter_id":"const",
              "adapter_version":"v1",
              "adapter_outputs_schema_version":"schema/v1"
            }"#,
        )
        .expect("prepare trace");
        std::fs::write(
            dir.path().join("nodes").join("execute").join("trace.json"),
            r#"{
              "node_id":"execute",
              "status":"cancelled",
              "started_unix_ms":3,
              "finished_unix_ms":4,
              "attempt":1,
              "fingerprint":"fp-execute",
              "adapter_id":"shell",
              "adapter_version":"v1",
              "adapter_outputs_schema_version":"schema/v1",
              "failure":{"kind":"Execution","code":"EXEC_CANCELLED","message":"execution cancelled by operator"},
              "transition_cause":"CancelRequested",
              "lifecycle_state":"cancelled"
            }"#,
        )
        .expect("execute trace");
        std::fs::write(
            dir.path().join("run.audit.json"),
            r#"[{"action":"cancel","ts":4,"run_id":"run-cancelled"}]"#,
        )
        .expect("audit");
        std::fs::write(dir.path().join("run.log.jsonl"), r#"{"event":"run_cancelled","ts":4}"#)
            .expect("run log");
        std::fs::write(dir.path().join("outputs").join("index.json"), r#"{"files":[]}"#)
            .expect("outputs");

        let repair_cli = quiet_json_cli(repair_command(dir.path().to_path_buf(), true));
        let repair =
            handle_runtime_command(&repair_cli, &repair_command(dir.path().to_path_buf(), true))
                .expect("repair");
        assert_eq!(repair, ExitCode::SUCCESS);

        let manifest: Value = serde_json::from_str(
            &std::fs::read_to_string(dir.path().join("manifest.json")).expect("manifest"),
        )
        .expect("parse manifest");
        assert_eq!(manifest["status"], "cancelled");
        assert_eq!(manifest["run_cancellation_cause"], "operator_interrupt");
        assert_eq!(manifest["node_counts"]["success"], 1);
        assert_eq!(manifest["node_counts"]["cancelled"], 1);
        assert_eq!(manifest["run_summary"]["cancelled"], 1);
    }

    #[test]
    fn runtime_routes_support_event_log_audits() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(dir.path().join("manifest.json"), r#"{"run_id":"run-1"}"#)
            .expect("manifest");
        std::fs::write(
            dir.path().join("run.log.jsonl"),
            r#"{"event":"run_started","ts":1}
{"event":"plan_built","ts":2}
{"event":"node_ready","ts":3,"node_id":"n1"}
{"event":"node_scheduled","ts":4,"node_id":"n1"}
{"event":"node_started","ts":5,"node_id":"n1"}
{"event":"node_attempt_started","ts":6,"node_id":"n1"}
{"event":"node_attempt_finished","ts":7,"node_id":"n1"}
{"event":"node_finished","ts":8,"node_id":"n1","status":"failed","reason":"timeout"}
{"event":"run_finished","ts":9}"#,
        )
        .expect("run log");
        std::fs::write(
            dir.path().join("run-log.index.json"),
            r#"[{"event":"run_started"},{"event":"plan_built"},{"event":"node_ready"},{"event":"node_scheduled"},{"event":"node_started"},{"event":"node_attempt_started"},{"event":"node_attempt_finished"},{"event":"node_finished"},{"event":"run_finished"}]"#,
        )
        .expect("index");
        std::fs::write(
            dir.path().join("observability.timeline.json"),
            r#"{"schema_version":"v0.1","entries":[{"unix_ms":1,"category":"start","label":"run","node_id":null}]}"#,
        )
        .expect("timeline");

        let cli = quiet_json_cli(RuntimeCommands::Events { run_dir: dir.path().to_path_buf() });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Events { run_dir: dir.path().to_path_buf() },
        )
        .expect("events");
        assert_eq!(code, ExitCode::SUCCESS);
    }

    #[test]
    fn runtime_routes_accept_stable_and_legacy_lifecycle_state_names() {
        assert_eq!(parse_node_state_str("ready"), Some(NodeState::Eligible));
        assert_eq!(parse_node_state_str("eligible"), Some(NodeState::Eligible));
        assert_eq!(parse_node_state_str("succeeded"), Some(NodeState::Success));
        assert_eq!(parse_node_state_str("success"), Some(NodeState::Success));
    }
}
