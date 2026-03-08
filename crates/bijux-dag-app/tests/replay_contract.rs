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

use std::path::Path;

#[test]
fn replay_fixture_family_exists() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let root = repo.join("evidence/cache/replay");
    for rel in [
        "match_case.json",
        "mismatch_case.json",
        "corruption_case.json",
        "unsupported_version_case.json",
    ] {
        assert!(root.join(rel).exists(), "missing replay fixture: {}", rel);
    }
}

#[test]
fn replay_battle_scenario_declares_mandatory_proof() {
    let value = bijux_dag_testkit::load_replay_fixture_json(
        env!("CARGO_MANIFEST_DIR"),
        "evidence/battle/workflows/replay/replay_semantic_comparison.json",
    );
    let assertions = value["assertions"].as_array().expect("assertions array");
    assert!(assertions.iter().any(|v| v == "replay_mandatory_proof"));
}
