use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn repo_tree_541_560_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/repo_tree_inventory_541_560_report.md",
        "docs/reports/foundation/repo_tree_tiny_module_inline_candidates_report.md",
        "docs/reports/foundation/repo_tree_giant_module_split_candidates_report.md",
        "docs/reports/foundation/repo_tree_shrink_trend_report.md",
        "docs/reports/foundation/repo_tree_simplification_541_560_status_report.md",
        "configs/suites/repo_tree_simplification_verification.json",
        "docs/adr/20260308-repo-tree-shape-governance.md",
    ] {
        assert!(root.join(rel).exists(), "missing repo-tree artifact: {rel}");
    }
}

#[test]
fn module_hygiene_policy_requires_split_plan_for_new_large_files() {
    let root = repo_root();
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/module_hygiene_governance.json"))
            .expect("read module hygiene policy"),
    )
    .expect("parse module hygiene policy");

    assert_eq!(
        policy["governance_rules"]["new_large_files_require_split_plan"].as_bool(),
        Some(true)
    );
}

#[test]
fn repo_tree_status_report_maps_541_560_requirements() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/repo_tree_simplification_541_560_status_report.md"),
    )
    .expect("read repo-tree status report");

    for token in [
        "541-548",
        "549-554",
        "555-560",
        "repo_tree_simplification_verification.json",
        "20260308-repo-tree-shape-governance.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn repo_tree_simplification_suite_contains_expected_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/repo_tree_simplification_verification.json"))
            .expect("read repo-tree suite"),
    )
    .expect("parse repo-tree suite");

    assert_eq!(suite["id"], "repo-tree-simplification-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "repo_tree_simplification_541_560_contracts",
        "repo_tree_governance_181_200_contracts",
        "module_hygiene_governance_contracts",
        "repo_health_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}
