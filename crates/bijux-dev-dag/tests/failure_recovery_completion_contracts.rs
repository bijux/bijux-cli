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
fn failure_taxonomy_and_recovery_specs_exist() {
    for rel in [
        "docs/spec/FAILURE_TAXONOMY_CONTRACT.md",
        "docs/RUN_RECOVERY_AND_RESILIENCE.md",
        "docs/spec/ERROR_TAXONOMY.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing failure or recovery spec: {rel}"
        );
    }
}

#[test]
fn failure_and_recovery_contract_tests_cover_required_surfaces() {
    let failure_tests = read("crates/bijux-dag-runtime/tests/runtime_failure_contracts.rs");
    for token in [
        "failure_paths_are_classified_explicitly",
        "failure_classification_matrix_covers_policy_cache_artifact_and_adapter",
        "failure_taxonomy_transient_and_permanent_mapping_is_explicit",
    ] {
        assert!(
            failure_tests.contains(token),
            "missing failure classification contract test: {token}"
        );
    }

    let recovery_tests = read("crates/bijux-dag-runtime/tests/runtime_recovery_contracts.rs");
    for token in [
        "recovery_required_for_checkpoint_without_terminal_completion",
        "recovery_required_for_partial_artifact_or_interrupted_execution",
    ] {
        assert!(
            recovery_tests.contains(token),
            "missing recovery contract test: {token}"
        );
    }

    let app_faults = read("crates/bijux-dag-app/tests/fault_resilience_integration.rs");
    for token in [
        "fault_subprocess_timeout_classification",
        "fault_trace_file_missing_detected",
        "fault_outputs_index_corruption_detected",
    ] {
        assert!(
            app_faults.contains(token),
            "missing app failure-path coverage: {token}"
        );
    }
}

#[test]
fn failure_regression_corpus_stress_suite_and_benchmark_report_are_present() {
    for rel in [
        "evidence/cache/failure/regression_corpus.json",
        "configs/suites/failure_recovery_injection_stress.json",
        "docs/reports/foundation/failure_handling_benchmarks.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing failure governance artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read("evidence/cache/failure/regression_corpus.json"))
        .expect("parse failure regression corpus");
    assert_eq!(corpus["version"], "v1");
    assert!(
        corpus["cases"].as_array().expect("cases").len() >= 6,
        "expected failure regression corpus breadth"
    );

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/failure_recovery_injection_stress.json",
    ))
    .expect("parse failure recovery suite");
    assert_eq!(suite["id"], "failure-recovery-injection-stress");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "runtime_failure_contracts",
        "runtime_recovery_contracts",
        "fault_resilience_integration",
        "failure_recovery_completion_contracts",
    ] {
        assert!(commands.contains(token), "missing suite command token: {token}");
    }
}
