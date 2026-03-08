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
fn internal_invariants_contract_and_reports_exist() {
    for rel in [
        "docs/spec/INTERNAL_INVARIANTS_CONSISTENCY_CONTRACT.md",
        "docs/reports/foundation/internal_invariants_telemetry_report.md",
        "docs/reports/foundation/internal_invariants_debugging_report.md",
        "docs/reports/foundation/internal_invariants_coverage_report.md",
        "docs/reports/foundation/internal_invariants_performance_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty internal invariants artifact: {rel}"
        );
    }
}

#[test]
fn internal_invariants_corpus_and_suite_are_machine_readable() {
    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/invariants/regression_corpus.json"))
            .expect("parse internal invariants corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 18,
        "expected broad internal invariants corpus"
    );

    for coverage in [
        "graph-state",
        "planner-state",
        "runtime-state",
        "scheduler-state",
        "artifact-store-state",
        "run-history-state",
        "violation-detection",
        "failure-logging",
        "monitoring-telemetry",
        "regression-fixtures",
        "stress",
        "fuzz",
        "coverage-reporting",
        "performance-impact",
        "debug-tooling",
        "trace-capture",
        "failure-simulation",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing internal invariants coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/internal_invariants_verification.json",
    ))
    .expect("parse internal invariants suite");
    assert_eq!(suite["id"], "internal-invariants-verification");
}

#[test]
fn internal_invariants_surfaces_anchor_existing_runtime_invariant_tests() {
    let invariants_spec = read("docs/spec/FORMAL_INVARIANTS.md");
    for token in [
        "INV-GRAPH-SHAPE-001",
        "INV-PLAN-SHAPE-001",
        "INV-SCHED-READY-001",
        "INV-RUN-COUNTS-001",
        "INV-TRACE-TIME-001",
        "INV-CACHE-PROOF-001",
    ] {
        assert!(
            invariants_spec.contains(token),
            "missing formal invariant token: {token}"
        );
    }

    let runtime_invariant_tests =
        read("crates/bijux-dag-runtime/src/internal/testing/invariants_tests.rs");
    for token in [
        "run_summary_invariant_matches_trace_totals",
        "run_summary_invariant_detects_mismatch",
        "trace_time_invariant_requires_monotonic_timestamps",
        "invariant_registry_ids_are_stable_and_unique",
    ] {
        assert!(
            runtime_invariant_tests.contains(token),
            "missing runtime invariant test anchor token: {token}"
        );
    }

    let scheduler_hardening = read("crates/bijux-dev-dag/tests/scheduler_hardening_contracts.rs");
    assert!(
        scheduler_hardening.contains("scheduler-invariants"),
        "missing scheduler invariants governance anchor"
    );
}
