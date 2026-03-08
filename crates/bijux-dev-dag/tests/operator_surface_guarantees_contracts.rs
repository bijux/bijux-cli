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
fn operator_surface_441_460_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/operator_command_inventory_by_value_report.md",
        "docs/reports/foundation/operator_command_value_map_report.md",
        "docs/reports/foundation/operator_command_redundancy_report.md",
        "docs/reports/foundation/operator_command_merge_candidates_report.md",
        "docs/reports/foundation/compact_operator_command_set_report.md",
        "docs/reports/foundation/operator_command_complexity_report.md",
        "docs/reports/foundation/operator_command_usage_heatmap_report.md",
        "docs/reports/foundation/operator_surface_441_460_status_report.md",
        "docs/reports/foundation/operator_surface_dashboard.md",
        "configs/suites/operator_surface_verification.json",
        "docs/adr/20260308-stable-operator-surface.md",
    ] {
        assert!(root.join(rel).exists(), "missing operator surface artifact: {rel}");
    }
}

#[test]
fn operator_surface_status_report_maps_441_460() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/operator_surface_441_460_status_report.md"),
    )
    .expect("read operator surface status report");

    for token in [
        "441-450",
        "451-455",
        "456-460",
        "operator_surface_verification.json",
        "20260308-stable-operator-surface.md",
    ] {
        assert!(
            report.contains(token),
            "operator status report missing token: {token}"
        );
    }
}

#[test]
fn operator_surface_suite_contains_expected_commands() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/operator_surface_verification.json"))
            .expect("read operator suite"),
    )
    .expect("parse operator suite");

    assert_eq!(suite["id"], "operator-surface-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "operator_surface_guarantees_contracts",
        "operator_ux_contract",
        "help_surface_contracts",
        "operator_schema_lockstep_contracts",
        "route_output_wording_snapshot_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}

#[test]
fn core_operator_flow_tests_are_present() {
    let root = repo_root();
    for rel in [
        "crates/bijux-dag-app/tests/operator_ux_contract.rs",
        "crates/bijux-dag-app/tests/help_surface_contracts.rs",
        "crates/bijux-dag-app/tests/operator_schema_lockstep_contracts.rs",
        "crates/bijux-dag-app/tests/route_output_wording_snapshot_contracts.rs",
        "crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs",
    ] {
        assert!(root.join(rel).exists(), "missing app operator flow test: {rel}");
    }
}
