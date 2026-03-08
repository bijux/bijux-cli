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
fn determinism_specs_and_reports_exist() {
    for rel in [
        "docs/spec/EXECUTION_KERNEL_DETERMINISM_GUARANTEES.md",
        "docs/spec/DETERMINISM.md",
        "docs/spec/DETERMINISTIC_SCHEDULING_CONTRACT.md",
        "docs/reports/foundation/determinism_benchmark_suite.md",
        "docs/reports/foundation/determinism_telemetry_report.md",
        "docs/reports/foundation/determinism_drift_detection_report.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty determinism surface: {rel}");
    }
}

#[test]
fn determinism_corpus_and_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/determinism/regression_corpus.json",
        "configs/suites/determinism_hardening_regression.json",
        "evidence/compare/scenarios/determinism.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing determinism artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/determinism/regression_corpus.json"))
            .expect("parse determinism corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 14, "expected determinism corpus breadth");
    for coverage in [
        "run-determinism",
        "node-ordering",
        "scheduler-determinism",
        "artifact-hash",
        "diff-ordering",
        "replay-determinism",
        "provenance-ordering",
        "explain-ordering",
        "cli-json-ordering",
        "fuzz-dag",
        "fuzz-environment",
        "fuzz-artifact-path",
        "fuzz-scheduler",
        "fuzz-runtime-events",
        "drift-detection",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing determinism coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/determinism_hardening_regression.json",
    ))
    .expect("parse determinism suite");
    assert_eq!(suite["id"], "determinism-hardening-regression");
}

#[test]
fn core_runtime_and_app_tests_anchor_determinism_contracts() {
    let core = read("crates/bijux-dag-core/tests/graph_identity_kernel_contracts.rs");
    for token in [
        "random_dag_identity_property_is_deterministic",
        "identity_is_stable_across_yaml_key_ordering_differences",
    ] {
        assert!(
            core.contains(token),
            "missing core determinism contract token: {token}"
        );
    }

    let runtime = read("crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs");
    for token in [
        "scheduler_determinism_is_stable_for_same_inputs",
        "deterministic_schedule_order",
    ] {
        assert!(
            runtime.contains(token),
            "missing runtime determinism contract token: {token}"
        );
    }

    let replay_fuzz = read("crates/bijux-dag-runtime/tests/replay_determinism_fuzz_contracts.rs");
    assert!(
        replay_fuzz.contains("replay_equivalence_fuzz_contract_preserves_equality_semantics"),
        "missing replay determinism fuzz contract token"
    );

    let app = read("crates/bijux-dag-app/tests/diff_semantic_completion_contracts.rs");
    assert!(
        app.contains("diff_schema_lockstep_human_snapshot_and_determinism_hold_under_stress"),
        "missing app determinism drift contract token"
    );
}
