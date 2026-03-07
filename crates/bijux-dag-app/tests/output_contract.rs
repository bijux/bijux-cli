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

use std::path::PathBuf;
use std::process::Command;

fn examples_file(file_name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples")
        .join(file_name);
    root.to_string_lossy().into_owned()
}

#[test]
#[ignore = "slow"]
fn app_text_validate_output_contract() {
    let output = Command::new("cargo")
        .env("CARGO_TARGET_DIR", "artifacts/target")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "validate",
            &examples_file("hello.dag.json"),
        ])
        .output()
        .expect("run validate");

    assert!(output.status.success());
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("status:"));
}

#[test]
#[ignore = "slow"]
fn app_json_validate_output_contract() {
    let output = Command::new("cargo")
        .env("CARGO_TARGET_DIR", "artifacts/target")
        .args([
            "run",
            "-p",
            "bijux-dag-cli",
            "--",
            "dag",
            "validate",
            "--json",
            &examples_file("hello.dag.json"),
        ])
        .output()
        .expect("run validate json");

    assert!(output.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("parse json response");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"].is_object());
}
