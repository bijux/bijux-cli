#![forbid(unsafe_code)]
//! Contract execution e2e smoke checks.

use std::path::Path;

use bijux_dev_cli::contracts::status::{run_all_contracts, run_contract};

#[test]
fn run_all_with_non_matching_kind_is_empty_and_successful() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let payload = run_all_contracts(&root, Some("non-matching-kind"), &[]);

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

#[test]
fn run_contract_with_missing_identifier_fails() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let payload = run_contract(&root, None, None, &[]);

    assert_eq!(
        payload.get("status").and_then(serde_json::Value::as_str),
        Some("failed")
    );
    assert!(payload
        .get("error")
        .and_then(serde_json::Value::as_str)
        .is_some());
}
