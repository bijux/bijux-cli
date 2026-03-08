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
fn runtime_telemetry_schema_and_spec_surfaces_exist() {
    for rel in [
        "configs/schema/operator/runtime_telemetry.schema.json",
        "docs/spec/RUNTIME_TELEMETRY_SCHEMA.md",
        "docs/spec/OBSERVABILITY_CONTRACT.md",
        "docs/spec/DIAGNOSTICS_MODES.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty runtime observability surface: {rel}"
        );
    }
}

#[test]
fn telemetry_regression_corpus_and_stress_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/telemetry/regression_corpus.json",
        "configs/suites/runtime_observability_stress.json",
        "docs/reports/foundation/runtime_observability_benchmarks.md",
        "docs/reports/foundation/telemetry_coverage_report.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing runtime observability artifact: {rel}"
        );
    }

    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/telemetry/regression_corpus.json"))
            .expect("parse telemetry corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 10, "expected telemetry corpus breadth");
    for coverage in [
        "node-duration",
        "run-duration",
        "scheduler",
        "cache-hit",
        "cache-miss",
        "replay",
        "diff",
        "prove",
        "verify",
        "artifact-io",
        "backend-capability",
        "failure-path",
        "cancellation-path",
        "partial-rerun",
        "telemetry-json-schema",
        "diagnostics-snapshot",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing telemetry coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/runtime_observability_stress.json"))
            .expect("parse observability suite");
    assert_eq!(suite["id"], "runtime-observability-stress");
}

#[test]
fn runtime_and_app_tests_anchor_diagnostics_and_telemetry_surfaces() {
    let app_faults = read("crates/bijux-dag-app/tests/fault_resilience_integration.rs");
    for token in ["verify", "run_matches", "create_corrupted_run_dir"] {
        assert!(
            app_faults.contains(token),
            "missing app diagnostics anchor token: {token}"
        );
    }

    let app_snapshots = read("crates/bijux-dag-app/tests/operator_human_snapshot_contracts.rs");
    for token in [
        "prove_human_output_snapshot_is_stable",
        "verify_human_output_snapshot_is_stable",
    ] {
        assert!(
            app_snapshots.contains(token),
            "missing operator diagnostics snapshot token: {token}"
        );
    }

    let runtime_resilience =
        read("crates/bijux-dag-runtime/tests/runtime_execution_resilience_contracts.rs");
    for token in ["verify_post_run_state_consistency", "classify_failure"] {
        assert!(
            runtime_resilience.contains(token),
            "missing runtime telemetry/failure anchor token: {token}"
        );
    }
}
