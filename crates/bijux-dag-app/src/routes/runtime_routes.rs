use crate::commands::{DagCli, RuntimeCommands};
use crate::{emit_json, parse_graph, read_file, ExitCode};
use bijux_dag_runtime::{
    audit_dispatch_discipline, build_execution_isolation_report, build_retry_decision_report,
    build_timeout_audit_report, BatchLifecycleEvent, DispatchKeyRecord, RuntimeConfig,
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
}
