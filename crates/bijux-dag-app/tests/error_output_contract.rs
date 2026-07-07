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
#[ignore = "experimental"]
fn json_error_output_contains_structured_fields() {
    let output = support::run_dag_command(
        &["lint", "--strict", "--json", &examples_file("hello.dag.json")],
        &repo_root(),
    );

    assert_ne!(output.0, 0);
    let payload: serde_json::Value = serde_json::from_str(&output.1).expect("parse json response");
    assert_eq!(payload["ok"], false);
    assert!(payload["error"].is_object());
    assert!(payload["error"]["category"].is_string());
    assert!(payload["error"]["code"].is_string());
    assert!(payload["error"]["exit_code"].is_number());
}
