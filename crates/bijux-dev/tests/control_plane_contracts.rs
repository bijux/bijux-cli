use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn control_plane_schemas_are_valid_json_schema_objects() {
    let root = repo_root();
    for rel in [
        "configs/dag/schema/dev-control/command_report.schema.json",
        "configs/dag/schema/dev-control/suite_selection_report.schema.json",
    ] {
        let payload = fs::read_to_string(root.join(rel)).expect("read schema file");
        let value: Value = serde_json::from_str(&payload).expect("parse schema json");
        assert_eq!(value["$schema"].as_str(), Some("https://json-schema.org/draft/2020-12/schema"));
        assert!(value["title"].as_str().is_some());
    }
}

#[test]
fn suite_run_contract_exposes_advisory_and_why_flags() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .expect("read command source");
    assert!(
        source.contains("advisory: bool"),
        "suite run command should support advisory execution mode"
    );
    assert!(source.contains("why: bool"), "suite run command should support why explanation mode");
    assert!(
        source.contains("CommandLine::Foundation"),
        "foundation super-suite command should be present"
    );
    assert!(
        source.contains("CommandLine::FoundationHardening"),
        "foundation hardening command should be present"
    );
}
