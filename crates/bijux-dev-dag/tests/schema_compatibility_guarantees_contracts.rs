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
fn schema_compatibility_completion_artifacts_exist_for_361_380() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/schema_compatibility_361_380_completion_report.md",
        "docs/reports/foundation/schema_compatibility_matrix_report.md",
        "docs/reports/foundation/schema_change_detection_ci_report.md",
        "docs/reports/foundation/schema_drift_diagnostics_report.md",
        "docs/reports/foundation/schema_stability_dashboard.md",
        "docs/reports/foundation/schema_usage_inventory_report.md",
        "docs/reports/foundation/schema_compatibility_heatmap.md",
        "configs/suites/schema_compatibility_verification.json",
        "docs/adr/20260308-schema-compatibility-guarantees.md",
    ] {
        assert!(root.join(rel).exists(), "missing schema artifact: {rel}");
    }
}

#[test]
fn schema_completion_report_maps_each_361_380_slot() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/schema_compatibility_361_380_completion_report.md"),
    )
    .expect("read completion report");

    for token in [
        "361-366",
        "367-371",
        "372-378",
        "379-380",
        "schema_compatibility_matrix_report.md",
        "schema_change_detection_ci_report.md",
        "schema_drift_diagnostics_report.md",
        "schema_stability_dashboard.md",
        "schema_usage_inventory_report.md",
        "schema_compatibility_heatmap.md",
        "schema_compatibility_verification.json",
        "20260308-schema-compatibility-guarantees.md",
    ] {
        assert!(
            report.contains(token),
            "completion report missing mapping token: {token}"
        );
    }
}

#[test]
fn schema_compatibility_verification_suite_contains_expected_contract_commands() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/schema_compatibility_verification.json"))
            .expect("read suite"),
    )
    .expect("parse suite");

    assert_eq!(suite["id"], "schema-compatibility-verification");

    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|value| value.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "schema_governance_contracts",
        "schema_evolution_completion_contracts",
        "proof_schema_compatibility_contracts",
        "schema_compatibility_guarantees_contracts",
    ] {
        assert!(commands.contains(token), "missing suite command token: {token}");
    }
}
