use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use super::{
    build_audit_report, build_diff_report, build_generators_report, build_migrated_report,
    build_remaining_report, build_requirement_catalog_report, build_status_contracts_report,
};

#[test]
fn contract_reports_are_shaped() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(build_migrated_report(&root).get("migrated").is_some());
    assert!(build_remaining_report(&root)
        .get("remaining_root_maintenance")
        .is_some());
    assert!(build_diff_report(&root).get("remaining").is_some());
    let audit = build_audit_report(&root);
    assert!(audit.get("diff").is_some());
    assert!(audit.get("status_generators").is_some());
    assert!(audit.get("status_contracts").is_some());
    assert!(audit.get("requirement_catalog").is_some());
    assert!(audit.get("flaky_tests").is_some());
}

#[test]
fn status_generator_ids_are_stable_and_prefixed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rows = build_generators_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!rows.is_empty());
    for row in rows {
        let id = row
            .get("generator_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(id.starts_with("GEN-STATUS-"));
    }
}

#[test]
fn requirement_ids_use_req_prefix() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rows = build_requirement_catalog_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!rows.is_empty(), "requirement catalog must not be empty");
    for row in rows {
        let id = row
            .get("requirement_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(id.starts_with("REQ-"));
    }
}

#[test]
fn requirement_catalog_covers_status_contract_inventory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let requirement_rows = build_requirement_catalog_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    let status_requirements: BTreeSet<String> = requirement_rows
        .into_iter()
        .filter(|row| row.get("domain").and_then(serde_json::Value::as_str) == Some("STATUS"))
        .filter_map(|row| {
            row.get("contract_id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    let status_inventory: BTreeSet<String> = build_status_contracts_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("contract_id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert_eq!(
        status_requirements, status_inventory,
        "status requirement catalog must mirror status inventory"
    );
}

#[test]
fn migrated_report_controls_are_non_empty_and_removed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let controls = build_migrated_report(&root)
        .get("migration_controls")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!controls.is_empty(), "migration controls must not be empty");
    let remaining: Vec<String> = controls
        .into_iter()
        .filter(|row| row.get("status").and_then(serde_json::Value::as_str) == Some("remaining"))
        .filter_map(|row| {
            row.get("control_id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert!(
        remaining.is_empty(),
        "expected migration controls to be removed; remaining controls:\n{}",
        remaining.join("\n")
    );
}

#[test]
fn status_contract_ids_are_stable_and_prefixed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let rows = build_status_contracts_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();
    assert!(!rows.is_empty());
    for row in rows {
        let id = row
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("");
        assert!(id.starts_with("STATUS-CONTRACT-"));
    }
}

#[test]
fn ci_status_contract_ids_match_status_inventory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let ci = fs::read_to_string(root.join(".github/workflows/ci.yml")).expect("read ci");
    let referenced: BTreeSet<String> = ci
        .split_whitespace()
        .map(|token| {
            token.trim_matches(|ch: char| {
                !ch.is_ascii_uppercase() && !ch.is_ascii_digit() && ch != '-'
            })
        })
        .filter(|token| token.starts_with("STATUS-CONTRACT-"))
        .map(ToString::to_string)
        .collect();
    assert!(
        !referenced.is_empty(),
        "expected STATUS-CONTRACT IDs in CI workflow"
    );

    let valid: BTreeSet<String> = build_status_contracts_report(&root)
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|row| {
            row.get("contract_id")
                .and_then(serde_json::Value::as_str)
                .map(ToString::to_string)
        })
        .collect();
    assert!(!valid.is_empty(), "expected status contract inventory rows");

    let missing: Vec<String> = referenced.difference(&valid).cloned().collect();
    assert!(
        missing.is_empty(),
        "CI references unknown STATUS-CONTRACT IDs; missing:\n{}",
        missing.join("\n")
    );
}
