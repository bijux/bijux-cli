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
fn system_reliability_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_RELIABILITY_GUARANTEES_CONTRACT.md",
        "docs/reports/foundation/system_reliability_coverage_report.md",
        "docs/reports/foundation/system_reliability_benchmarks_report.md",
        "docs/reports/foundation/system_reliability_telemetry_report.md",
        "docs/reports/foundation/system_reliability_review_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty system reliability artifact: {rel}"
        );
    }
}

#[test]
fn system_reliability_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_reliability/regression_corpus.json",
    ))
    .expect("parse system reliability corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 13, "expected broad system reliability corpus");

    for coverage in [
        "runtime-reliability-targets",
        "artifact-reliability-targets",
        "replay-reliability-targets",
        "scheduler-reliability-targets",
        "reliability-target-tests",
        "regression-fixtures",
        "stress-suite",
        "telemetry-reporting",
        "anomaly-detection",
        "runtime-reliability-benchmarks",
        "artifact-reliability-benchmarks",
        "replay-reliability-benchmarks",
        "diagnostics-tooling",
        "failure-simulation-tests",
        "monitoring-tests",
        "architecture-review",
        "verification-tools",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing system reliability coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/system_reliability_verification.json",
    ))
    .expect("parse system reliability suite");
    assert_eq!(suite["id"], "system-reliability-verification");
}

#[test]
fn system_reliability_surfaces_anchor_existing_reliability_completion_contracts() {
    let runtime = read("crates/bijux-dev-dag/tests/runtime_fault_tolerance_completion_contracts.rs");
    assert!(
        runtime.contains("runtime_fault_tolerance_corpus_and_suite_are_machine_readable"),
        "missing runtime reliability anchor"
    );

    let artifact = read("crates/bijux-dev-dag/tests/artifact_durability_completion_contracts.rs");
    assert!(
        artifact.contains("artifact_durability_corpus_and_suite_are_machine_readable"),
        "missing artifact reliability anchor"
    );

    let replay = read("crates/bijux-dev-dag/tests/replay_equivalence_completion_contracts.rs");
    assert!(
        replay.contains("replay_equivalence_corpus_and_suite_are_machine_readable"),
        "missing replay reliability anchor"
    );

    let adversarial = read("crates/bijux-dev-dag/tests/adversarial_system_resilience_completion_contracts.rs");
    assert!(
        adversarial.contains("scheduler-starvation"),
        "missing scheduler resilience anchor"
    );
}

