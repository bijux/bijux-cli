use base64 as _;
use bijux_dag_app::{
    dag_command, dag_run, explain_run_id, inspect_summary, run_timeline, runs_history,
    runs_history_query,
};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::Path;
use std::sync::{Arc, Barrier};

fn write_run_manifest(
    path: &Path,
    run_id: &str,
    status: &str,
    created_unix_ms: u64,
    run_metadata: Value,
) {
    fs::create_dir_all(path).expect("mkdir");
    fs::write(
        path.join("manifest.json"),
        serde_json::to_vec_pretty(&json!({
            "manifest_version":"run-manifest/v0.1",
            "run_id": run_id,
            "created_unix_ms": created_unix_ms,
            "started_unix_ms": created_unix_ms + 1,
            "finished_unix_ms": created_unix_ms + 2,
            "graph_snapshot":"graph.snapshot.json",
            "status": status,
            "spec":"bijux-dag/v0.1",
            "graph_fingerprint":"g",
            "tool_version":"0.1.0",
            "jobs":1,
            "adapters":[],
            "outputs":[],
            "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
            "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
            "run_metadata": run_metadata
        }))
        .expect("manifest"),
    )
    .expect("write");
}

fn write_required_run_files(run_dir: &Path, failed: bool) {
    fs::write(run_dir.join("graph.snapshot.json"), "{}").expect("graph snapshot");
    fs::write(run_dir.join("snapshot.json"), "{}").expect("snapshot");
    fs::create_dir_all(run_dir.join("outputs")).expect("outputs");
    fs::write(run_dir.join("outputs").join("index.json"), "{\"files\":[]}").expect("outputs index");
    fs::write(run_dir.join("outputs.index.json"), "{\"files\":[]}").expect("legacy outputs index");
    fs::create_dir_all(run_dir.join("nodes").join("a")).expect("node dir");
    fs::write(
        run_dir.join("nodes").join("a").join("trace.json"),
        serde_json::to_vec_pretty(&json!({
            "node_id":"a",
            "status": if failed { "failed" } else { "success" },
            "started_unix_ms": 10,
            "finished_unix_ms": 11,
            "attempt": 1,
            "fingerprint": "nfp"
        }))
        .expect("trace"),
    )
    .expect("trace write");
    fs::write(run_dir.join("observability.timeline.json"), "{}").expect("timeline");
    fs::write(run_dir.join("observability.events.json"), "{}").expect("events");
    if failed {
        fs::write(run_dir.join("observability.root-causes.json"), "{}").expect("root causes");
    }
}

#[test]
fn history_query_supports_status_filter_and_pagination_contract() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    for (run_id, status, created) in [
        ("r1", "success", 1_u64),
        ("r2", "failed", 2),
        ("r3", "success", 3),
        ("r4", "cancelled", 4),
    ] {
        write_run_manifest(
            &root.join(format!("run-{run_id}")),
            run_id,
            status,
            created,
            json!({
                "submission_source":"manual",
                "trigger_source":"cli",
                "operator":"tester",
                "labels":[]
            }),
        );
    }

    let filtered = runs_history_query(&root, Some("success"), None, None).expect("history");
    assert_eq!(filtered["runs"].as_array().expect("runs").len(), 2);

    let paged = runs_history_query(&root, None, None, Some((1, 2))).expect("paged");
    assert_eq!(paged["runs"].as_array().expect("runs").len(), 2);
    assert_eq!(paged["page"]["offset"], 1);
    assert_eq!(paged["page"]["limit"], 2);
    assert_eq!(paged["page"]["total"], 4);
}

#[test]
fn mixed_local_imported_replayed_and_drifted_history_fixture_is_stable() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let fixtures: Value =
        serde_json::from_str(include_str!("fixtures/run_history_mixed_runs.json"))
            .expect("fixtures");

    for row in fixtures.as_array().expect("array") {
        let dir = root.join(row["dir"].as_str().expect("dir"));
        write_run_manifest(
            &dir,
            row["run_id"].as_str().expect("run id"),
            row["status"].as_str().expect("status"),
            row["created_unix_ms"].as_u64().expect("created"),
            json!({
                "submission_source": row["run_metadata"]["submission_source"],
                "trigger_source": row["run_metadata"]["trigger_source"],
                "operator":"fixture",
                "labels":[],
                "parent_run_id": row["run_metadata"]["parent_run_id"],
                "source_run_id": row["run_metadata"]["source_run_id"]
            }),
        );
    }

    let first = runs_history(&root).expect("history");
    let second = runs_history(&root).expect("history");
    assert_eq!(first, second, "history traversal must stay deterministic");
    assert_eq!(first["runs"].as_array().expect("rows").len(), 4);
}

#[test]
fn ancestry_fields_are_present_for_failed_cancelled_and_partial_replay_runs() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");

    write_run_manifest(
        &root.join("run-failed"),
        "failed",
        "failed",
        10,
        json!({
            "submission_source":"replay",
            "trigger_source":"cli",
            "operator":"tester",
            "labels":[],
            "parent_run_id":"src-run",
            "source_run_id":"src-run"
        }),
    );
    write_run_manifest(
        &root.join("run-cancelled"),
        "cancelled",
        "cancelled",
        20,
        json!({
            "submission_source":"replay",
            "trigger_source":"cli",
            "operator":"tester",
            "labels":[],
            "parent_run_id":"failed",
            "source_run_id":"src-run"
        }),
    );
    write_run_manifest(
        &root.join("run-partial"),
        "partial",
        "success",
        30,
        json!({
            "submission_source":"replay",
            "trigger_source":"import",
            "operator":"tester",
            "labels":[],
            "parent_run_id":"cancelled",
            "source_run_id":"imported-run"
        }),
    );

    let history = runs_history(&root).expect("history");
    let rows = history["runs"].as_array().expect("rows");
    assert!(rows.iter().any(|r| r["run_id"] == "failed" && r["source_run_id"] == "src-run"));
    assert!(rows.iter().any(|r| r["run_id"] == "cancelled" && r["parent_run_id"] == "failed"));
    assert!(rows.iter().any(|r| r["run_id"] == "partial" && r["source_run_id"] == "imported-run"));
}

#[test]
fn latest_pointer_concurrent_updates_do_not_corrupt_history() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    write_run_manifest(
        &root.join("run-a"),
        "a",
        "success",
        1,
        json!({"submission_source":"manual","trigger_source":"cli","operator":"x","labels":[]}),
    );
    write_run_manifest(
        &root.join("run-b"),
        "b",
        "success",
        2,
        json!({"submission_source":"manual","trigger_source":"cli","operator":"x","labels":[]}),
    );

    let latest_path = root.join("latest");
    let start = Arc::new(Barrier::new(5));
    let mut threads = Vec::new();
    for idx in 0..4 {
        let latest = latest_path.clone();
        let sync = start.clone();
        threads.push(std::thread::spawn(move || {
            sync.wait();
            let target = if idx % 2 == 0 { "run-a" } else { "run-b" };
            for _ in 0..50 {
                let _ = fs::write(&latest, target);
            }
        }));
    }
    start.wait();
    for t in threads {
        t.join().expect("join");
    }

    let history = runs_history(&root).expect("history");
    assert_eq!(history["runs"].as_array().expect("rows").len(), 2);
}

#[test]
fn strict_verify_rejects_tampered_timestamps_environment_summary_and_missing_events() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-stable");
    write_run_manifest(
        &run,
        "stable",
        "success",
        100,
        json!({
            "submission_source":"manual",
            "trigger_source":"cli",
            "operator":"tester",
            "labels":[],
            "environment_summary":{"os":"linux","env":"clean"},
            "environment_summary_sha256":"bad-hash"
        }),
    );
    write_required_run_files(&run, false);

    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(run.join("manifest.json")).expect("manifest"))
            .expect("json");
    manifest["started_unix_ms"] = json!(9999);
    manifest["finished_unix_ms"] = json!(1000);
    fs::write(run.join("manifest.json"), serde_json::to_vec_pretty(&manifest).expect("encode"))
        .expect("write");
    fs::remove_file(run.join("observability.events.json")).expect("remove events");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "runs",
            "verify",
            "stable",
            "--root",
            root.to_string_lossy().as_ref(),
            "--strict",
            "--json",
        ])
        .expect("parse");
    let exit = dag_run(&matches).expect_err("run verify should fail");
    assert_eq!(exit, std::process::ExitCode::from(3));
}

#[test]
fn strict_verify_rejects_run_id_mutation_after_completion() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-frozen");
    write_run_manifest(
        &run,
        "different-id",
        "success",
        10,
        json!({"submission_source":"manual","trigger_source":"cli","operator":"tester","labels":[]}),
    );
    write_required_run_files(&run, false);

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "runs",
            "verify",
            "frozen",
            "--root",
            root.to_string_lossy().as_ref(),
            "--strict",
            "--json",
        ])
        .expect("parse");
    let exit = dag_run(&matches).expect_err("run verify should fail");
    assert_eq!(exit, std::process::ExitCode::from(3));
}

#[test]
fn strict_verify_reports_missing_event_traces_referenced_by_manifest() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-missing-events");
    write_run_manifest(
        &run,
        "missing-events",
        "failed",
        50,
        json!({"submission_source":"manual","trigger_source":"cli","operator":"tester","labels":[]}),
    );
    fs::write(run.join("graph.snapshot.json"), "{}").expect("graph snapshot");
    fs::write(run.join("snapshot.json"), "{}").expect("snapshot");
    fs::create_dir_all(run.join("outputs")).expect("outputs");
    fs::write(run.join("outputs").join("index.json"), "{\"files\":[]}").expect("outputs index");
    fs::write(run.join("outputs.index.json"), "{\"files\":[]}").expect("legacy outputs index");
    fs::create_dir_all(run.join("nodes")).expect("nodes");

    let matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "runs",
            "verify",
            "missing-events",
            "--root",
            root.to_string_lossy().as_ref(),
            "--strict",
            "--json",
        ])
        .expect("parse");
    let exit = dag_run(&matches).expect_err("run verify should fail");
    assert_eq!(exit, std::process::ExitCode::from(3));
}

#[test]
fn damaged_run_directories_return_errors_without_panics() {
    let tmp = tempfile::tempdir().expect("tmp");
    let root = tmp.path().join("runs");
    let run = root.join("run-damaged");
    fs::create_dir_all(&run).expect("mkdir");
    fs::write(run.join("manifest.json"), "{bad-json").expect("manifest");
    fs::create_dir_all(run.join("nodes").join("node-a")).expect("nodes");
    fs::write(run.join("nodes").join("node-a").join("trace.json"), "{bad-json").expect("trace");

    assert!(inspect_summary(&run).is_ok(), "inspect summary should not panic on damaged run");
    assert!(run_timeline(&run).is_ok(), "timeline should not panic on damaged run");
    assert!(runs_history(&root).is_ok(), "history should not panic on damaged run set");
    assert!(explain_run_id(&root, "run-damaged").is_ok(), "id explain should not panic");
}
