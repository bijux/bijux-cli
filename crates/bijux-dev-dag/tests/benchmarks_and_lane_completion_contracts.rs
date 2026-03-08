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

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn benchmark_focus_and_lane_reports_exist() {
    for rel in [
        "docs/reports/foundation/benchmark_suite_git_for_computation_graphs.md",
        "docs/reports/foundation/benchmark_suite_app_operator_workflows.md",
        "docs/reports/foundation/benchmark_suite_bundle_portability_workflows.md",
        "docs/reports/foundation/benchmark_suite_runtime_event_state_machine.md",
        "docs/reports/foundation/top_10_slowest_commands.md",
        "docs/reports/foundation/top_10_slowest_tests.md",
        "docs/reports/foundation/fast_lane_unique_inventory.json",
        "docs/reports/foundation/full_lane_unique_inventory.json",
        "docs/reports/foundation/fast_lane_skipped_inventory.json",
        "docs/reports/foundation/fast_lane_unique_tests_report.md",
        "docs/reports/foundation/full_lane_only_tests_report.md",
        "docs/reports/foundation/slow_contract_promotion_review.md",
        "docs/reports/foundation/next_phase_candidate_report.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing benchmark/lane artifact: {rel}"
        );
    }
}

#[test]
fn ci_budget_targets_include_all_required_entrypoints() {
    let raw = fs::read_to_string(
        repo_root().join("docs/reports/foundation/ci_runtime_budget_targets.json"),
    )
    .expect("read ci runtime budget targets");
    let value: serde_json::Value =
        serde_json::from_str(&raw).expect("parse ci runtime budget targets");
    let budgets = value["budgets"].as_object().expect("budgets object");
    for key in [
        "make_test_max_minutes",
        "make_test_all_max_minutes",
        "make_coverage_max_minutes",
        "make_evidence_all_max_minutes",
    ] {
        assert!(budgets.contains_key(key), "missing budget key: {key}");
        assert!(
            budgets[key].as_u64().unwrap_or_default() > 0,
            "budget value must be > 0 for {key}"
        );
    }
}
