use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

#[test]
fn run_dir_contract_documents_sections_required_by_governance_guard() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/RUN_DIR_CONTRACT.md")).expect("contract");

    for token in [
        "Required entries (authoritative)",
        "Optional entries",
        "Derived artifacts (non-authoritative)",
        "Verification behavior",
        "dag verify --strict",
    ] {
        assert!(contract.contains(token), "run-dir contract missing token: {token}");
    }
}

#[test]
fn run_dir_contract_uses_live_retained_paths_and_optional_plan_language() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/RUN_DIR_CONTRACT.md")).expect("contract");
    let storage =
        fs::read_to_string(root.join("docs/spec/RUN_DIR_STORAGE_CONTRACT.md")).expect("storage");

    for token in [
        "outputs/index.json",
        "nodes/<node_id>/trace.json",
        "nodes/<node_id>/inputs/index.json",
        "nodes/<node_id>/outputs/index.json",
        "promotions/index.json",
        "plan.json",
        "standard local run snapshots do not currently retain `plan.json` by default",
    ] {
        assert!(contract.contains(token), "run-dir contract missing token: {token}");
    }

    for token in [
        "outputs/index.json",
        "nodes/<node_id>/trace.json",
        "nodes/<node_id>/attempts.json",
        "nodes/<node_id>/resolved_params.json",
        "plan.json",
    ] {
        assert!(storage.contains(token), "run-dir storage contract missing token: {token}");
    }

    assert!(
        !contract.contains("outputs.index.json"),
        "run-dir contract must use the live outputs/index.json path"
    );
    assert!(
        !contract.contains("`trace/`"),
        "run-dir contract must describe retained node trace files, not a fake trace directory"
    );
}

#[test]
fn import_export_contract_documents_bundle_versioning_modes_and_provenance() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/IMPORT_EXPORT_CONTRACT.md")).expect("contract");

    for token in [
        "Bundle versioning",
        "export-bundle/v0.1",
        "dag export --manifest-only",
        "dag export --with-files",
        "provenance.source",
    ] {
        assert!(contract.contains(token), "import/export contract missing token: {token}");
    }
}

#[test]
fn run_dir_import_export_report_links_runtime_docs_tests_and_trust_properties() {
    let root = workspace_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/RUN_DIR_IMPORT_EXPORT_HARDENING_REPORT.md"),
    )
    .expect("report");

    for token in [
        "docs/spec/RUN_DIR_STORAGE_CONTRACT.md",
        "docs/spec/RUN_DIR_CONTRACT.md",
        "docs/spec/RUN_DIR_OWNERSHIP.md",
        "docs/spec/IMPORT_EXPORT_CONTRACT.md",
        "docs/spec/ARTIFACT_OWNERSHIP_TABLE.md",
        "docs/spec/ARTIFACT_LIFECYCLE.md",
        "crates/bijux-dag-artifacts/src/storage/hardening.rs",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs",
        "crates/bijux-dev/tests/run_dir_import_export_hardening_contracts.rs",
        "tp_run_dir_resilience",
        "tp_import_export_compatibility",
    ] {
        assert!(report.contains(token), "hardening report missing token: {token}");
    }
}
