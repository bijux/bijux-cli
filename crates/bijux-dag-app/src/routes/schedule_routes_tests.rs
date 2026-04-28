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
