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
fn scalability_spec_and_reports_exist() {
    for rel in [
        "docs/spec/LARGE_DAG_SCALABILITY_CONTRACT.md",
        "docs/reports/foundation/large_dag_scalability_benchmarks.md",
        "docs/reports/foundation/dag_memory_footprint_regression_benchmarks.md",
        "docs/reports/foundation/large_dag_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty scalability surface: {rel}");
    }
}

#[test]
fn scalability_corpus_and_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/scalability/regression_corpus.json",
        "configs/suites/large_dag_scalability_regression.json",
        "evidence/perf/fixtures/large_dag.json",
        "evidence/battle/workflows/runtime/large_dag_workflow.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing scalability artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/scalability/regression_corpus.json"))
            .expect("parse scalability corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 12, "expected scalability corpus breadth");
    for coverage in [
        "dag-1000",
        "dag-10000",
        "fan-out-large",
        "fan-in-large",
        "deep-chain",
        "planner-scalability",
        "scheduler-scalability",
        "runtime-memory",
        "artifact-store-stress",
        "replay-planning",
        "diff-performance",
        "provenance-traversal",
        "run-history-stress",
        "runtime-profiling",
        "telemetry-large-dag",
        "cpu-memory-regression",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing scalability coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/large_dag_scalability_regression.json",
    ))
    .expect("parse scalability suite");
    assert_eq!(suite["id"], "large-dag-scalability-regression");
}

#[test]
fn core_runtime_and_app_tests_anchor_large_dag_contracts() {
    let core = read("crates/bijux-dag-core/tests/planner_scale_budget_contracts.rs");
    assert!(
        core.contains("planner_stress_handles_thousands_of_nodes"),
        "missing planner scalability contract token"
    );

    let core_validation = read("crates/bijux-dag-core/tests/validation_contracts_21_40.rs");
    assert!(
        core_validation.contains("validation_stress_thousands_of_nodes"),
        "missing validation scalability contract token"
    );

    let runtime = read("crates/bijux-dag-runtime/tests/concurrency_contracts.rs");
    assert!(
        runtime.contains("deterministic_stress_medium_graph_high_concurrency_stays_stable"),
        "missing runtime scalability contract token"
    );

    let app = read("crates/bijux-dag-app/tests/run_history_identity_completion_contracts.rs");
    assert!(
        app.contains("run_history_stress_suite_many_runs_is_deterministic"),
        "missing app scalability contract token"
    );
}
