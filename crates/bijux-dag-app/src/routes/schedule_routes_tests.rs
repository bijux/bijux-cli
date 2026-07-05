use super::handle_schedule_command;
use crate::commands::{Commands, DagCli, ScheduleBackfillCommands, ScheduleCommands};
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
    .expect("write submit registry");
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
              "requested_unix_ms": 175000
            }
          ],
          "events": [
            {
              "event_id": "evt-001",
              "event_type": "dataset.ready",
              "source": "catalog",
              "occurred_unix_ms": 176000
            }
          ]
        }"#,
    )
    .expect("write submit inputs");
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
              "requested_unix_ms": 175000,
              "created_unix_ms": 170000,
              "run_id": "sched-manual-ops-existing",
              "trigger_kind": "manual",
              "dedupe_key": "manual:manual-ops:manual-001",
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

#[test]
fn schedule_validate_returns_success_for_valid_registry() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry })
        .expect("schedule validate");
    assert_eq!(code, ExitCode::SUCCESS);
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
            out: Some(out.clone()),
        },
    )
    .expect("schedule submit");
    assert_eq!(code, ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read written ledger"))
            .expect("parse written ledger");
    let entries = written["entries"].as_array().expect("ledger entries");
    assert_eq!(entries.len(), 2);
    assert!(entries.iter().any(|entry| entry["schedule_id"] == "event-ingest"));
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
    let paused_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&paused).expect("read paused"),
    )
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
    let resumed_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&resumed).expect("read resumed"),
    )
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
    let advanced_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&advanced).expect("read advanced"),
    )
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
    let cancelled_payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(&cancelled).expect("read cancelled"),
    )
    .expect("parse cancelled");
    assert_eq!(cancelled_payload["lifecycle"], "cancelled");
    assert!(cancelled_payload["runs"]
        .as_array()
        .expect("runs")
        .iter()
        .any(|run| run["status"] == "cancelled"));
}
