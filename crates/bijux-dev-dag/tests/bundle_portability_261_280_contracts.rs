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

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn bundle_261_280_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/bundle_portability_261_280_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "261-270 export/import completeness, stability, idempotence, and corruption behavior",
        "271-274 fsck regression fixtures and replay compatibility",
        "275-276 portability and import diagnostics reports",
        "277 bundle verification suite",
        "278 bundle schema drift detection",
        "279 bundle compatibility dashboard",
        "280 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn bundle_261_280_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/bundle_portability_261_280_status_report.md",
        "docs/reports/foundation/bundle_portability_report.md",
        "docs/reports/foundation/bundle_import_diagnostics_report.md",
        "docs/reports/foundation/bundle_compatibility_dashboard.md",
        "docs/reports/foundation/bundle_fixture_inventory_report.md",
        "docs/reports/foundation/schema_changelog.md",
        "docs/adr/20260308-bundle-portability-guarantees.md",
        "configs/suites/bundle_portability_verification.json",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "crates/bijux-dag-cli/tests/contract_surface.rs",
        "crates/bijux-dev-dag/tests/bundle_portability_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/run_dir_import_export_hardening_contracts.rs",
        "crates/bijux-dev-dag/tests/schema_governance_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
