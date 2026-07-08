use base64 as _;
use bijux_dag_app as _;
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

use bijux_dag_app::{dag_command, dag_run};
use std::process::ExitCode;

#[test]
fn validation_errors_map_to_contract_exit_code() {
    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from(["bijux-dag", "validate", "/definitely/missing/file.json"])
        .expect("clap parse");

    let code = dag_run(&matches).expect_err("expected failure");
    assert_eq!(code, ExitCode::from(3));
}

#[test]
fn parse_errors_return_non_success_without_panic() {
    let dir = tempfile::tempdir().expect("tmp");
    let invalid = dir.path().join("bad.json");
    std::fs::write(&invalid, "{ invalid").expect("write");

    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from(["bijux-dag", "validate", invalid.to_string_lossy().as_ref()])
        .expect("clap parse");

    let code = dag_run(&matches).expect_err("expected parse failure");
    assert_eq!(code, ExitCode::from(2));
}
