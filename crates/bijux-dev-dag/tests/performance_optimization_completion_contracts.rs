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
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("read file")
}

#[test]
fn performance_optimization_contract_and_reports_exist() {
    for rel in [
        "docs/spec/PERFORMANCE_OPTIMIZATION_CONTRACT.md",
        "docs/reports/foundation/performance_optimization_telemetry_report.md",
        "docs/reports/foundation/performance_optimization_trend_report.md",
        "docs/reports/foundation/performance_optimization_checklist.md",
        "docs/reports/foundation/performance_regression_summary_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty optimization artifact: {rel}");
    }
}

#[test]
fn performance_optimization_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/performance_optimization/regression_corpus.json",
    ))
    .expect("parse performance optimization corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 16, "expected broad optimization corpus");

    for coverage in [
        "graph-parsing",
        "dag-validation",
        "planner-execution",
        "scheduler-decision",
        "runtime-node-overhead",
        "artifact-hashing",
        "artifact-io",
        "replay-planning",
        "diff-computation",
        "explain-command",
        "run-history-query",
        "provenance-traversal",
        "artifact-store-rw",
        "memory-allocation",
        "cpu-utilization",
        "regression-detection",
        "trend-reporting",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing optimization coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/performance_optimization_regression.json"))
            .expect("parse performance optimization suite");
    assert_eq!(suite["id"], "performance-optimization-regression");
}

#[test]
fn optimization_surfaces_anchor_existing_benchmark_contracts() {
    let benchmark_completion = read("crates/bijux-dev-dag/tests/benchmark_completion_contracts.rs");
    assert!(
        benchmark_completion.contains("run_history_query_latency_report.md"),
        "missing run history benchmark anchor"
    );

    let perf_evidence = read("crates/bijux-dev-dag/tests/perf_evidence_contracts.rs");
    assert!(
        perf_evidence.contains("evidence/perf/metadata.json"),
        "missing performance evidence metadata anchor"
    );

    let signal_quality = read("crates/bijux-dev-dag/tests/benchmark_signal_quality_contracts.rs");
    assert!(
        signal_quality.contains("benchmark_scenarios_by_claim_report.md"),
        "missing benchmark signal quality anchor"
    );
}
