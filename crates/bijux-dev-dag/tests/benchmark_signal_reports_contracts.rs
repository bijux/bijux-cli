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

use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn benchmark_signal_reports_exist_for_product_relevant_latency_dimensions() {
    let root = repo_root();
    for rel in [
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
        assert!(
            root.join(rel).exists(),
            "missing benchmark signal report: {rel}"
        );
    }
}
