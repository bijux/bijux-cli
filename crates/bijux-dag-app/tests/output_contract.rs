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

use std::path::PathBuf;

mod support;

fn repo_root() -> PathBuf {
    support::repo_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn examples_file(file_name: &str) -> String {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../evidence/authoring/examples")
        .join(file_name);
    root.to_string_lossy().into_owned()
}

#[test]
fn app_text_validate_output_contract() {
    let output =
        support::run_dag_command(&["validate", &examples_file("hello.dag.json")], &repo_root());

    assert_eq!(output.0, 0);
    let text = output.1;
    assert!(text.contains("status:"));
}

#[test]
fn app_json_validate_output_contract() {
    let output = support::run_dag_command(
        &["validate", "--json", &examples_file("hello.dag.json")],
        &repo_root(),
    );

    assert_eq!(output.0, 0);
    let payload: serde_json::Value = serde_json::from_str(&output.1).expect("parse json response");
    assert_eq!(payload["command"], "dag.validate");
    assert_eq!(payload["ok"], true);
    assert!(payload["data"].is_object());
}
