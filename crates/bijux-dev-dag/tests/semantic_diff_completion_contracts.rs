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
fn semantic_diff_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SEMANTIC_DIFF_COMPLETENESS_CONTRACT.md",
        "docs/reports/foundation/semantic_diff_coverage_report.md",
        "docs/reports/foundation/semantic_diff_telemetry_report.md",
        "docs/reports/foundation/semantic_diff_diagnostics_report.md",
        "docs/reports/foundation/semantic_diff_visualization_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty semantic diff artifact: {rel}");
    }
}

#[test]
fn semantic_diff_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/semantic_diff/regression_corpus.json",
    ))
    .expect("parse semantic diff corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected broad semantic diff corpus");

    for coverage in [
        "graph-diff-semantics",
        "run-diff-semantics",
        "artifact-diff-semantics",
        "environment-diff-semantics",
        "backend-capability-diff-semantics",
        "correctness",
        "classification-consistency",
        "determinism-across-runs",
        "determinism-across-platforms",
        "regression-fixtures",
        "fuzz-suite",
        "anomaly-detection",
        "performance-benchmarks",
        "explainability",
        "telemetry",
        "visualization-data",
        "diagnostics-tooling",
        "documentation",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing semantic diff coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/semantic_diff_verification.json"))
            .expect("parse semantic diff suite");
    assert_eq!(suite["id"], "semantic-diff-verification");
}

#[test]
fn semantic_diff_surfaces_anchor_existing_app_runtime_and_contract_tests() {
    let app_diff = read("crates/bijux-dag-app/tests/diff_semantic_completion_contracts.rs");
    for token in [
        "semantic_diff_classification_and_explain_surfaces_are_stable",
        "diff_schema_lockstep_human_snapshot_and_determinism_hold_under_stress",
    ] {
        assert!(app_diff.contains(token), "missing app diff token: {token}");
    }

    let replay_diff = read("crates/bijux-dag-app/tests/replay_diff_hardening_contract.rs");
    assert!(
        replay_diff.contains("replay_diff_and_explain_schemas_are_lockstep_and_semantic"),
        "missing replay diff semantic anchor"
    );

    let runtime_planner = read("crates/bijux-dag-runtime/tests/planner_analysis_contract.rs");
    assert!(
        runtime_planner.contains("planner_supports_closure_replay_backfill_diff_and_explain"),
        "missing runtime diff planner anchor"
    );

    let dev_contract = read("crates/bijux-dev-dag/tests/diff_semantics_contracts.rs");
    assert!(
        dev_contract.contains("diff_semantics_docs_and_schemas_exist"),
        "missing dev diff contract anchor"
    );
}

