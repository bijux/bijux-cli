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
fn repo_tree_181_200_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/repo_tree_governance_181_200_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "181-184 module size and direct-test inventory reports",
        "185-187 coverage, fixture, and docs-link reports",
        "188-190 governance rules for ownership and module sizing",
        "191-193 naming and unused-surface reports",
        "194-195 duplicate helper inventories",
        "196-197 repo-tree hotspot and cleanup-candidate pages",
        "198 module-hygiene drift gate",
        "199 maintainer repo-tree health dashboard",
        "200 ADR for target repo-tree shape",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn repo_tree_181_200_governance_artifacts_exist() {
    for rel in [
        "configs/policy/module_hygiene_governance.json",
        "docs/reports/foundation/module_inventory_under_10_lines_review.md",
        "docs/reports/foundation/module_inventory_over_500_lines.md",
        "docs/reports/foundation/module_inventory_over_1000_lines.md",
        "docs/reports/foundation/module_inventory_zero_direct_tests.md",
        "docs/reports/foundation/module_low_coverage_high_churn_report.md",
        "docs/reports/foundation/module_no_linked_fixtures_report.md",
        "docs/reports/foundation/module_no_linked_docs_report.md",
        "docs/reports/foundation/module_name_oversell_report.md",
        "docs/reports/foundation/module_rename_alignment_report.md",
        "docs/reports/foundation/dead_reexports_unused_preludes_report.md",
        "docs/reports/foundation/duplicate_helper_modules_report.md",
        "docs/reports/foundation/top_25_duplicate_helper_areas_report.md",
        "docs/reports/foundation/repo_tree_hotspots_report.md",
        "docs/reports/foundation/repo_tree_cleanup_candidates_report.md",
        "docs/reports/foundation/module_hygiene_drift_gate_report.md",
        "docs/reports/foundation/repo_tree_health_dashboard.md",
        "docs/adr/20260308-repo-tree-shape-target-v0-1-0.md",
        "crates/bijux-dev-dag/tests/module_hygiene_governance_contracts.rs",
        "crates/bijux-dev-dag/tests/evidence_dashboard_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
