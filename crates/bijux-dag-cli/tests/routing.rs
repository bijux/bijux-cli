use bijux_dag_app as _;
use clap as _;
use clap_complete as _;
use serde_json as _;
use tempfile as _;

use std::process::Command;

#[test]
fn dag_validate_routes() {
    let bin = env!("CARGO_BIN_EXE_bijux-dag");
    let dag = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("evidence")
        .join("authoring")
        .join("examples")
        .join("hello.dag.json");
    let out = Command::new(bin).args(["validate", dag.to_str().unwrap()]).output().unwrap();
    assert!(out.status.success());
}

#[test]
fn unknown_subapp_fails() {
    let bin = env!("CARGO_BIN_EXE_bijux-dag");
    let out = Command::new(bin).args(["foo"]).output().unwrap();
    assert!(!out.status.success());
}
