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
fn introspection_architecture_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_INTROSPECTION_ARCHITECTURE_CONTRACT.md",
        "docs/reports/foundation/system_introspection_architecture_coverage_report.md",
        "docs/reports/foundation/system_introspection_architecture_benchmarks_report.md",
        "docs/reports/foundation/system_introspection_architecture_reliability_report.md",
        "docs/reports/foundation/system_introspection_architecture_review_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty introspection architecture artifact: {rel}"
        );
    }
}

#[test]
fn introspection_architecture_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_introspection_architecture/regression_corpus.json",
    ))
    .expect("parse introspection architecture corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected broad introspection architecture corpus");

    for coverage in [
        "command-correctness",
        "json-schema-stability",
        "determinism",
        "failure-behavior",
        "regression-fixtures",
        "performance-benchmarks",
        "anomaly-detection",
        "telemetry-reporting",
        "diagnostics-tooling",
        "visualization-data",
        "documentation",
        "fuzz-suite",
        "stress-suite",
        "reliability-tests",
        "architecture-review",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing introspection architecture coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/system_introspection_architecture_verification.json",
    ))
    .expect("parse introspection architecture suite");
    assert_eq!(suite["id"], "system-introspection-architecture-verification");
}

#[test]
fn introspection_architecture_surfaces_anchor_existing_command_and_completion_contracts() {
    let cli = read("crates/bijux-dev-dag/src/commands/cli.rs");
    for token in ["StorageHealth", "BackendRegistryReport", "DriftDashboard", "DagCommand"] {
        assert!(cli.contains(token), "missing introspection CLI token: {token}");
    }

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "run_dag_run_inspect",
        "run_dag_scheduler_timeline",
        "run_storage_health",
        "run_backend_registry_report",
        "run_cache_coverage_report",
        "run_drift_dashboard",
    ] {
        assert!(commands.contains(token), "missing introspection command token: {token}");
    }

    let prior_completion = read("crates/bijux-dev-dag/tests/system_introspection_completion_contracts.rs");
    assert!(
        prior_completion.contains("system_introspection_surfaces_anchor_existing_command_implementations"),
        "missing prior introspection completion anchor"
    );
}

