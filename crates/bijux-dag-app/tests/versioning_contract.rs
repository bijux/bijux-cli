use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
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
use std::fs;

#[test]
fn version_inspect_reports_supported_graph_versions() {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("ok.json");
    fs::write(
        &dag,
        r#"{"spec":"0.1","meta":{"name":"ok","owners":[],"tags":[]},"nodes":[],"edges":[]}"#,
    )
    .expect("write dag");

    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--dag",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse args");
    let code = dag_run(&matches).expect("run inspect");
    assert_eq!(code, std::process::ExitCode::SUCCESS);
}

#[test]
fn version_inspect_rejects_unsupported_graph_versions() {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("bad.json");
    fs::write(
        &dag,
        r#"{"spec":"9.9","meta":{"name":"bad","owners":[],"tags":[]},"nodes":[],"edges":[]}"#,
    )
    .expect("write dag");

    let cmd = dag_command();
    let matches = cmd
        .try_get_matches_from([
            "bijux-dag",
            "--json",
            "version-inspect",
            "--dag",
            dag.to_string_lossy().as_ref(),
        ])
        .expect("parse args");
    let result = dag_run(&matches);
    assert!(result.is_err());
}

#[test]
fn migrate_noop_supported_and_cross_version_rejected() {
    let dir = tempfile::tempdir().expect("tmp");
    let dag = dir.path().join("migrate.json");
    fs::write(
        &dag,
        r#"{"spec":"0.1","meta":{"name":"migrate","owners":[],"tags":[]},"nodes":[],"edges":[]}"#,
    )
    .expect("write dag");

    let cmd = dag_command();
    let noop = cmd
        .clone()
        .try_get_matches_from([
            "bijux-dag",
            "migrate",
            "dag",
            dag.to_string_lossy().as_ref(),
            "--from",
            "0.1",
            "--to",
            "0.1",
        ])
        .expect("parse noop");
    assert!(dag_run(&noop).is_ok());

    let reject = cmd
        .try_get_matches_from([
            "bijux-dag",
            "migrate",
            "dag",
            dag.to_string_lossy().as_ref(),
            "--from",
            "0.1",
            "--to",
            "0.2",
        ])
        .expect("parse reject");
    assert!(dag_run(&reject).is_err());
}
