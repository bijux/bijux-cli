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
fn explain_301_320_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/explain_301_320_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "301-314 explain determinism, corrupted/partial/imported/replay behavior, and reasoning coverage",
        "315-316 coverage and determinism reports",
        "317 explain verification suite",
        "318 explain diagnostics dashboard",
        "319 explain operator smoke tests",
        "320 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn explain_301_320_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/explain_301_320_status_report.md",
        "docs/reports/foundation/explain_coverage_report.md",
        "docs/reports/foundation/explain_determinism_report.md",
        "docs/reports/foundation/explain_diagnostics_dashboard.md",
        "docs/reports/foundation/explainability_completeness_report.md",
        "docs/reports/foundation/explainability_anomaly_detection_report.md",
        "docs/reports/foundation/advanced_explainability_coverage_report.md",
        "docs/adr/20260308-explain-semantics-guarantees.md",
        "configs/suites/explain_verification.json",
        "configs/suites/explain_surface_stress.json",
        "configs/suites/advanced_explainability_regression.json",
        "configs/schema/operator/run_explain_failure.schema.json",
        "configs/schema/operator/run_id_explain.schema.json",
        "crates/bijux-dag-app/tests/diff_explain_contract.rs",
        "crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs",
        "crates/bijux-dag-app/tests/artifact_identity_explain_contract.rs",
        "crates/bijux-dev-dag/tests/explain_surface_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/advanced_explainability_completion_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
