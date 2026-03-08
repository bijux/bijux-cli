use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn artifact_io_hardening_reports_and_adr_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/artifact_store_truth_report.md",
        "docs/reports/foundation/artifact_integrity_fixture_inventory_report.md",
        "docs/reports/foundation/artifact_io_hardening_completion_report.md",
        "docs/adr/20260308-artifact-crate-scope-runtime-app-boundaries.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing artifact io hardening output: {rel}"
        );
    }

    let completion = fs::read_to_string(
        root.join("docs/reports/foundation/artifact_io_hardening_completion_report.md"),
    )
    .expect("read completion");
    for required in [
        "481-500",
        "artifact_io_store_hardening_expansion_contracts.rs",
        "artifact_io_hardening_fast.json",
        "artifact_io_zero_coverage_gate_contracts.rs",
    ] {
        assert!(
            completion.contains(required),
            "completion report missing {required}"
        );
    }
}
