use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn fast_full_lane_docs_and_reports_exist() {
    for rel in [
        "docs/testing/FAST_FULL_TEST_RULES.md",
        "docs/testing/CONTRIBUTOR_TEST_WORKFLOW.md",
        "docs/reports/foundation/fast_lane_skipped_inventory.json",
        "docs/reports/foundation/full_lane_unique_inventory.json",
        "docs/reports/foundation/fast_lane_slowest_ten.md",
        "docs/reports/foundation/full_lane_slowest_ten.md",
        "docs/reports/foundation/ci_runtime_budget_targets.json",
        "docs/reports/foundation/smoke_coverage_matrix.md",
        "docs/reports/foundation/smoke_contract_regression_board.md",
        "docs/reports/foundation/next_iteration_candidate_ranking.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing governance artifact: {rel}"
        );
    }
}

#[test]
fn lane_inventories_are_generated_artifacts() {
    for rel in [
        "docs/reports/foundation/fast_lane_skipped_inventory.json",
        "docs/reports/foundation/full_lane_unique_inventory.json",
    ] {
        let raw = std::fs::read_to_string(repo_root().join(rel)).expect("read inventory");
        let value: serde_json::Value = serde_json::from_str(&raw).expect("inventory json");
        assert!(
            value.get("generated_from").is_some(),
            "missing generated_from in {rel}"
        );
    }
}

#[test]
fn smoke_coverage_matrix_includes_release_domains() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/smoke_coverage_matrix.md"),
    )
    .expect("read smoke matrix");
    for token in ["identity", "replay", "diff", "bundle", "proof"] {
        assert!(raw.contains(token), "missing smoke domain token {token}");
    }
}
