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
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn system_confidence_dashboards_exist_for_381_399() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/system_invariants_dashboard.md",
        "docs/reports/foundation/replay_correctness_dashboard.md",
        "docs/reports/foundation/artifact_integrity_dashboard.md",
        "docs/reports/foundation/bundle_portability_dashboard.md",
        "docs/reports/foundation/backend_equivalence_dashboard.md",
        "docs/reports/foundation/cli_stability_dashboard.md",
        "docs/reports/foundation/schema_compatibility_dashboard.md",
        "docs/reports/foundation/benchmark_signal_dashboard.md",
        "docs/reports/foundation/repo_health_dashboard.md",
        "docs/reports/foundation/module_ownership_dashboard.md",
        "docs/reports/foundation/runtime_invariants_dashboard.md",
        "docs/reports/foundation/replay_determinism_dashboard.md",
        "docs/reports/foundation/artifact_lifecycle_dashboard.md",
        "docs/reports/foundation/bundle_compatibility_dashboard.md",
        "docs/reports/foundation/backend_divergence_dashboard.md",
        "docs/reports/foundation/cli_error_classification_dashboard.md",
        "docs/reports/foundation/schema_evolution_dashboard.md",
        "docs/reports/foundation/ci_gate_health_dashboard.md",
        "docs/reports/foundation/overall_system_readiness_dashboard.md",
    ] {
        assert!(root.join(rel).exists(), "missing dashboard artifact: {rel}");
    }
}

#[test]
fn completion_report_maps_381_400_outputs() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/system_confidence_381_400_completion_report.md"),
    )
    .expect("read completion report");

    for token in [
        "381",
        "399",
        "system_invariants_dashboard.md",
        "overall_system_readiness_dashboard.md",
        "20260308-dashboards-and-readiness-metrics.md",
    ] {
        assert!(
            report.contains(token),
            "system confidence completion report missing token: {token}"
        );
    }
}

#[test]
fn readiness_suite_declares_expected_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(
            root.join("configs/suites/system_readiness_dashboards_verification.json"),
        )
        .expect("read readiness suite"),
    )
    .expect("parse readiness suite");

    assert_eq!(suite["id"], "system-readiness-dashboards-verification");

    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "system_readiness_dashboard_contracts",
        "repo_health_contracts",
        "backend_semantic_equivalence_contracts",
        "cli_stability_guarantees_contracts",
        "schema_compatibility_guarantees_contracts",
    ] {
        assert!(
            commands.contains(token),
            "missing readiness suite token: {token}"
        );
    }
}
