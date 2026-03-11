#![forbid(unsafe_code)]
//! Suite catalog coverage contracts.

use std::collections::BTreeSet;
use std::path::Path;

use bijux_dev_cli::contracts::status::build_inventory_report;

#[test]
fn status_inventory_contains_contracts_from_all_suite_domains() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let inventory = build_inventory_report(&root);
    let rows = inventory
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let ids: BTreeSet<String> = rows
        .iter()
        .filter_map(|row| row.get("contract_id").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect();

    assert!(ids.contains("STATUS-CONTRACT-GENERATE-REPO-HEALTH-REPORTS"));
    assert!(ids.contains("STATUS-CONTRACT-GENERATE-INSTALL-TRUTH-REPORTS"));
    assert!(ids.contains("STATUS-CONTRACT-GENERATE-RELEASE-BUILD-REPORTS"));
    assert!(ids.contains("STATUS-CONTRACT-GENERATE-PARSER-FUZZ-HARDENING-REPORTS"));
}

#[test]
fn status_inventory_contract_ids_are_unique() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let inventory = build_inventory_report(&root);
    let rows = inventory
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .cloned()
        .unwrap_or_default();

    let mut ids = BTreeSet::new();
    for row in rows {
        let id = row
            .get("contract_id")
            .and_then(serde_json::Value::as_str)
            .expect("contract_id must be present");
        assert!(ids.insert(id.to_string()), "duplicate contract id: {id}");
    }
}
