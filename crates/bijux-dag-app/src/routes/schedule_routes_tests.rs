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

#[test]
fn schedule_validate_returns_success_for_valid_registry() {
    let (_tmp, registry) = write_registry_fixture();
    let cli = quiet_json_cli();
    let code = handle_schedule_command(&cli, &ScheduleCommands::Validate { registry })
        .expect("schedule validate");
    assert_eq!(code, ExitCode::SUCCESS);
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
