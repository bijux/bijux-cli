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
fn benchmark_focus_reports_exist_for_identity_history_replay_diff_artifact_bundle() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/graph_identity_canonicalization_benchmarks.md",
        "docs/reports/foundation/run_history_query_benchmarks.md",
        "docs/reports/foundation/replay_proof_generation_benchmarks.md",
        "docs/reports/foundation/semantic_diff_explain_benchmarks.md",
        "docs/reports/foundation/artifact_inspect_hash_trace_benchmarks.md",
        "docs/reports/foundation/bundle_import_export_verify_fsck_benchmarks.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing benchmark focus report: {rel}"
        );
    }
}

#[test]
fn benchmark_threshold_assertion_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/benchmark_threshold_assertions_graph_identity.json",
        "docs/reports/foundation/benchmark_threshold_assertions_run_history.json",
        "docs/reports/foundation/benchmark_threshold_assertions_artifact_trace.json",
        "docs/reports/foundation/benchmark_threshold_assertions_replay_mismatch_grouping.json",
        "docs/reports/foundation/benchmark_threshold_assertions_semantic_diff_equivalence.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing threshold assertion report: {rel}"
        );
    }
}

#[test]
fn slowest_and_gap_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/top_10_slowest_app_routes_report.md",
        "docs/reports/foundation/top_10_slowest_runtime_helpers_report.md",
        "docs/reports/foundation/top_10_slowest_dev_dag_commands_report.md",
        "docs/reports/foundation/benchmarks_without_regression_thresholds_report.md",
        "docs/reports/foundation/claims_without_benchmark_coverage_report.md",
        "docs/reports/foundation/next_hot_spots_after_600_report.md",
        "docs/reports/foundation/master_delivery_board_1_600.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing benchmark/governance report: {rel}"
        );
    }
}

#[test]
fn release_gates_for_benchmark_smoke_matrix_and_generated_doc_sources_exist() {
    let root = repo_root();
    let matrix =
        root.join("docs/reports/foundation/git_for_computation_graphs_benchmark_smoke_matrix.md");
    let generated_guard =
        root.join("docs/reports/foundation/benchmark_docs_generated_sources_guard.md");
    assert!(matrix.exists(), "missing benchmark/smoke matrix report");
    assert!(
        generated_guard.exists(),
        "missing generated benchmark docs guard report"
    );

    let body = fs::read_to_string(&matrix).expect("read matrix");
    for required in [
        "graph identity",
        "replay proof",
        "semantic diff",
        "artifact inspect trace",
        "bundle import export verify fsck",
        "run history queries",
    ] {
        assert!(
            body.contains(required),
            "matrix missing primitive: {required}"
        );
    }
}

#[test]
fn benchmark_release_signal_fast_suite_declares_core_contracts() {
    let root = repo_root();
    let payload =
        fs::read_to_string(root.join("configs/suites/benchmark_release_signals_fast.json"))
            .expect("read suite config");
    let suite: serde_json::Value = serde_json::from_str(&payload).expect("parse suite config");
    assert_eq!(suite["suite"], "benchmark-release-signals-fast");
    assert_eq!(suite["lane"], "fast");
}
