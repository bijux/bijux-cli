use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn count_numbered_items(rel: &str) -> usize {
    fs::read_to_string(root().join(rel))
        .expect("read report")
        .lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            let mut chars = trimmed.chars();
            let first = chars.next();
            let second = chars.next();
            let third = chars.next();
            first.is_some_and(|c| c.is_ascii_digit())
                && second.is_some_and(|c| c == '.')
                && third.is_some_and(|c| c == ' ')
                || (first.is_some_and(|c| c.is_ascii_digit())
                    && second.is_some_and(|c| c.is_ascii_digit())
                    && third.is_some_and(|c| c == '.'))
        })
        .count()
}

#[test]
fn top_25_ranking_reports_exist_and_have_25_items() {
    for rel in [
        "docs/reports/foundation/top_25_largest_files_remaining_report.md",
        "docs/reports/foundation/top_25_lowest_covered_product_paths_report.md",
        "docs/reports/foundation/top_25_highest_churn_files_remaining_report.md",
        "docs/reports/foundation/top_25_important_files_with_weak_direct_tests_report.md",
        "docs/reports/foundation/top_25_duplicate_helper_areas_report.md",
        "docs/reports/foundation/top_25_speculative_runtime_broad_surfaces_report.md",
        "docs/reports/foundation/top_25_benchmark_gaps_by_core_claim_report.md",
        "docs/reports/foundation/top_25_evidence_outputs_low_decision_value_report.md",
        "docs/reports/foundation/top_25_docs_pages_drift_risk_report.md",
        "docs/reports/foundation/top_25_promotable_fast_lane_tests_report.md",
    ] {
        assert!(root().join(rel).exists(), "missing ranking report: {rel}");
        let count = count_numbered_items(rel);
        assert_eq!(count, 25, "ranking report must contain 25 items: {rel}");
    }
}

#[test]
fn grouping_dependency_and_delivery_board_reports_exist() {
    for rel in [
        "docs/reports/foundation/backlog_1_800_grouped_by_crate_report.md",
        "docs/reports/foundation/backlog_1_800_grouped_by_work_type_report.md",
        "docs/reports/foundation/backlog_dependency_unlock_map_report.md",
        "docs/reports/foundation/delivery_board_1_800_report.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing backlog synthesis report: {rel}"
        );
    }
}

#[test]
fn shortlist_reports_exist_and_have_50_items() {
    for rel in [
        "docs/reports/foundation/backlog_high_impact_shortlist_50_report.md",
        "docs/reports/foundation/backlog_low_risk_high_signal_shortlist_50_report.md",
        "docs/reports/foundation/backlog_make_test_promotable_shortlist_50_report.md",
        "docs/reports/foundation/backlog_v0_1_publish_readiness_shortlist_50_report.md",
        "docs/reports/foundation/backlog_docs_site_publish_readiness_shortlist_50_report.md",
    ] {
        assert!(root().join(rel).exists(), "missing shortlist report: {rel}");
        let count = count_numbered_items(rel);
        assert_eq!(count, 50, "shortlist report must contain 50 items: {rel}");
    }
}

#[test]
fn backlog_rollover_adr_exists_and_links_core_inputs() {
    let rel = "docs/adr/20260308-backlog-rollover-governance-800-to-1000.md";
    let body = fs::read_to_string(root().join(rel)).expect("read rollover adr");
    for token in [
        "high_impact_shortlist_50",
        "low_risk_high_signal_shortlist_50",
        "make_test_promotable_shortlist_50",
        "v0_1_publish_readiness_shortlist_50",
        "docs_site_publish_readiness_shortlist_50",
        "backlog_dependency_unlock_map_report",
        "delivery_board_1_800_report",
    ] {
        assert!(
            body.contains(token),
            "rollover ADR missing reference token: {token}"
        );
    }
}
