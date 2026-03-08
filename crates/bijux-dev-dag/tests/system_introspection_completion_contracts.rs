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
fn system_introspection_contract_and_reports_exist() {
    for rel in [
        "docs/spec/SYSTEM_INTROSPECTION_COMMANDS_CONTRACT.md",
        "docs/reports/foundation/system_introspection_command_coverage_report.md",
        "docs/reports/foundation/system_introspection_schema_report.md",
        "docs/reports/foundation/system_introspection_snapshot_report.md",
        "docs/reports/foundation/system_introspection_benchmarks_report.md",
        "docs/reports/foundation/system_introspection_telemetry_report.md",
        "docs/reports/foundation/system_introspection_anomaly_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty introspection artifact: {rel}"
        );
    }
}

#[test]
fn system_introspection_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/system_introspection/regression_corpus.json",
    ))
    .expect("parse system introspection corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 18, "expected broad introspection corpus");

    for coverage in [
        "execution-trace-inspection",
        "artifact-store-health-inspection",
        "run-history-integrity-inspection",
        "scheduler-state-inspection",
        "backend-capabilities-inspection",
        "replay-compatibility-inspection",
        "cache-state-inspection",
        "provenance-graph-inspection",
        "artifact-lineage-graph-inspection",
        "runtime-diagnostics-inspection",
        "deterministic-ordering",
        "schema-validation",
        "snapshot-stability",
        "regression-fixtures",
        "performance-benchmarks",
        "telemetry",
        "anomaly-detection",
        "stress",
        "verification-suite",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing introspection coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/system_introspection_verification.json",
    ))
    .expect("parse system introspection suite");
    assert_eq!(suite["id"], "system-introspection-verification");
}

#[test]
fn system_introspection_surfaces_anchor_existing_command_implementations() {
    let cli = read("crates/bijux-dev-dag/src/commands/cli.rs");
    for token in [
        "StorageHealth",
        "BackendRegistryReport",
        "DriftDashboard",
        "CacheCoverageReport",
        "DagCommand",
    ] {
        assert!(
            cli.contains(token),
            "missing introspection CLI surface token: {token}"
        );
    }

    let commands = read("crates/bijux-dev-dag/src/commands/mod.rs");
    for token in [
        "dag.run-inspect",
        "run_dag_run_inspect",
        "dag.scheduler-timeline",
        "run_dag_scheduler_timeline",
        "run_storage_health",
        "run_backend_registry_report",
        "run_cache_coverage_report",
        "run_evidence_replay_verify",
        "run_drift_dashboard",
        "run_evidence_drift_verify",
    ] {
        assert!(
            commands.contains(token),
            "missing introspection command implementation token: {token}"
        );
    }

    let runtime_observability =
        read("crates/bijux-dev-dag/tests/runtime_observability_completion_contracts.rs");
    assert!(
        runtime_observability.contains("diagnostics-snapshot"),
        "missing diagnostics snapshot introspection anchor"
    );
}
