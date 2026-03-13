#![forbid(unsafe_code)]
//! Status inventory contracts.

use std::path::Path;

use bijux_dev_cli::contracts::status::{build_inventory_report, run_all_contracts, run_contract};

#[test]
fn status_inventory_rows_are_sorted_and_well_formed() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let inventory = build_inventory_report(&root);
    let rows = inventory
        .get("rows")
        .and_then(serde_json::Value::as_array)
        .expect("rows array");

    let ids: Vec<String> = rows
        .iter()
        .filter_map(|row| row.get("contract_id").and_then(serde_json::Value::as_str))
        .map(ToString::to_string)
        .collect();

    let mut sorted = ids.clone();
    sorted.sort();

    assert_eq!(ids, sorted);
    assert_eq!(
        inventory.get("count").and_then(serde_json::Value::as_u64),
        Some(ids.len() as u64)
    );
    assert!(rows.iter().all(|row| row.get("kind").is_some()));
    assert!(rows.iter().all(|row| row.get("implementation").is_some()));
    assert!(rows
        .iter()
        .all(|row| row.get("workspace_outputs_ready").is_some()));
    assert!(rows.iter().all(|row| row.get("output_artifacts").is_some()));
    assert!(inventory.get("workspace_visibility").is_some());
}

#[test]
fn status_contract_runner_reports_unknown_contracts_cleanly() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let payload = run_contract(&root, Some("STATUS-CONTRACT-UNKNOWN"), None, &[]);

    assert_eq!(
        payload.get("status").and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert!(payload
        .get("error")
        .and_then(serde_json::Value::as_str)
        .is_some());
}

#[test]
fn status_contract_runner_honors_kind_filter() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let payload = run_all_contracts(&root, Some("nonexistent"), &[]);

    assert_eq!(
        payload.get("count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("ok").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("failed").and_then(serde_json::Value::as_u64),
        Some(0)
    );
}
