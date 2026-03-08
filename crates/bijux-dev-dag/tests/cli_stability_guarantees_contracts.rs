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
fn cli_341_360_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/cli_stability_341_360_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "341-353 CLI surface, compatibility, help/output, errors, ordering, latency, smoke, and no-panic coverage",
        "354-355 command inventory and usage heatmap reports",
        "356 CLI compatibility verification suite",
        "357 CLI stability dashboard",
        "358 regression fixture pack",
        "359 CLI error taxonomy report",
        "360 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn cli_341_360_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/cli_stability_341_360_status_report.md",
        "docs/reports/foundation/cli_command_inventory_report.md",
        "docs/reports/foundation/cli_command_usage_heatmap.md",
        "docs/reports/foundation/cli_error_taxonomy_report.md",
        "docs/reports/foundation/cli_stability_dashboard.md",
        "docs/reports/foundation/cli_surface_compatibility_report.md",
        "docs/reports/foundation/cli_json_compatibility_report.md",
        "docs/reports/foundation/cli_command_coverage_report.md",
        "docs/adr/20260308-cli-stability-guarantees.md",
        "configs/suites/cli_stability_verification.json",
        "crates/bijux-dag-cli/tests/contract_surface.rs",
        "crates/bijux-dag-cli/tests/taxonomy_and_policy_contracts.rs",
        "crates/bijux-dag-cli/tests/smoke_pipeline.rs",
        "crates/bijux-dag-cli/tests/cli_surface_completion_contracts.rs",
        "crates/bijux-dag-app/tests/error_exit_contract.rs",
        "crates/bijux-dag-app/tests/error_output_contract.rs",
        "crates/bijux-dag-app/tests/error_snapshot_contract.rs",
        "crates/bijux-dag-app/tests/snapshots/dag_command_tree.txt",
        "crates/bijux-dag-app/tests/snapshots/error_json_shape.json",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
