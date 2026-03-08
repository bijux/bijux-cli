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

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn benchmark_reports_cover_required_latency_and_cost_surfaces() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/fsck_deep_verification_cost_report.md",
        "docs/reports/foundation/app_hot_path_latency_report.md",
        "docs/reports/foundation/replay_proof_latency_report.md",
        "docs/reports/foundation/diff_explain_latency_report.md",
        "docs/reports/foundation/bundle_export_import_latency_report.md",
        "docs/reports/foundation/artifact_inspect_verify_latency_report.md",
        "docs/reports/foundation/run_history_query_latency_report.md",
        "docs/reports/foundation/scheduler_overhead_small_dag_report.md",
        "docs/reports/foundation/scheduler_overhead_medium_dag_report.md",
        "docs/reports/foundation/scheduler_overhead_large_dag_report.md",
        "docs/reports/foundation/semantic_diff_equivalence_cost_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing benchmark report: {rel}");
    }
}

#[test]
fn benchmark_regression_policy_and_trend_summary_exist() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/benchmark_regression_thresholds.json"))
            .expect("read benchmark thresholds policy"),
    )
    .expect("parse benchmark thresholds policy");

    for key in [
        "app_hot_paths",
        "replay_diff_proof_hot_paths",
        "bundle_import_export",
        "history_queries",
    ] {
        let ratio = policy["max_regression_ratio"][key]
            .as_f64()
            .expect("regression ratio must be f64");
        assert!(
            ratio >= 0.0 && ratio <= 1.0,
            "invalid regression ratio for {key}"
        );
    }
    assert_eq!(
        policy["requires_raw_data_for_performance_claims"]
            .as_bool()
            .expect("requires raw data bool"),
        true
    );

    assert!(
        root.join("docs/reports/foundation/benchmark_baseline_trend_summary.json")
            .exists(),
        "missing benchmark trend summary"
    );
}

#[test]
fn benchmark_top_ten_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/top_10_slowest_commands.md",
        "docs/reports/foundation/top_10_slowest_evidence_commands.md",
        "docs/reports/foundation/top_10_slowest_tests.md",
    ] {
        assert!(root.join(rel).exists(), "missing top ten report: {rel}");
    }
}

#[test]
fn benchmark_docs_cover_scorecards_types_hygiene_and_review() {
    let root = repo_root();
    for rel in [
        "docs/spec/BENCHMARK_SCORECARD_GUIDE.md",
        "docs/spec/BENCHMARK_TYPES.md",
        "docs/reports/foundation/benchmark_hygiene_report.md",
        "docs/reports/foundation/benchmark_fixture_simplification_report.md",
        "docs/reference/BENCHMARK_REVIEW_CHECKLIST.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing benchmark guidance doc: {rel}"
        );
    }
}

#[test]
fn benchmark_suite_focus_reports_and_coverage_map_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/benchmark_suite_git_for_computation_graphs.md",
        "docs/reports/foundation/benchmark_suite_app_operator_workflows.md",
        "docs/reports/foundation/benchmark_suite_bundle_portability_workflows.md",
        "docs/reports/foundation/benchmark_suite_runtime_event_state_machine.md",
        "docs/reports/foundation/benchmark_coverage_map.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing benchmark suite report: {rel}"
        );
    }
}

#[test]
fn release_checklist_includes_performance_claim_data_gate() {
    let root = repo_root();
    let checklist = fs::read_to_string(root.join("docs/spec/RELEASE_REVIEW_CHECKLIST.md"))
        .expect("read release review checklist");
    assert!(
        checklist.contains("Performance claims are backed by raw benchmark data"),
        "release checklist missing performance claim data gate"
    );
}
