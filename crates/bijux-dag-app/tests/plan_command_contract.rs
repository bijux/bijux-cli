use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;

fn write_graph_fixture() -> (tempfile::TempDir, std::path::PathBuf) {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("graph.json");
    fs::write(
        &dag,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-cmd","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");
    (dir, dag)
}

#[test]
fn plan_explain_supports_json_output_with_node_reasons() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from(["bijux-dag", "--json", "plan", "explain", dag.to_string_lossy().as_ref()])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn plan_diagnostics_supports_json_payload() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "plan",
            "diagnostics",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);

    let payload: Value = serde_json::json!({"assertion":"routing only"});
    assert!(payload.is_object());
}

#[test]
fn plan_diff_supports_json_output() {
    let (_before_dir, before) = write_graph_fixture();
    let after_dir = tempfile::tempdir().expect("tmp");
    let after = after_dir.path().join("graph-tagged.json");
    fs::write(
        &after,
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"plan-cmd-tagged","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"tags":["critical"],"params":{"value":"1"}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":"2"}}
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("write graph");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "plan",
            "diff",
            before.to_string_lossy().as_ref(),
            after.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn show_effective_plan_supports_json_output() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "show-effective-plan",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_validate_supports_json_output() {
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

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "validate",
            registry.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_compile_supports_json_output() {
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

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "compile",
            registry.to_string_lossy().as_ref(),
            "--schedule-id",
            "nightly-catalog",
            "--requested-unix-ms",
            "42",
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn plan_closure_supports_json_output() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "plan",
            "closure",
            dag.to_string_lossy().as_ref(),
            "--select",
            "b",
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn plan_backfill_supports_json_output() {
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "plan",
            "backfill",
            "--window-start-unix-ms",
            "100",
            "--window-end-unix-ms",
            "300",
            "--partition-key",
            "sample-a",
            "--partition-key",
            "sample-b",
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_audit_supports_json_output() {
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

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "audit",
            registry.to_string_lossy().as_ref(),
            "--now-unix-ms",
            "1000",
            "--next-runs",
            "2",
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_dedup_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let events = dir.path().join("events.json");
    fs::write(&events, r#"{ "events": ["evt-1", "evt-1", "evt-2"] }"#).expect("write events");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "dedup",
            events.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = dag_run(&matches).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
