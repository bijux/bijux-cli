use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    generate_adapter_reference_markdown, registered_adapter_reference_document,
};
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn generated_adapter_reference_matches_checked_in_spec() {
    let document = registered_adapter_reference_document();
    let rendered = format!("{}\n", generate_adapter_reference_markdown(&document));
    let checked_in =
        fs::read_to_string(repo_root().join("docs/spec/ADAPTER_CONTRACT.md")).expect("read spec");
    assert_eq!(rendered, checked_in);
}

#[test]
fn generated_adapter_reference_renders_execution_observations_without_option_debug_output() {
    let document = registered_adapter_reference_document();
    let rendered = generate_adapter_reference_markdown(&document);
    assert!(rendered.contains("checked_by_execution=true"));
    assert!(rendered.contains("observed status=success"));
    assert!(!rendered.contains("Some(\""));
}
