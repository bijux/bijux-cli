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
fn execution_trace_contract_and_reports_exist() {
    for rel in [
        "docs/spec/EXECUTION_TRACE_RECORDS_CONTRACT.md",
        "docs/reports/foundation/execution_trace_benchmarks.md",
        "docs/reports/foundation/execution_trace_regression_fixtures.md",
        "docs/reports/foundation/execution_trace_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty execution trace artifact: {rel}"
        );
    }
}

#[test]
fn execution_trace_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/execution_trace/regression_corpus.json",
    ))
    .expect("parse execution trace corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 18, "expected broad trace corpus");

    for coverage in [
        "node-start",
        "node-completion",
        "scheduler-decision",
        "artifact-write",
        "artifact-read",
        "replay-decision",
        "cache-hit-miss",
        "backend-dispatch",
        "worker-communication",
        "ordering-determinism",
        "success-completeness",
        "failure-completeness",
        "cancel-completeness",
        "restart-persistence",
        "serialization-schema",
        "corruption-detection",
        "replay-inspection",
        "trace-performance",
        "regression-fixtures",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing execution trace coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/execution_trace_regression.json"))
            .expect("parse execution trace suite");
    assert_eq!(suite["id"], "execution-trace-regression");
}

#[test]
fn execution_trace_surfaces_anchor_runtime_and_app_contracts() {
    let app = read("crates/bijux-dag-app/src/lib.rs");
    for token in [
        "trace schema parse failed",
        "trace missing",
        "INV-TRACE-ATTEMPT-001",
        "trace_time_order_ok",
    ] {
        assert!(
            app.contains(token),
            "missing trace invariant anchor token: {token}"
        );
    }

    let diagnostics = read("crates/bijux-dag-app/src/routes/diagnostics_routes.rs");
    assert!(
        diagnostics.contains("trace_artifact_payload"),
        "missing trace artifact inspection anchor"
    );

    let observability =
        read("crates/bijux-dev-dag/tests/runtime_observability_completion_contracts.rs");
    assert!(
        observability.contains("diagnostics-snapshot"),
        "missing runtime observability diagnostics anchor"
    );
}
