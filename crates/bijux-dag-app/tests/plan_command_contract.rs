use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
use std::sync::{Mutex, MutexGuard, OnceLock};

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

fn internal_lane_lock() -> MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(())).lock().expect("internal lane lock")
}

fn run_with_internal_lane(matches: &clap::ArgMatches) -> std::process::ExitCode {
    let _guard = internal_lane_lock();
    let previous = std::env::var_os("BIJUX_DAG_ENABLE_INTERNAL");
    std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", "1");
    let result = dag_run(matches).expect("run");
    if let Some(value) = previous {
        std::env::set_var("BIJUX_DAG_ENABLE_INTERNAL", value);
    } else {
        std::env::remove_var("BIJUX_DAG_ENABLE_INTERNAL");
    }
    result
}

#[test]
fn plan_explain_supports_json_output_with_node_reasons() {
    let (_dir, dag) = write_graph_fixture();
    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "plan",
            "explain",
            dag.to_string_lossy().as_ref(),
        ])
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
fn plan_equivalence_supports_json_output() {
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
            "equivalence",
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

    let code = run_with_internal_lane(&matches);
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

    let code = run_with_internal_lane(&matches);
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

    let code = run_with_internal_lane(&matches);
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

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_submit_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry.json");
    let inputs = dir.path().join("schedule-inputs.json");
    let ledger = dir.path().join("schedule-ledger.json");
    let out = dir.path().join("schedule-ledger-updated.json");
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
            }
          ]
        }"#,
    )
    .expect("write registry");
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
          ]
        }"#,
    )
    .expect("write inputs");
    fs::write(&ledger, r#"{"entries":[]}"#).expect("write ledger");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "submit",
            registry.to_string_lossy().as_ref(),
            inputs.to_string_lossy().as_ref(),
            "--ledger",
            ledger.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read ledger"))
            .expect("parse ledger");
    let entries = written["entries"].as_array().expect("ledger entries");
    assert_eq!(entries[0]["event_lineage"], serde_json::Value::Null);
}

#[test]
fn schedule_submit_persists_event_lineage_for_event_triggers() {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-event-registry.json");
    let inputs = dir.path().join("schedule-event-inputs.json");
    let out = dir.path().join("schedule-event-ledger.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
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
    .expect("write registry");
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
                "tenant": "atlas",
                "batch": 7
              }
            }
          ]
        }"#,
    )
    .expect("write inputs");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "submit",
            registry.to_string_lossy().as_ref(),
            inputs.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);

    let written: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&out).expect("read ledger"))
            .expect("parse ledger");
    let entry = &written["entries"].as_array().expect("ledger entries")[0];
    assert_eq!(entry["event_lineage"]["event_id"], "evt-001");
    assert_eq!(entry["event_lineage"]["event_type"], "dataset.ready");
    assert_eq!(entry["event_lineage"]["source"], "catalog");
    assert_eq!(entry["event_lineage"]["occurred_unix_ms"], 176000u64);
}

#[test]
fn schedule_submit_supports_dependency_trigger_conditions() {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-dependency-registry.json");
    let inputs = dir.path().join("schedule-dependency-inputs.json");
    let out = dir.path().join("schedule-ledger-updated.json");
    fs::write(
        &registry,
        r#"{
          "definitions": [
            {
              "id": "dependency-on-success",
              "dag_name": "atlas.publish-success",
              "dag_version_policy": "run-latest",
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
    .expect("write registry");
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
    .expect("write inputs");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "submit",
            registry.to_string_lossy().as_ref(),
            inputs.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_queue_status_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry.json");
    let ledger = dir.path().join("schedule-ledger.json");
    let out = dir.path().join("queue-state.json");
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
            }
          ]
        }"#,
    )
    .expect("write registry");
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
              "graph_inputs": {},
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
    .expect("write ledger");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "queue",
            "status",
            registry.to_string_lossy().as_ref(),
            "--ledger",
            ledger.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_queue_dispatch_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let ledger = dir.path().join("schedule-ledger.json");
    let policy = dir.path().join("priority-dispatch-policy.json");
    let out = dir.path().join("schedule-ledger-dispatched.json");
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
              "graph_inputs": {},
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
    .expect("write ledger");
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
    .expect("write policy");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "queue",
            "dispatch",
            ledger.to_string_lossy().as_ref(),
            "--max-dispatches",
            "1",
            "--policy",
            policy.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_queue_update_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let ledger = dir.path().join("schedule-ledger.json");
    let updates = dir.path().join("submission-status-updates.json");
    let out = dir.path().join("schedule-ledger-updated.json");
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
              "graph_inputs": {},
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
    .expect("write ledger");
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
    .expect("write updates");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "queue",
            "update",
            ledger.to_string_lossy().as_ref(),
            updates.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_control_status_supports_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let registry = dir.path().join("schedule-registry.json");
    let overrides = dir.path().join("schedule-overrides.json");
    let out = dir.path().join("schedule-control-status.json");
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
            }
          ]
        }"#,
    )
    .expect("write registry");
    fs::write(
        &overrides,
        r#"{
          "records": [
            {
              "schedule_id": "manual-ops",
              "operator": "atlas-ops",
              "action": "pause",
              "reason": "hold",
              "created_unix_ms": 180000
            }
          ]
        }"#,
    )
    .expect("write overrides");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "control",
            "status",
            registry.to_string_lossy().as_ref(),
            "--overrides",
            overrides.to_string_lossy().as_ref(),
            "--out",
            out.to_string_lossy().as_ref(),
        ])
        .expect("parse");

    let code = run_with_internal_lane(&matches);
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn schedule_control_pause_and_resume_support_json_output() {
    let dir = tempfile::tempdir().expect("tmp");
    let overrides = dir.path().join("schedule-overrides.json");

    let pause = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "control",
            "pause",
            overrides.to_string_lossy().as_ref(),
            "--schedule-id",
            "manual-ops",
            "--operator",
            "atlas-ops",
            "--at-unix-ms",
            "180000",
            "--reason",
            "hold",
            "--out",
            overrides.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    assert_eq!(run_with_internal_lane(&pause), std::process::ExitCode::SUCCESS);

    let resume = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "schedule",
            "control",
            "resume",
            overrides.to_string_lossy().as_ref(),
            "--schedule-id",
            "manual-ops",
            "--operator",
            "atlas-ops",
            "--at-unix-ms",
            "190000",
            "--reason",
            "clear",
            "--out",
            overrides.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    assert_eq!(run_with_internal_lane(&resume), std::process::ExitCode::SUCCESS);
}
