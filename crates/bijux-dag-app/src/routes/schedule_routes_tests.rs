use super::handle_schedule_command;
use crate::commands::{
    Commands, DagCli, ScheduleBackfillCommands, ScheduleCommands, ScheduleControlCommands,
    ScheduleQueueCommands,
};
use crate::ExitCode;
use std::fs;
use std::path::PathBuf;

fn quiet_json_cli() -> DagCli {
    DagCli { json: true, quiet: true, command: Commands::Version }
}

fn write_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "nightly-catalog",
              "dag_name": "atlas.catalog",
              "dag_version_policy": "run-latest",
              "trigger": {
                "Cron": {
                  "expression": "0 2 * * *",
                  "timezone": "UTC"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": true, "max_catch_up_runs": 3}
            }
          ]
        }"#,
    )
    .expect("write registry");
    (dir, registry)
}

fn write_submission_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-submit-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "manual-ops",
              "dag_name": "atlas.manual-ops",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "requested_at": {"type": "integer", "required": true},
                "manual_region": {"type": "string", "required": true}
              },
              "input_bindings": {
                "requested_at": {"source": "requested_unix_ms"},
                "manual_region": {
                  "source": "manual_argument",
                  "key": "region"
                }
              },
              "trigger": "Manual",
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            },
            {
              "id": "event-ingest",
              "dag_name": "atlas.event-ingest",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "event_tenant": {"type": "string", "required": true},
                "event_payload": {"type": "object", "required": true}
              },
              "input_bindings": {
                "event_tenant": {
                  "source": "event_payload",
                  "pointer": "/tenant"
                },
                "event_payload": {"source": "event_payload"}
              },
              "trigger": {
                "Event": {
                  "event_type": "dataset.ready",
                  "source": "catalog"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            },
            {
              "id": "signal-refresh",
              "dag_name": "atlas.signal-refresh",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "signal_tenant": {"type": "string", "required": true}
              },
              "input_bindings": {
                "signal_tenant": {
                  "source": "signal_payload",
                  "pointer": "/tenant"
                }
              },
              "trigger": {
                "Signal": {
                  "signal_name": "refresh-cache",
                  "payload_schema": "atlas.refresh-cache"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write submit registry");
    (dir, registry)
}

fn write_submission_rejection_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-submit-rejection-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "event-ingest",
              "dag_name": "atlas.event-ingest",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "event_tenant": {"type": "integer", "required": true}
              },
              "input_bindings": {
                "event_tenant": {
                  "source": "event_payload",
                  "pointer": "/tenant"
                }
              },
              "trigger": {
                "Event": {
                  "event_type": "dataset.ready",
                  "source": "catalog"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write rejection registry");
    (dir, registry)
}

fn write_compile_rejection_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-compile-rejection-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "manual-ops",
              "dag_name": "atlas.manual-ops",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "manual_region": {"type": "string", "required": true}
              },
              "input_bindings": {
                "manual_region": {
                  "source": "manual_argument",
                  "key": "region"
                }
              },
              "trigger": "Manual",
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write compile rejection registry");
    (dir, registry)
}

fn write_backfill_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-backfill-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "historical-catalog",
              "dag_name": "atlas.catalog",
              "dag_version_policy": "run-latest",
              "trigger": {
                "Backfill": {
                  "window_start_unix_ms": 1000,
                  "window_end_unix_ms": 121000,
                  "partition_by": "dataset",
                  "partition_keys": ["sample-a", "sample-b"],
                  "max_parallelism": 2,
                  "failure_policy": "pause"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write backfill registry");
    (dir, registry)
}

fn write_invalid_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry-invalid.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "broken",
              "dag_name": "atlas.catalog",
              "dag_version_policy": "run-latest",
              "trigger": {
                "Cron": {
                  "expression": "bad cron",
                  "timezone": "UTC"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": true, "max_catch_up_runs": 3}
            }
          ]
        }"#,
    )
    .expect("write invalid registry");
    (dir, registry)
}

fn write_invalid_timezone_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry-invalid-timezone.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "broken-timezone",
              "dag_name": "atlas.catalog",
              "dag_version_policy": "run-latest",
              "trigger": {
                "Cron": {
                  "expression": "0 2 * * *",
                  "timezone": "Mars/Olympus"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": true, "max_catch_up_runs": 3}
            }
          ]
        }"#,
    )
    .expect("write invalid timezone registry");
    (dir, registry)
}

fn write_invalid_dependency_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry-invalid-dependency.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "dependency-invalid",
              "dag_name": "atlas.publish",
              "dag_version_policy": "run-latest",
              "trigger": {
                "Dependency": {
                  "dag_name": "atlas.ingest",
                  "on_status": "deferred"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write invalid dependency registry");
    (dir, registry)
}

fn write_ordering_simulation_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let simulation = dir.path().join("schedule-order.json");
    fs::write(
        &simulation,
        r#"{
          "submissions": [
            {
              "schedule_id": "standard",
              "run_id": "run-2",
              "created_unix_ms": 20,
              "status": "Pending"
            },
            {
              "schedule_id": "critical",
              "run_id": "run-1",
              "created_unix_ms": 10,
              "status": "Pending"
            }
          ],
          "priorities": {
            "critical": "Critical",
            "standard": "Standard"
          },
          "policy": {
            "critical_weight": 100,
            "high_weight": 75,
            "standard_weight": 50,
            "low_weight": 25
          }
        }"#,
    )
    .expect("write ordering simulation");
    (dir, simulation)
}

fn write_throttling_simulation_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let simulation = dir.path().join("schedule-throttle.json");
    fs::write(
        &simulation,
        r#"{
          "pending_backfill_runs": 10,
          "pending_live_runs": 20,
          "policy": {
            "max_backfill_submissions_per_tick": 6,
            "reserve_live_capacity_percent": 20
          }
        }"#,
    )
    .expect("write throttling simulation");
    (dir, simulation)
}

fn write_dedup_simulation_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let simulation = dir.path().join("schedule-dedup.json");
    fs::write(
        &simulation,
        r#"{
          "events": ["evt-1", "evt-1", "evt-2"]
        }"#,
    )
    .expect("write dedup simulation");
    (dir, simulation)
}

fn write_sla_simulation_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let simulation = dir.path().join("schedule-sla.json");
    fs::write(
        &simulation,
        r#"{
          "start_samples": [
            {"observed_ms": 20, "expected_ms": 10},
            {"observed_ms": 5, "expected_ms": 10}
          ],
          "finish_samples": [
            {"observed_ms": 50, "expected_ms": 40}
          ],
          "queue_saturation_count": 2,
          "fairness_drift_count": 1
        }"#,
    )
    .expect("write sla simulation");
    (dir, simulation)
}

fn write_submission_inputs_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let inputs = dir.path().join("schedule-submit-inputs.json");
    fs::write(
        &inputs,
        r#"{
          "now_unix_ms": 200000,
          "manual_requests": [
            {
              "request_id": "manual-001",
              "schedule_id": "manual-ops",
              "requested_unix_ms": 175000,
              "arguments": {
                "region": "eu-west-1"
              }
            }
          ],
          "events": [
            {
              "event_id": "evt-001",
              "event_type": "dataset.ready",
              "source": "catalog",
              "occurred_unix_ms": 176000,
              "payload": {
                "tenant": "atlas",
                "batch": 7
              }
            }
          ],
          "signals": [
            {
              "signal_id": "sig-001",
              "signal_name": "refresh-cache",
              "occurred_unix_ms": 177000,
              "payload": {
                "tenant": "atlas"
              }
            }
          ]
        }"#,
    )
    .expect("write submit inputs");
    (dir, inputs)
}

fn write_dependency_submission_registry_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-dependency-registry.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "dependency-on-success",
              "dag_name": "atlas.publish-success",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "dependency_run_id": {"type": "string", "required": true},
                "dependency_status": {"type": "string", "required": true}
              },
              "input_bindings": {
                "dependency_run_id": {"source": "dependency_upstream_run_id"},
                "dependency_status": {"source": "dependency_status"}
              },
              "trigger": {
                "Dependency": {
                  "dag_name": "atlas.ingest",
                  "on_status": "success"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            },
            {
              "id": "dependency-on-failure",
              "dag_name": "atlas.publish-failure",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "dependency_run_id": {"type": "string", "required": true},
                "dependency_status": {"type": "string", "required": true}
              },
              "input_bindings": {
                "dependency_run_id": {"source": "dependency_upstream_run_id"},
                "dependency_status": {"source": "dependency_status"}
              },
              "trigger": {
                "Dependency": {
                  "dag_name": "atlas.ingest",
                  "on_status": "failure"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            },
            {
              "id": "dependency-on-terminal",
              "dag_name": "atlas.publish-terminal",
              "dag_version_policy": "run-latest",
              "input_contract": {
                "dependency_run_id": {"type": "string", "required": true},
                "dependency_status": {"type": "string", "required": true}
              },
              "input_bindings": {
                "dependency_run_id": {"source": "dependency_upstream_run_id"},
                "dependency_status": {"source": "dependency_status"}
              },
              "trigger": {
                "Dependency": {
                  "dag_name": "atlas.ingest",
                  "on_status": "any_terminal"
                }
              },
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "concurrency": {
                "per_dag": 2,
                "per_queue": 4,
                "per_tenant": 4,
                "per_node_group": null
              },
              "catch_up": {"enabled": false, "max_catch_up_runs": 0}
            }
          ]
        }"#,
    )
    .expect("write dependency registry");
    (dir, registry)
}

fn write_dependency_submission_inputs_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let inputs = dir.path().join("schedule-dependency-inputs.json");
    fs::write(
        &inputs,
        r#"{
          "now_unix_ms": 220000,
          "dependencies": [
            {
              "upstream_run_id": "atlas-run-success",
              "dag_name": "atlas.ingest",
              "status": "SUCCEEDED",
              "finished_unix_ms": 210000
            },
            {
              "upstream_run_id": "atlas-run-failure",
              "dag_name": "atlas.ingest",
              "status": "timed out",
              "finished_unix_ms": 211000
            }
          ]
        }"#,
    )
    .expect("write dependency inputs");
    (dir, inputs)
}

fn write_submission_rejection_inputs_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let inputs = dir.path().join("schedule-submit-rejection-inputs.json");
    fs::write(
        &inputs,
        r#"{
          "now_unix_ms": 200000,
          "events": [
            {
              "event_id": "evt-001",
              "event_type": "dataset.ready",
              "source": "catalog",
              "occurred_unix_ms": 176000,
              "payload": {
                "tenant": "atlas"
              }
            }
          ]
        }"#,
    )
    .expect("write rejection inputs");
    (dir, inputs)
}

fn write_submission_ledger_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let ledger = dir.path().join("schedule-submit-ledger.json");
    fs::write(
        &ledger,
        r#"{
          "entries": [
            {
              "schedule_id": "manual-ops",
              "dag_name": "atlas.manual-ops",
              "dag_version_policy": "run-latest",
              "queue": {"queue_name": "catalog", "tenant": "atlas"},
              "priority": "High",
              "graph_inputs": {
                "requested_at": 170000,
                "manual_region": "us-east-1"
              },
              "requested_unix_ms": 170000,
              "created_unix_ms": 170000,
              "run_id": "sched-manual-ops-existing",
              "trigger_kind": "manual",
              "dedupe_key": "manual:manual-ops:manual-000",
              "status": "Pending"
            }
          ]
        }"#,
    )
    .expect("write submit ledger");
    (dir, ledger)
}

fn write_backfill_advance_request_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let request = dir.path().join("backfill-advance-request.json");
    fs::write(
        &request,
        r#"{
          "now_unix_ms": 2500,
          "pending_live_runs": 1,
          "throttling_policy": {
            "max_backfill_submissions_per_tick": 4,
            "reserve_live_capacity_percent": 0
          },
          "status_updates": []
        }"#,
    )
    .expect("write advance request");
    (dir, request)
}

fn write_submission_status_updates_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let updates = dir.path().join("submission-status-updates.json");
    fs::write(
        &updates,
        r#"{
          "updates": [
            {
              "run_id": "sched-manual-ops-existing",
              "status": "Completed",
              "updated_unix_ms": 180000
            }
          ]
        }"#,
    )
    .expect("write status updates");
    (dir, updates)
}

fn write_priority_dispatch_policy_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let policy = dir.path().join("priority-dispatch-policy.json");
    fs::write(
        &policy,
        r#"{
          "weights": {
            "critical_weight": 100,
            "high_weight": 75,
            "standard_weight": 50,
            "low_weight": 25
          },
          "starvation": {
            "max_ticks_without_dispatch": 3,
            "priority_boost_after_ticks": 1
          }
        }"#,
    )
    .expect("write dispatch policy");
    (dir, policy)
}

fn write_schedule_override_fixture() -> (tempfile::TempDir, PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let overrides = dir.path().join("schedule-overrides.json");
    fs::write(
        &overrides,
        r#"{
          "records": [
            {
              "schedule_id": "manual-ops",
              "operator": "atlas-ops",
              "action": "pause",
              "reason": "hold while downstream validation is degraded",
              "created_unix_ms": 180000
            }
          ]
        }"#,
    )
    .expect("write overrides");
    (dir, overrides)
}

#[test]
fn schedule_validate_returns_success_for_valid_registry() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry })
        .expect("schedule validate");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_validate_rejects_unknown_dependency_trigger_condition() {
    let (_tmp, registry) = write_invalid_dependency_registry_fixture();
    let cli = quiet_json_cli();
    let err = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry })
        .expect_err("invalid dependency registry must fail");
    assert_eq!(err, ExitCode::from(3));
}

#[test]
fn schedule_submit_returns_success_and_can_write_updated_ledger() {
    let (_tmp_registry, registry) = write_submission_registry_fixture();
    let (_tmp_inputs, inputs) = write_submission_inputs_fixture();
    let (_tmp_ledger, ledger) = write_submission_ledger_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("updated-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Submit {
            registry,
            inputs,
            ledger: Some(ledger),
            overrides: None,
            out: Some(out.clone()),
        },
    )
    .expect("schedule submit");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read written ledger"))
            .expect("parse written ledger");
    let entries = written["entries"].as_array().expect("ledger entries");
    assert_eq!(entries.len(), 4);

    let manual_entry = entries
        .iter()
        .find(|entry| entry["dedupe_key"] == "manual:manual-ops:manual-001")
        .expect("manual ledger entry");
    assert_eq!(manual_entry["graph_inputs"]["requested_at"], 175000u64);
    assert_eq!(manual_entry["graph_inputs"]["manual_region"], "eu-west-1");

    let event_entry = entries
        .iter()
        .find(|entry| entry["schedule_id"] == "event-ingest")
        .expect("event ledger entry");
    assert_eq!(event_entry["graph_inputs"]["event_tenant"], "atlas");
    assert_eq!(event_entry["graph_inputs"]["event_payload"]["batch"], 7);
    assert_eq!(event_entry["event_lineage"]["event_id"], "evt-001");
    assert_eq!(event_entry["event_lineage"]["event_type"], "dataset.ready");
    assert_eq!(event_entry["event_lineage"]["source"], "catalog");
    assert_eq!(event_entry["event_lineage"]["occurred_unix_ms"], 176000u64);

    let signal_entry = entries
        .iter()
        .find(|entry| entry["schedule_id"] == "signal-refresh")
        .expect("signal ledger entry");
    assert_eq!(signal_entry["graph_inputs"]["signal_tenant"], "atlas");
}

#[test]
fn schedule_submit_writes_dependency_triggered_entries() {
    let (_tmp_registry, registry) = write_dependency_submission_registry_fixture();
    let (_tmp_inputs, inputs) = write_dependency_submission_inputs_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("dependency-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Submit {
            registry,
            inputs,
            ledger: None,
            overrides: None,
            out: Some(out.clone()),
        },
    )
    .expect("dependency schedule submit");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read dependency ledger"))
            .expect("parse dependency ledger");
    let entries = written["entries"].as_array().expect("ledger entries");
    assert_eq!(entries.len(), 4);

    let success_entry = entries
        .iter()
        .find(|entry| entry["schedule_id"] == "dependency-on-success")
        .expect("success dependency entry");
    assert_eq!(success_entry["graph_inputs"]["dependency_run_id"], "atlas-run-success");
    assert_eq!(success_entry["graph_inputs"]["dependency_status"], "succeeded");

    let failure_entry = entries
        .iter()
        .find(|entry| entry["schedule_id"] == "dependency-on-failure")
        .expect("failure dependency entry");
    assert_eq!(failure_entry["graph_inputs"]["dependency_run_id"], "atlas-run-failure");
    assert_eq!(failure_entry["graph_inputs"]["dependency_status"], "timed_out");

    let terminal_entries = entries
        .iter()
        .filter(|entry| entry["schedule_id"] == "dependency-on-terminal")
        .collect::<Vec<_>>();
    assert_eq!(terminal_entries.len(), 2);
}

#[test]
fn schedule_submit_rejects_invalid_trigger_mapping_and_keeps_ledger_clean() {
    let (_tmp_registry, registry) = write_submission_rejection_registry_fixture();
    let (_tmp_inputs, inputs) = write_submission_rejection_inputs_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("updated-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Submit {
            registry,
            inputs,
            ledger: None,
            overrides: None,
            out: Some(out.clone()),
        },
    )
    .expect("schedule submit");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read written ledger"))
            .expect("parse written ledger");
    assert_eq!(written["entries"].as_array().expect("ledger entries").len(), 0);
}

#[test]
fn schedule_submit_respects_paused_schedule_overrides() {
    let (_tmp_registry, registry) = write_submission_registry_fixture();
    let (_tmp_inputs, inputs) = write_submission_inputs_fixture();
    let (_tmp_overrides, overrides) = write_schedule_override_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("paused-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Submit {
            registry,
            inputs,
            ledger: None,
            overrides: Some(overrides),
            out: Some(out.clone()),
        },
    )
    .expect("schedule submit with overrides");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read paused ledger"))
            .expect("parse paused ledger");
    let entries = written["entries"].as_array().expect("ledger entries");
    assert!(entries.iter().all(|entry| entry["schedule_id"] != "manual-ops"));
}

#[test]
fn schedule_queue_status_writes_reconstructed_state() {
    let (_tmp_registry, registry) = write_submission_registry_fixture();
    let (_tmp_ledger, ledger) = write_submission_ledger_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("queue-state.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Queue {
            command: ScheduleQueueCommands::Status {
                registry,
                ledger: Some(ledger),
                out: Some(out.clone()),
            },
        },
    )
    .expect("schedule queue status");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read queue state"))
            .expect("parse queue state");
    let queues = written["queues"].as_array().expect("queues");
    assert_eq!(queues.len(), 1);
    assert_eq!(queues[0]["queue_name"], "catalog");
    assert_eq!(queues[0]["active_runs"], 1);
    assert_eq!(queues[0]["available_slots"], 3);
    assert_eq!(queues[0]["tenants"][0]["tenant"], "atlas");
    assert_eq!(queues[0]["tenants"][0]["active_runs"], 1);
    assert_eq!(queues[0]["runs"][0]["starvation_ticks"], 0);
}

#[test]
fn schedule_control_status_reports_paused_schedule() {
    let (_tmp_registry, registry) = write_submission_registry_fixture();
    let (_tmp_overrides, overrides) = write_schedule_override_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("schedule-control-status.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Control {
            command: ScheduleControlCommands::Status {
                registry,
                overrides: Some(overrides),
                out: Some(out.clone()),
            },
        },
    )
    .expect("schedule control status");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read control status"))
            .expect("parse control status");
    let statuses = written.as_array().expect("schedule status");
    let manual =
        statuses.iter().find(|entry| entry["schedule_id"] == "manual-ops").expect("manual status");
    assert_eq!(manual["paused"], true);
    assert_eq!(manual["operator"], "atlas-ops");
}

#[test]
fn schedule_control_pause_and_resume_write_override_log() {
    let dir = tempfile::tempdir().expect("tmp");
    let overrides = dir.path().join("schedule-overrides.json");
    let cli = quiet_json_cli();

    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Control {
            command: ScheduleControlCommands::Pause {
                overrides: overrides.clone(),
                schedule_id: "manual-ops".to_string(),
                operator: "atlas-ops".to_string(),
                at_unix_ms: 180000,
                reason: Some("hold".to_string()),
                out: Some(overrides.clone()),
            },
        },
    )
    .expect("schedule control pause");
    assert_eq!(code, ExitCode::SUCCESS);

    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Control {
            command: ScheduleControlCommands::Resume {
                overrides: overrides.clone(),
                schedule_id: "manual-ops".to_string(),
                operator: "atlas-ops".to_string(),
                at_unix_ms: 190000,
                reason: Some("clear".to_string()),
                out: Some(overrides.clone()),
            },
        },
    )
    .expect("schedule control resume");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&overrides).expect("read overrides"))
            .expect("parse overrides");
    let records = written["records"].as_array().expect("override records");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["action"], "pause");
    assert_eq!(records[1]["action"], "resume");
    assert_eq!(records[1]["operator"], "atlas-ops");
}

#[test]
fn schedule_queue_dispatch_writes_updated_ledger() {
    let (_tmp_ledger, ledger) = write_submission_ledger_fixture();
    let (_tmp_policy, policy) = write_priority_dispatch_policy_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("dispatched-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Queue {
            command: ScheduleQueueCommands::Dispatch {
                ledger,
                max_dispatches: 1,
                policy: Some(policy),
                out: Some(out.clone()),
            },
        },
    )
    .expect("schedule queue dispatch");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read dispatched ledger"))
            .expect("parse dispatched ledger");
    assert_eq!(written["entries"][0]["status"], "Running");
    assert_eq!(written["entries"][0]["starvation_ticks"], 0);
}

#[test]
fn schedule_queue_update_writes_updated_ledger() {
    let (_tmp_ledger, ledger) = write_submission_ledger_fixture();
    let (_tmp_updates, updates) = write_submission_status_updates_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("updated-ledger.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Queue {
            command: ScheduleQueueCommands::Update { ledger, updates, out: Some(out.clone()) },
        },
    )
    .expect("schedule queue update");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read updated ledger"))
            .expect("parse updated ledger");
    assert_eq!(written["entries"][0]["status"], "Completed");
}

#[test]
fn schedule_validate_rejects_invalid_registry() {
    let (_tmp, registry) = write_invalid_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry }).unwrap_err();
    assert_eq!(code, ExitCode::from(3));
}

#[test]
fn schedule_validate_rejects_invalid_timezone_registry() {
    let (_tmp, registry) = write_invalid_timezone_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry }).unwrap_err();
    assert_eq!(code, ExitCode::from(3));
}

#[test]
fn schedule_preview_returns_success_for_valid_registry() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Preview { registry, now_unix_ms: 1_000, next_runs: 2 },
    )
    .expect("schedule preview");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_compile_returns_success_for_known_schedule() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Compile {
            registry,
            schedule_id: "nightly-catalog".to_string(),
            requested_unix_ms: 42,
        },
    )
    .expect("schedule compile");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_compile_rejects_bindings_that_need_missing_trigger_context() {
    let (_tmp, registry) = write_compile_rejection_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Compile {
            registry,
            schedule_id: "manual-ops".to_string(),
            requested_unix_ms: 42,
        },
    )
    .unwrap_err();
    assert_eq!(code, ExitCode::from(3));
}

#[test]
fn schedule_order_returns_success_for_priority_simulation() {
    let (_tmp, simulation) = write_ordering_simulation_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Order { simulation })
        .expect("schedule order");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_throttle_returns_success_for_backfill_simulation() {
    let (_tmp, simulation) = write_throttling_simulation_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Throttle { simulation })
        .expect("schedule throttle");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_audit_returns_success_for_valid_registry() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Audit { registry, now_unix_ms: 1_000, next_runs: 2 },
    )
    .expect("schedule audit");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_dedup_returns_success_for_event_stream() {
    let (_tmp, events) = write_dedup_simulation_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_schedule_command(&cli, &ScheduleCommands::Dedup { events }).expect("schedule dedup");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_sla_returns_success_for_metric_simulation() {
    let (_tmp, simulation) = write_sla_simulation_fixture();
    let cli = quiet_json_cli();
    let code =
        handle_schedule_command(&cli, &ScheduleCommands::Sla { simulation }).expect("schedule sla");
    assert_eq!(code, ExitCode::SUCCESS);
}

#[test]
fn schedule_backfill_plan_writes_operation_state() {
    let (_tmp, registry) = write_backfill_registry_fixture();
    let out_dir = tempfile::tempdir().expect("tmp");
    let out = out_dir.path().join("backfill-state.json");
    let cli = quiet_json_cli();
    let code = handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Plan {
                registry,
                schedule_id: "historical-catalog".to_string(),
                planned_unix_ms: 500,
                backfill_id: Some("catalog-history".to_string()),
                out: Some(out.clone()),
            },
        },
    )
    .expect("backfill plan");
    assert_eq!(code, ExitCode::SUCCESS);

    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read state")).expect("parse state");
    assert_eq!(payload["backfill_id"], "catalog-history");
    assert_eq!(payload["lifecycle"], "active");
    assert_eq!(payload["runs"].as_array().expect("runs").len(), 6);
}

#[test]
fn schedule_backfill_lifecycle_commands_update_state_and_dispatches() {
    let (_tmp_registry, registry) = write_backfill_registry_fixture();
    let state_dir = tempfile::tempdir().expect("tmp");
    let state = state_dir.path().join("backfill-state.json");
    let paused = state_dir.path().join("backfill-paused.json");
    let resumed = state_dir.path().join("backfill-resumed.json");
    let advanced = state_dir.path().join("backfill-advanced.json");
    let cancelled = state_dir.path().join("backfill-cancelled.json");
    let (_tmp_request, request) = write_backfill_advance_request_fixture();
    let cli = quiet_json_cli();

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Plan {
                registry,
                schedule_id: "historical-catalog".to_string(),
                planned_unix_ms: 500,
                backfill_id: None,
                out: Some(state.clone()),
            },
        },
    )
    .expect("plan state");
    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Pause {
                state: state.clone(),
                at_unix_ms: 1_000,
                reason: Some("operator hold".to_string()),
                out: Some(paused.clone()),
            },
        },
    )
    .expect("pause state");
    let paused_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&paused).expect("read paused"))
            .expect("parse paused");
    assert_eq!(paused_payload["lifecycle"], "paused");

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Resume {
                state: paused.clone(),
                at_unix_ms: 1_500,
                out: Some(resumed.clone()),
            },
        },
    )
    .expect("resume state");
    let resumed_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&resumed).expect("read resumed"))
            .expect("parse resumed");
    assert_eq!(resumed_payload["lifecycle"], "active");

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Advance {
                state: resumed.clone(),
                request,
                out: Some(advanced.clone()),
            },
        },
    )
    .expect("advance state");
    let advanced_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&advanced).expect("read advanced"))
            .expect("parse advanced");
    let submitted = advanced_payload["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .filter(|run| run["status"] == "submitted")
        .count();
    assert_eq!(submitted, 2);

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Cancel {
                state: advanced.clone(),
                at_unix_ms: 2_000,
                reason: Some("operator stop".to_string()),
                out: Some(cancelled.clone()),
            },
        },
    )
    .expect("cancel state");
    let cancelled_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&cancelled).expect("read cancelled"))
            .expect("parse cancelled");
    assert_eq!(cancelled_payload["lifecycle"], "cancelled");
    assert!(cancelled_payload["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .any(|run| run["status"] == "cancelled"));
}

#[test]
fn schedule_backfill_summary_reports_aggregate_operation_state() {
    let (_tmp, registry) = write_backfill_registry_fixture();
    let state_dir = tempfile::tempdir().expect("tmp");
    let state = state_dir.path().join("backfill-state.json");
    let summary = state_dir.path().join("backfill-summary.json");
    let cli = quiet_json_cli();

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Plan {
                registry,
                schedule_id: "historical-catalog".to_string(),
                planned_unix_ms: 500,
                backfill_id: Some("catalog-history".to_string()),
                out: Some(state.clone()),
            },
        },
    )
    .expect("plan state");
    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Summary { state, out: Some(summary.clone()) },
        },
    )
    .expect("summary state");

    let summary_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&summary).expect("read summary"))
            .expect("parse summary");
    assert_eq!(summary_payload["backfill_id"], "catalog-history");
    assert_eq!(summary_payload["total_runs"], 6);
    assert_eq!(summary_payload["queued_runs"], 6);
    assert_eq!(summary_payload["failed_runs"], 0);
    assert_eq!(summary_payload["total_retry_attempts"], 0);
    assert_eq!(summary_payload["partitions"].as_array().expect("partitions").len(), 6);
}

#[test]
fn schedule_backfill_retry_failed_requeues_partition_with_attempt_history() {
    let (_tmp_registry, registry) = write_backfill_registry_fixture();
    let state_dir = tempfile::tempdir().expect("tmp");
    let state = state_dir.path().join("backfill-state.json");
    let dispatched = state_dir.path().join("backfill-dispatched.json");
    let failed_request = state_dir.path().join("backfill-failed-request.json");
    let failed_state = state_dir.path().join("backfill-failed.json");
    let retried_state = state_dir.path().join("backfill-retried.json");
    let (_tmp_request, request) = write_backfill_advance_request_fixture();
    let cli = quiet_json_cli();

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Plan {
                registry,
                schedule_id: "historical-catalog".to_string(),
                planned_unix_ms: 500,
                backfill_id: Some("catalog-history".to_string()),
                out: Some(state.clone()),
            },
        },
    )
    .expect("plan state");
    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Advance {
                state: state.clone(),
                request,
                out: Some(dispatched.clone()),
            },
        },
    )
    .expect("dispatch state");

    let dispatched_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&dispatched).expect("read dispatched"))
            .expect("parse dispatched");
    let failed_run_id = dispatched_payload["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| run["status"] == "submitted" && run["partition_key"] == "sample-a")
        .and_then(|run| run["run_id"].as_str())
        .expect("failed run id")
        .to_string();

    fs::write(
        &failed_request,
        serde_json::to_vec_pretty(&serde_json::json!({
            "now_unix_ms": 2800,
            "pending_live_runs": 0,
            "throttling_policy": {
                "max_backfill_submissions_per_tick": 4,
                "reserve_live_capacity_percent": 0
            },
            "status_updates": [
                {
                    "run_id": failed_run_id,
                    "status": "failed",
                    "updated_unix_ms": 2700
                }
            ]
        }))
        .expect("failed request"),
    )
    .expect("write failed request");

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::Advance {
                state: dispatched.clone(),
                request: failed_request,
                out: Some(failed_state.clone()),
            },
        },
    )
    .expect("fail state");
    let failed_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&failed_state).expect("read failed state"))
            .expect("parse failed state");
    assert_eq!(failed_payload["lifecycle"], "paused");

    handle_schedule_command(
        &cli,
        &ScheduleCommands::Backfill {
            command: ScheduleBackfillCommands::RetryFailed {
                state: failed_state.clone(),
                at_unix_ms: 3_000,
                out: Some(retried_state.clone()),
            },
        },
    )
    .expect("retry failed state");
    let retried_payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&retried_state).expect("read retried state"))
            .expect("parse retried state");
    assert_eq!(retried_payload["lifecycle"], "active");

    let retried_run = retried_payload["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .find(|run| run["partition_key"] == "sample-a" && run["requested_unix_ms"] == 1000)
        .expect("retried partition");
    assert_eq!(retried_run["status"], "queued");
    assert_eq!(retried_run["attempt"], 2);
    assert_eq!(
        retried_run["previous_run_ids"].as_array().expect("previous run ids")[0],
        serde_json::json!(failed_run_id)
    );
    assert_ne!(retried_run["run_id"], retried_run["previous_run_ids"][0]);
}
