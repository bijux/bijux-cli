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

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn release_signal_completion_artifacts_cover_381_400() {
    let root = repo_root();

    let required = [
        "docs/reports/foundation/release_signal_benchmarks_completion_report.md",
        "docs/reports/foundation/benchmark_suite_git_for_computation_graphs.md",
        "docs/reports/foundation/benchmark_suite_app_operator_workflows.md",
        "docs/reports/foundation/benchmark_suite_bundle_portability_workflows.md",
        "docs/reports/foundation/benchmark_suite_runtime_event_state_machine.md",
        "docs/reports/foundation/replay_proof_latency_report.md",
        "docs/reports/foundation/diff_explain_latency_report.md",
        "docs/reports/foundation/artifact_inspect_verify_latency_report.md",
        "docs/reports/foundation/run_history_query_latency_report.md",
        "docs/reports/foundation/bundle_export_import_latency_report.md",
        "docs/reports/foundation/top_10_slowest_commands.md",
        "docs/reports/foundation/top_10_slowest_tests.md",
        "docs/reports/foundation/fast_lane_unique_tests_report.md",
        "docs/reports/foundation/full_lane_only_tests_report.md",
        "docs/reports/foundation/fast_lane_skipped_inventory.json",
        "docs/reports/foundation/slow_contract_promotion_review.md",
        "docs/reports/foundation/ci_runtime_budget_targets.json",
        "docs/reports/foundation/next_phase_candidate_report.md",
    ];

    for rel in required {
        assert!(
            root.join(rel).exists(),
            "missing release-signal completion artifact {rel}"
        );
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/release_signal_benchmarks_completion_report.md"),
    )
    .expect("read release-signal completion report");

    for required in [
        "(381-400)",
        "benchmark_suite_runtime_event_state_machine.md",
        "replay_proof_latency_report.md",
        "top_10_slowest_commands.md",
        "fast_lane_unique_tests_report.md",
        "ci_runtime_budget_targets.json",
        "next_phase_candidate_report.md",
    ] {
        assert!(
            report.contains(required),
            "release-signal completion report missing {required}"
        );
    }
}
