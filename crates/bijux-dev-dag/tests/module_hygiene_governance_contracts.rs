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
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn policy() -> serde_json::Value {
    serde_json::from_str(
        &fs::read_to_string(root().join("configs/policy/module_hygiene_governance.json"))
            .expect("read module hygiene governance policy"),
    )
    .expect("parse module hygiene governance policy")
}

#[test]
fn module_hygiene_policy_declares_required_rules() {
    let policy = policy();
    assert_eq!(
        policy["governance_rules"]["new_top_level_modules_require_ownership_classification"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["new_large_modules_require_split_rationale"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["new_tiny_wrapper_modules_require_justification"],
        true
    );
    assert_eq!(
        policy["governance_rules"]["module_hygiene_drift_is_release_gate"],
        true
    );
}

#[test]
fn module_hygiene_reports_exist() {
    for rel in [
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
        "docs/reports/foundation/duplicate_path_policy_report_helpers_report.md",
        "docs/reports/foundation/repo_tree_hotspots_report.md",
        "docs/reports/foundation/repo_tree_cleanup_candidates_report.md",
        "docs/reports/foundation/repo_tree_health_dashboard.md",
        "docs/reports/foundation/module_hygiene_drift_gate_report.md",
        "docs/adr/20260308-repo-tree-shape-target-v0-1-0.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing module hygiene governance artifact: {rel}"
        );
    }
}

#[test]
fn module_hygiene_release_gate_target_exists_in_makefile() {
    let make_root = fs::read_to_string(root().join("make/root.mk")).expect("read make/root.mk");
    assert!(
        make_root.contains("module-hygiene-drift:"),
        "missing module-hygiene-drift gate target"
    );
    assert!(
        make_root.contains("module_hygiene_governance_contracts"),
        "module-hygiene-drift gate should execute module hygiene governance contract"
    );
}
