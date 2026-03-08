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
fn runtime_scope_101_120_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/runtime_scope_contraction_101_120_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "101-106 inventory and classification",
        "107-109 no-tests / no-fixtures / no-user-path reports",
        "111-114 invariants for identity/replay/help/capabilities",
        "117 speculative-surface budget",
        "119 stable-vs-experimental surface page",
        "120 end-state ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn runtime_scope_101_120_governance_artifacts_exist() {
    for rel in [
        "configs/policy/runtime_scope_v2.json",
        "configs/policy/advanced_semantics_governance.json",
        "docs/reports/foundation/runtime_internal_surface_inventory_report.md",
        "docs/reports/foundation/advanced_semantics_no_direct_tests_report.md",
        "docs/reports/foundation/advanced_semantics_no_examples_report.md",
        "docs/reports/foundation/advanced_semantics_no_user_path_report.md",
        "docs/reports/foundation/speculative_surface_budget.md",
        "docs/reports/foundation/runtime_stable_vs_experimental_surface_page.md",
        "docs/adr/20260308-advanced-semantics-runtime-boundary.md",
        "docs/adr/ADR-advanced-semantics-end-state.md",
        "crates/bijux-dev-dag/tests/advanced_semantics_governance_contracts.rs",
        "crates/bijux-dev-dag/tests/advanced_semantics_progress_contracts.rs",
        "crates/bijux-dev-dag/tests/runtime_scope_v2_guardrails.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
