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
fn internal_contract_561_580_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "configs/policy/internal_contract_governance.json",
        "docs/spec/INTERNAL_CONTRACT_DISCIPLINE_POLICY.md",
        "docs/reports/foundation/internal_contract_inventory_561_580_report.md",
        "docs/reports/foundation/internal_contracts_without_direct_tests_report.md",
        "docs/reports/foundation/internal_contracts_without_fixtures_report.md",
        "docs/reports/foundation/internal_contracts_without_docs_report.md",
        "docs/reports/foundation/internal_contract_coverage_report.md",
        "docs/reports/foundation/internal_contract_stability_report.md",
        "docs/reports/foundation/contract_to_fixture_map_report.md",
        "docs/reports/foundation/contract_to_suite_map_report.md",
        "docs/reports/foundation/internal_contract_health_dashboard.md",
        "docs/reports/foundation/internal_contract_review_checklist.md",
        "docs/reports/foundation/internal_contract_drift_detection_report.md",
        "docs/reports/foundation/internal_contract_discipline_561_580_status_report.md",
        "configs/suites/internal_contract_verification.json",
        "docs/adr/20260308-internal-contract-discipline.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing internal contract artifact: {rel}"
        );
    }
}

#[test]
fn internal_contract_governance_rules_are_enabled() {
    let root = repo_root();
    let policy: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/internal_contract_governance.json"))
            .expect("read internal contract governance policy"),
    )
    .expect("parse internal contract governance policy");

    assert_eq!(policy["format"], "internal-contract-governance/v1");
    let rules = policy["rules"].as_object().expect("rules object");
    for key in [
        "every_internal_contract_requires_direct_test_and_owner",
        "stable_internal_contract_requires_docs_or_spec_link",
        "contract_drift_detection_required",
    ] {
        assert_eq!(rules.get(key).and_then(Value::as_bool), Some(true));
    }
}

#[test]
fn internal_contract_status_report_maps_561_580() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/internal_contract_discipline_561_580_status_report.md"),
    )
    .expect("read internal contract status report");

    for token in [
        "561-569",
        "570-573",
        "574-580",
        "internal_contract_verification.json",
        "20260308-internal-contract-discipline.md",
    ] {
        assert!(report.contains(token), "missing status token: {token}");
    }
}

#[test]
fn internal_contract_suite_contains_expected_boundary_contracts() {
    let root = repo_root();
    let suite: Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/suites/internal_contract_verification.json"))
            .expect("read internal contract suite"),
    )
    .expect("parse internal contract suite");

    assert_eq!(suite["id"], "internal-contract-verification");
    let commands = suite["commands"]
        .as_array()
        .expect("commands array")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for token in [
        "internal_contract_governance_contracts",
        "dependency_boundary_contracts",
        "app_service_boundary_progress_contracts",
        "runtime_scope_reports_contracts",
        "backend_adapter_scope_reports_contracts",
    ] {
        assert!(commands.contains(token), "missing suite token: {token}");
    }
}
