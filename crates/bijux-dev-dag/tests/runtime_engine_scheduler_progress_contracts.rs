use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn runtime_engine_scheduler_reports_are_present_and_named() {
    for rel in [
        "docs/reports/foundation/runtime_engine_scheduler_hotpath_benchmark.md",
        "docs/reports/foundation/runtime_state_machine_cancellation_trace_fixtures.md",
        "docs/reports/foundation/runtime_scheduler_contract_drift_report.md",
        "docs/reports/foundation/runtime_engine_scheduler_fast_suite.md",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_state_machine_invariants_contracts.rs",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}

#[test]
fn scheduler_contract_drift_report_declares_no_drift() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/runtime_scheduler_contract_drift_report.md"),
    )
    .expect("drift report");
    assert!(raw.contains("Status: `no-drift`"));
    assert!(raw.contains("scheduler_contract_profile()"));
}

#[test]
fn runtime_fast_suite_report_points_to_runtime_contract_files() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/runtime_engine_scheduler_fast_suite.md"),
    )
    .expect("fast suite report");
    for rel in [
        "runtime_scheduler_state_machine_invariants_contracts.rs",
        "runtime_execution_resilience_contracts.rs",
        "state_machine_transitions.rs",
        "scheduler_contract.rs",
    ] {
        assert!(raw.contains(rel), "missing fast suite member {rel}");
    }
}
