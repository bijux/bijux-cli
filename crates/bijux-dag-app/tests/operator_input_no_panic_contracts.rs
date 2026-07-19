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
use std::fs;
use std::process::ExitCode;
use tar as _;
use tempfile as _;
use thiserror as _;

#[test]
fn malformed_graph_load_via_read_graph_path_does_not_panic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let malformed = tmp.path().join("bad-graph.json");
    fs::write(&malformed, "{not-json").expect("write malformed graph");
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from(["bijux-dag", "validate", malformed.to_string_lossy().as_ref()])
        .expect("parse validate args");
    let result = dag_run(&matches);
    assert_eq!(result, Err(ExitCode::from(2)));
}

#[test]
fn malformed_filesystem_input_via_fs_input_path_does_not_panic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let missing = tmp.path().join("missing-graph.json");
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from(["bijux-dag", "validate", missing.to_string_lossy().as_ref()])
        .expect("parse validate args");
    let result = dag_run(&matches);
    assert_eq!(result, Err(ExitCode::from(3)));
}

#[test]
fn malformed_bundle_import_routed_through_app_does_not_panic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let bundle = tmp.path().join("malformed.bundle.json");
    fs::write(&bundle, "{not-json").expect("write malformed bundle");
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "import",
            bundle.to_string_lossy().as_ref(),
            "--verify-only",
        ])
        .expect("parse import args");
    let result = dag_run(&matches);
    assert_eq!(result, Err(ExitCode::from(3)));
}

#[test]
fn corrupted_run_dir_inspect_routed_through_app_does_not_panic() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let run = tmp.path().join("run-corrupt");
    fs::create_dir_all(&run).expect("create run dir");
    fs::write(run.join("manifest.json"), "{not-json").expect("write corrupt manifest");
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "runs",
            "inspect",
            "run-corrupt",
            "--root",
            tmp.path().to_string_lossy().as_ref(),
        ])
        .expect("parse inspect args");
    let result = dag_run(&matches);
    assert_eq!(result, Ok(ExitCode::SUCCESS));
}
