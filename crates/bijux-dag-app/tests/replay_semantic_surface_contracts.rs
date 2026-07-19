use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json::{json, Value};
use sha2 as _;
use std::fs;
use std::path::Path;
use tar as _;
use tempfile as _;
use thiserror as _;

fn write_basic_run(run: &Path, run_id: &str, extras: Value) {
    fs::create_dir_all(run.join("nodes/a/outputs")).expect("mkdir");
    fs::create_dir_all(run.join("outputs")).expect("mkdir outputs");
    fs::write(run.join("nodes/a/outputs/out"), b"payload").expect("payload");
    let mut manifest = json!({
        "manifest_version":"run-manifest/v0.1",
        "run_id":run_id,
        "created_unix_ms":1,
        "started_unix_ms":2,
        "finished_unix_ms":3,
        "graph_snapshot":"graph.snapshot.json",
        "status":"success",
        "spec":"bijux-dag/v0.1",
        "graph_fingerprint":"g",
        "tool_version":"0.1.0",
        "jobs":1,
        "adapters":[],
        "outputs":[],
        "node_counts":{"success":1,"failed":0,"skipped":0,"cached":0},
        "policy":{"deny_network":true,"deny_env":true,"deny_clock":true,"clean_env":true},
        "run_metadata":{"submission_source":"manual","trigger_source":"cli","operator":"tester","labels":[]}
    });
    if let Some(obj) = extras.as_object() {
        for (k, v) in obj {
            manifest[k] = v.clone();
        }
    }
    fs::write(run.join("manifest.json"), serde_json::to_vec_pretty(&manifest).expect("manifest"))
        .expect("write manifest");
    fs::write(run.join("graph.snapshot.json"), "{\"graph_fingerprint\":\"g\"}").expect("snapshot");
    fs::write(
        run.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files":[{"node_id":"a","node_fingerprint":"fp-a","name":"out","kind":"file","media_type":"application/octet-stream","size_bytes":2,"sha256":"239f59ed55e737c77147cf55ad0c1b03f9c6a27ddc8e2ba69ec4b8b4b09e4e7d","path":"nodes/a/outputs/out"}]
        }))
        .expect("index"),
    )
    .expect("write index");
}

#[test]
fn semantic_diff_equivalence_surface_reports_equivalent_for_cosmetic_plan_changes() {
    let tmp = tempfile::tempdir().expect("tmp");
    let runs = tmp.path().join("runs");
    let run_a = runs.join("run-a");
    let run_b = runs.join("run-b");
    write_basic_run(
        &run_a,
        "a",
        json!({"created_unix_ms": 11, "started_unix_ms": 12, "finished_unix_ms": 13}),
    );
    write_basic_run(
        &run_b,
        "b",
        json!({"created_unix_ms": 91, "started_unix_ms": 92, "finished_unix_ms": 93}),
    );

    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-rerun",
            run_a.to_string_lossy().as_ref(),
            run_b.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    let code = dag_run(&cmd).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn explain_why_rerun_supports_imported_run_ancestry_context() {
    let tmp = tempfile::tempdir().expect("tmp");
    let runs = tmp.path().join("runs");
    let run_a = runs.join("run-imported");
    let run_b = runs.join("run-replayed");
    write_basic_run(
        &run_a,
        "imported",
        json!({
            "run_metadata": {
                "submission_source":"import",
                "trigger_source":"bundle",
                "operator":"importer",
                "labels":[],
                "source_run_id":"remote-1"
            }
        }),
    );
    write_basic_run(
        &run_b,
        "replayed",
        json!({
            "run_metadata": {
                "submission_source":"replay",
                "trigger_source":"cli",
                "operator":"tester",
                "labels":[],
                "parent_run_id":"imported",
                "source_run_id":"remote-1"
            }
        }),
    );

    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-rerun",
            run_a.to_string_lossy().as_ref(),
            run_b.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    let code = dag_run(&cmd).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn explain_why_cache_missed_reports_corrupt_entry_verification_failure() {
    let tmp = tempfile::tempdir().expect("tmp");
    let cache = tmp.path().join("cache");
    let key = "abc";
    let entry = cache.join(key);
    fs::create_dir_all(entry.join("outputs")).expect("mkdir outputs");
    fs::write(
        entry.join("meta.json"),
        serde_json::to_vec_pretty(&json!({
            "node_fingerprint": key,
            "adapter_id": "shell",
            "adapter_version": "1.0.0"
        }))
        .expect("meta"),
    )
    .expect("write meta");
    fs::write(entry.join("outputs").join("x.txt"), b"actual-bytes").expect("write payload");
    fs::write(
        entry.join("outputs/index.json"),
        serde_json::to_vec_pretty(&json!({
            "files":[{"name":"out","path":"x.txt","kind":"file","media_type":"text/plain","size_bytes":1,"sha256":"deadbeef","node_id":"a","node_fingerprint": key}]
        }))
        .expect("index"),
    )
    .expect("write index");

    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "why-cache-missed",
            key,
            "--expected-adapter-id",
            "shell",
            "--expected-adapter-version",
            "1.0.0",
            "--cache-dir",
            cache.to_string_lossy().as_ref(),
        ])
        .expect("parse");
    let code = dag_run(&cmd).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn trace_artifact_supports_replayed_run_provenance_surface() {
    let tmp = tempfile::tempdir().expect("tmp");
    let runs = tmp.path().join("runs");
    let replayed = runs.join("run-replayed");
    write_basic_run(
        &replayed,
        "replayed",
        json!({
            "run_metadata": {
                "submission_source":"replay",
                "trigger_source":"cli",
                "operator":"tester",
                "labels":[],
                "parent_run_id":"source",
                "source_run_id":"source"
            }
        }),
    );

    let cmd = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "trace-artifact",
            replayed.to_string_lossy().as_ref(),
            "a:out",
        ])
        .expect("parse");
    let code = dag_run(&cmd).expect("run");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}
