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
fn run_history_221_240_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/run_history_221_240_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "221-230 ordering, reconstruction, corruption, and operational behavior",
        "231-232 regression fixtures for corruption and orphan recovery",
        "233-234 explain output snapshots and schema checks",
        "235-236 size-growth and corruption resilience reports",
        "237 run history invariants verification suite",
        "238 run history diagnostics report",
        "239 run history consistency dashboard",
        "240 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn run_history_221_240_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/run_history_221_240_status_report.md",
        "docs/reports/foundation/run_history_size_growth_report.md",
        "docs/reports/foundation/run_history_corruption_resilience_report.md",
        "docs/reports/foundation/run_history_diagnostics_report.md",
        "docs/reports/foundation/run_history_consistency_dashboard.md",
        "docs/reports/foundation/run_history_api_report.json",
        "docs/adr/20260308-run-history-guarantees.md",
        "configs/suites/run_history_invariants.json",
        "evidence/cache/replay/run_manifest_regression_corpus.json",
        "crates/bijux-dag-app/tests/fixtures/run_history_mixed_runs.json",
        "crates/bijux-dag-app/tests/run_history_contract.rs",
        "crates/bijux-dag-app/tests/run_history_hardening_contract.rs",
        "crates/bijux-dag-app/tests/run_history_reliability_contract.rs",
        "crates/bijux-dag-app/tests/run_history_ancestry_contracts.rs",
        "crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/run_history_completion_contracts.rs",
        "crates/bijux-dev-dag/tests/run_history_resilience_suite_contracts.rs",
        "crates/bijux-dev-dag/tests/run_history_api_report_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
