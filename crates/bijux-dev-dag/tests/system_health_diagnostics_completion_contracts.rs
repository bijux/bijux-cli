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
fn system_health_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_HEALTH_DIAGNOSTICS_CONTRACT.md",
        "docs/reports/foundation/system_health_diagnostics_documentation.md",
        "docs/reports/foundation/system_health_reporting_dashboard.md",
        "docs/reports/foundation/health_regression_summary_report.md",
        "docs/reports/foundation/automated_health_verification_suite.md",
    ] {
        let body = read(rel);
        assert!(!body.trim().is_empty(), "empty health artifact: {rel}");
    }
}

#[test]
fn system_health_corpus_and_suite_are_machine_readable() {
    let corpus: Value =
        serde_json::from_str(&read("evidence/cache/system_health/regression_corpus.json"))
            .expect("parse system health corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 12, "expected broad system health corpus");

    for coverage in [
        "system-health-command",
        "artifact-store-health",
        "run-history-health",
        "runtime-engine-health",
        "scheduler-health",
        "adapter-health",
        "backend-capability-health",
        "bundle-integrity",
        "replay-integrity",
        "diff-consistency",
        "provenance-integrity",
        "artifact-lineage",
        "telemetry-inspection",
        "anomaly-detection",
        "determinism-drift",
        "drift-detection",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing system health coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/system_health_verification.json"))
            .expect("parse system health suite");
    assert_eq!(suite["id"], "system-health-verification");
}

#[test]
fn health_surfaces_anchor_existing_command_and_drift_diagnostics() {
    let trust_health = read("crates/bijux-dev-dag/src/bin/trust_health.rs");
    assert!(
        trust_health.contains("fn main()"),
        "missing trust-health command surface"
    );

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "run_storage_health",
        "run_drift_dashboard",
        "run_evidence_drift_verify",
    ] {
        assert!(
            commands.contains(token),
            "missing health diagnostics command token: {token}"
        );
    }

    let repo_health = read("crates/bijux-dev-dag/tests/repo_health_contracts.rs");
    assert!(
        repo_health.contains("repo-health"),
        "missing repo health contract anchor"
    );
}
