use super::handle_schedule_command;
use crate::commands::{Commands, DagCli, ScheduleCommands};
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
