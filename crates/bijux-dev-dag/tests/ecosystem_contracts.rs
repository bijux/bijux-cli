use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn ecosystem_contract_docs_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/BIJUX_SHARED_IDENTITY_CONTRACT.md",
        "docs/spec/BIJUX_CLI_INTEGRATION_CONTRACT.md",
        "docs/spec/ATLAS_EXECUTION_CONTRACT.md",
        "docs/spec/DNA_EXECUTION_CONTRACT.md",
        "docs/reference/BIJUX_COMMAND_OWNERSHIP.md",
        "docs/reference/ECOSYSTEM_VERSION_COMPATIBILITY_MATRIX.md",
        "docs/reports/foundation/ecosystem_truth_surface.md",
        "docs/tracking/ECOSYSTEM_BACKLOG_BOOTSTRAP.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing ecosystem contract doc: {rel}"
        );
    }
}

#[test]
fn shared_identity_and_reference_fixtures_exist() {
    let root = repo_root();
    for rel in [
        "evidence/compat/ecosystem/shared_identity/graph_identity_fixture.json",
        "evidence/compat/ecosystem/shared_identity/run_identity_fixture.json",
        "evidence/compat/ecosystem/shared_identity/artifact_identity_fixture.json",
        "evidence/compat/ecosystem/shared_refs/run_artifact_reference_fixture.json",
    ] {
        assert!(root.join(rel).exists(), "missing shared fixture: {rel}");
    }
}

#[test]
fn cross_repo_workflow_scenarios_exist() {
    let root = repo_root();
    for rel in [
        "evidence/compat/ecosystem/workflows/cross_repo_sample_workflow.json",
        "evidence/compat/ecosystem/workflows/cross_repo_replay_scenario.json",
        "evidence/compat/ecosystem/workflows/cross_repo_artifact_lineage_scenario.json",
        "evidence/compat/ecosystem/workflows/cross_repo_proof_consumption_scenario.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing cross-repo scenario: {rel}"
        );
    }
}

#[test]
fn command_ownership_doc_references_bijux_and_bijux_dag() {
    let root = repo_root();
    let body = fs::read_to_string(root.join("docs/reference/BIJUX_COMMAND_OWNERSHIP.md"))
        .expect("read ownership doc");
    assert!(body.contains("root `bijux`"));
    assert!(body.contains("`bijux dag`"));
}

#[test]
fn ecosystem_matrix_mentions_all_products() {
    let root = repo_root();
    let body =
        fs::read_to_string(root.join("docs/reference/ECOSYSTEM_VERSION_COMPATIBILITY_MATRIX.md"))
            .expect("read matrix");
    for token in ["bijux-cli", "bijux-dag", "bijux-atlas", "bijux-dna"] {
        assert!(
            body.contains(token),
            "missing matrix product entry: {token}"
        );
    }
}

#[test]
fn release_gate_workflow_exists_for_ecosystem_contracts() {
    let root = repo_root();
    let path = root.join(".github/workflows/ecosystem-contracts.yml");
    assert!(path.exists(), "missing ecosystem release gate workflow");
    let body = fs::read_to_string(path).expect("read workflow");
    assert!(body.contains("ecosystem contract tests"));
}

#[test]
fn cli_and_atlas_dna_boundaries_are_explicit() {
    let root = repo_root();
    let cli = fs::read_to_string(root.join("docs/spec/BIJUX_CLI_INTEGRATION_CONTRACT.md"))
        .expect("read cli integration");
    let atlas =
        fs::read_to_string(root.join("docs/spec/ATLAS_EXECUTION_CONTRACT.md")).expect("read atlas");
    let dna =
        fs::read_to_string(root.join("docs/spec/DNA_EXECUTION_CONTRACT.md")).expect("read dna");

    assert!(cli.contains("must not alter DAG identity/replay semantics"));
    assert!(atlas.contains("may not redefine"));
    assert!(dna.contains("may not redefine"));
}

#[test]
fn shared_identity_fixture_shape_is_machine_readable() {
    let root = repo_root();
    let payload: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("evidence/compat/ecosystem/shared_identity/graph_identity_fixture.json"),
        )
        .expect("read graph identity fixture"),
    )
    .expect("parse fixture");
    assert!(payload.get("graph_id").and_then(Value::as_str).is_some());
}
