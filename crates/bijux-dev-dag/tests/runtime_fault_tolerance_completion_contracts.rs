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
fn runtime_fault_tolerance_contract_and_reports_exist() {
    for rel in [
        "docs/spec/RUNTIME_FAULT_TOLERANCE_CONTRACT.md",
        "docs/reports/foundation/runtime_fault_tolerance_benchmarks.md",
        "docs/reports/foundation/runtime_recovery_latency_report.md",
        "docs/reports/foundation/runtime_fault_tolerance_telemetry_report.md",
        "docs/reports/foundation/runtime_fault_tolerance_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty runtime fault tolerance artifact: {rel}"
        );
    }
}

#[test]
fn runtime_fault_tolerance_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/runtime_fault_tolerance/regression_corpus.json",
    ))
    .expect("parse runtime fault tolerance corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(
        cases.len() >= 18,
        "expected broad runtime fault tolerance corpus"
    );

    for coverage in [
        "crash-recovery",
        "restart-continuation",
        "state-persistence",
        "scheduler-restart",
        "worker-reconnect",
        "artifact-recovery",
        "replay-recovery",
        "cancellation-recovery",
        "event-log-recovery",
        "partial-run-recovery",
        "failure-detection",
        "resilience-benchmarks",
        "recovery-latency",
        "regression-fixtures",
        "crash-simulation",
        "failure-injection",
        "resilience-telemetry",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing runtime fault tolerance coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/runtime_fault_tolerance_verification.json",
    ))
    .expect("parse runtime fault tolerance suite");
    assert_eq!(suite["id"], "runtime-fault-tolerance-verification");
}

#[test]
fn runtime_fault_tolerance_surfaces_anchor_existing_recovery_tests() {
    let failure_recovery =
        read("crates/bijux-dev-dag/tests/failure_recovery_completion_contracts.rs");
    for token in [
        "recovery_required_for_checkpoint_without_terminal_completion",
        "recovery_required_for_partial_artifact_or_interrupted_execution",
        "fault_resilience_integration",
    ] {
        assert!(
            failure_recovery.contains(token),
            "missing failure-recovery anchor token: {token}"
        );
    }

    let runtime_recovery = read("crates/bijux-dag-runtime/tests/runtime_recovery_contracts.rs");
    assert!(
        runtime_recovery
            .contains("recovery_required_for_partial_artifact_or_interrupted_execution"),
        "missing runtime recovery contract anchor"
    );

    let runtime_invariants =
        read("crates/bijux-dag-runtime/tests/runtime_engine_invariants_contracts.rs");
    assert!(
        runtime_invariants.contains(
            "runtime_crash_recovery_simulation_requires_recovery_on_checkpointed_interruptions"
        ),
        "missing runtime crash simulation anchor"
    );
}
