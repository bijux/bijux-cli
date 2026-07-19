use base64 as _;
use bijux_dag_app::{dag_command, dag_run};
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::process::ExitCode;

fn write_graph(dir: &std::path::Path, file: &str, spec: &str) -> std::path::PathBuf {
    let dag = dir.join(file);
    fs::write(
        &dag,
        format!(
            "{{\"spec\":\"{}\",\"meta\":{{\"name\":\"g\",\"owners\":[],\"tags\":[]}},\"nodes\":[],\"edges\":[]}}",
            spec
        ),
    )
    .expect("write graph");
    dag
}

#[test]
fn app_graph_loading_from_filesystem_and_canonical_bytes_succeeds() {
    let dir = tempfile::tempdir().expect("tempdir");
    let dag = write_graph(dir.path(), "dag.json", "bijux-dag/v0.1");

    let validate_matches = dag_command()
        .try_get_matches_from(["bijux-dag", "validate", dag.to_string_lossy().as_ref()])
        .expect("parse validate args");
    let validate_code = dag_run(&validate_matches).expect("validate run");
    assert_eq!(validate_code, ExitCode::SUCCESS);

    let canonical_matches = dag_command()
        .try_get_matches_from(["bijux-dag", "canonical-bytes", dag.to_string_lossy().as_ref()])
        .expect("parse canonical bytes args");
    let canonical_code = dag_run(&canonical_matches).expect("canonical bytes run");
    assert_eq!(canonical_code, ExitCode::SUCCESS);
}

#[test]
fn graph_loading_error_and_version_rejection_are_classified() {
    let dir = tempfile::tempdir().expect("tempdir");
    let malformed = dir.path().join("bad.json");
    fs::write(&malformed, "{not-json").expect("write malformed");

    let malformed_matches = dag_command()
        .try_get_matches_from(["bijux-dag", "validate", malformed.to_string_lossy().as_ref()])
        .expect("parse malformed args");
    let malformed_result = dag_run(&malformed_matches);
    assert_eq!(malformed_result, Err(ExitCode::from(2)));

    let unsupported = write_graph(dir.path(), "unsupported.json", "v9");
    let unsupported_matches = dag_command()
        .try_get_matches_from(["bijux-dag", "validate", unsupported.to_string_lossy().as_ref()])
        .expect("parse unsupported args");
    let unsupported_result = dag_run(&unsupported_matches);
    assert_eq!(unsupported_result, Err(ExitCode::from(1)));
}

#[test]
fn import_and_replay_graph_load_flows_fail_cleanly_when_inputs_are_incomplete() {
    let dir = tempfile::tempdir().expect("tempdir");
    let bundle = dir.path().join("bundle.json");
    fs::write(
        &bundle,
        r#"{"bundle_version":"export-bundle/v0.1","export_mode":"manifest-only","manifest":{},"node_traces":{},"outputs":{}}"#,
    )
    .expect("write bundle");

    let import_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "import",
            bundle.to_string_lossy().as_ref(),
            "--verify-only",
        ])
        .expect("parse import args");
    let import_result = dag_run(&import_matches);
    assert_eq!(import_result, Err(ExitCode::from(3)));

    let run_dir = dir.path().join("run-001");
    fs::create_dir_all(&run_dir).expect("create run dir");
    fs::write(
        run_dir.join("manifest.json"),
        r#"{"run_id":"001","status":"success","created_unix_ms":1,"started_unix_ms":1,"finished_unix_ms":1,"graph_fingerprint":"x","node_counts":{"total":0,"success":0,"failed":0,"skipped":0},"nodes":[],"policy":{"deny_network":false,"deny_env":false,"deny_clock":false,"clean_env":false},"materialize_inputs":"copy","cache_mode":"off"}"#,
    )
    .expect("write manifest");

    let replay_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "replay",
            run_dir.to_string_lossy().as_ref(),
            "--out",
            dir.path().join("replay-out").to_string_lossy().as_ref(),
        ])
        .expect("parse replay args");
    let replay_result = dag_run(&replay_matches);
    assert_eq!(replay_result, Err(ExitCode::from(3)));
}

#[test]
fn graph_read_failure_happens_before_command_execution_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("missing.json");

    let run_matches = dag_command()
        .try_get_matches_from([
            "bijux-dag",
            "run",
            missing.to_string_lossy().as_ref(),
            "--out",
            dir.path().join("out").to_string_lossy().as_ref(),
        ])
        .expect("parse run args");
    let run_result = dag_run(&run_matches);
    assert_eq!(run_result, Err(ExitCode::from(3)));
}

#[test]
fn doctor_command_handles_missing_engine_binaries_with_fallback() {
    let matches = dag_command()
        .try_get_matches_from(["bijux-dag", "--json", "doctor"])
        .expect("parse doctor args");
    let result = dag_run(&matches);
    assert_eq!(result, Ok(ExitCode::SUCCESS));
}
