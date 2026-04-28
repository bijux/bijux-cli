use crate::commands::{DagCli, RuntimeCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_runtime::{
    audit_dispatch_discipline, build_cancellation_audit_report,
    build_execution_isolation_report, build_retry_decision_report,
    build_heartbeat_audit_report, build_manual_intervention_audit_report,
    build_pause_resume_audit_report, build_timeout_audit_report, build_transition_audit_report,
    BatchAttemptState, BatchLifecycleEvent, DispatchKeyRecord, InterruptionClass,
    ManualInterventionRecord, NodeState, NodeTransition, OperatorRetryPolicy, ResumePolicy,
    RunPausePolicy, RunState, RunTransition, RuntimeConfig, TaskIsolationMode,
};
use bijux_dag_runtime::simulated_platform::RemoteStatusEvent;
use serde::Deserialize;
use serde_json::json;
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
}

#[derive(Debug, Deserialize)]
struct TransitionSimulation {
    node_transitions: Vec<NodeTransition>,
    run_transitions: Vec<RunTransition>,
    final_run_state: RunState,
    final_node_states: Vec<NodeState>,
    causal_failure_count: usize,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

pub(crate) fn handle_runtime_command(
    cli: &DagCli,
    command: &RuntimeCommands,
) -> Result<ExitCode, ExitCode> {
    match command {
        RuntimeCommands::Isolation { dag } => {
            let graph = parse_graph(&read_file(dag)?)?;
            let report =
                build_execution_isolation_report(&graph, &RuntimeConfig::default()).map_err(|_| ExitCode::from(3))?;
            if cli.json {
                return emit_json(
                    cli,
                    "dag.runtime.isolation",
                    true,
                    serde_json::to_value(&report).map_err(|_| ExitCode::from(3))?,
                    Vec::new(),
                    ExitCode::SUCCESS,
                );
            }
            println!("{}", serde_json::to_string_pretty(&report).unwrap());
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
        RuntimeCommands::Retry { dag, node_id, attempt, failure_class } => {
            let graph = parse_graph(&read_file(dag)?)?;
            let report = build_retry_decision_report(
                &graph,
                &RuntimeConfig::default(),
                node_id,
                *attempt,
                failure_class,
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
            let report = build_manual_intervention_audit_report(
                &simulation.record,
                &simulation.policy,
                simulation.manual_attempts_so_far,
            );
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
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
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
            if ok { Ok(ExitCode::SUCCESS) } else { Err(ExitCode::from(3)) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::handle_runtime_command;
    use crate::commands::{Commands, DagCli, RuntimeCommands};
    use crate::ExitCode;
    use std::fs;
    use std::path::PathBuf;

    fn quiet_json_cli(command: RuntimeCommands) -> DagCli {
        DagCli { json: true, quiet: true, command: Commands::Runtime { command } }
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
                &RuntimeCommands::Dispatch {
                    simulation: PathBuf::from("/missing/dispatch.json"),
                },
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
        });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Retry {
                dag,
                node_id: "task1".to_string(),
                attempt: 2,
                failure_class: "artifact_transient".to_string(),
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

        let cli =
            quiet_json_cli(RuntimeCommands::Heartbeat { simulation: simulation.clone() });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Heartbeat { simulation },
        )
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

        let cli =
            quiet_json_cli(RuntimeCommands::Intervention { simulation: simulation.clone() });
        let code = handle_runtime_command(
            &cli,
            &RuntimeCommands::Intervention { simulation },
        )
        .expect("intervention");
        assert_eq!(code, ExitCode::SUCCESS);
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

        let cli = quiet_json_cli(RuntimeCommands::Transition {
            simulation: simulation.clone(),
        });
        let exit = handle_runtime_command(
            &cli,
            &RuntimeCommands::Transition { simulation },
        )
        .expect_err("transition");
        assert_eq!(exit, ExitCode::from(3));
    }
}
