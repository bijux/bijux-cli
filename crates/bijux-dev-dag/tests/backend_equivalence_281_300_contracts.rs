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
fn backend_281_300_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/backend_equivalence_281_300_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "281-294 backend equivalence tests and compatibility coverage",
        "295 backend equivalence report",
        "296 backend capability matrix report",
        "297 backend equivalence verification suite",
        "298 backend divergence diagnostics report",
        "299 backend equivalence dashboard",
        "300 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn backend_281_300_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/backend_equivalence_281_300_status_report.md",
        "docs/reports/foundation/backend_equivalence_report.md",
        "docs/reports/foundation/backend_capability_matrix.md",
        "docs/reports/foundation/backend_divergence_diagnostics_report.md",
        "docs/reports/foundation/backend_equivalence_dashboard.md",
        "docs/reports/foundation/backend_equivalence_quality_benchmark.md",
        "docs/reports/foundation/backend_equivalence_performance_benchmarks.md",
        "docs/adr/20260308-backend-semantic-equivalence-guarantees.md",
        "configs/suites/backend_equivalence_verification.json",
        "evidence/reports/backend_capability_matrix_generated.json",
        "evidence/compat/backend_equivalence/generated_fixture_corpus.json",
        "crates/bijux-dag-cli/tests/contract_surface.rs",
        "crates/bijux-dev-dag/tests/backend_equivalence_contracts.rs",
        "crates/bijux-dev-dag/tests/backend_equivalence_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
