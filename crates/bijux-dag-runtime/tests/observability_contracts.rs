use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    event_contains_sensitive_material, event_names_emitted_once, required_event_fields_present,
    validate_required_event_names, EventCategory, EventRecord, REQUIRED_RUNTIME_EVENT_NAMES,
};
use serde_json::json;

fn base_event(name: &str, ts: u128) -> EventRecord {
    EventRecord {
        category: EventCategory::Start,
        name: name.to_string(),
        unix_ms: ts,
        node_id: Some("node-a".to_string()),
        run_id: Some("run-1".to_string()),
        details: json!({"message":"ok"}),
    }
}

#[test]
fn required_runtime_event_names_are_present_for_reference_sequence() {
    let events = vec![
        base_event("run_started", 1),
        base_event("node_ready", 2),
        base_event("node_started", 3),
        base_event("node_attempt_started", 4),
        base_event("node_attempt_finished", 5),
        base_event("node_scheduled", 6),
        base_event("node_failed", 7),
        base_event("run_finished", 8),
    ];
    let missing = validate_required_event_names(&events);
    assert!(missing.is_empty(), "missing events: {missing:?}");
    assert!(event_names_emitted_once(
        &events,
        &["run_started", "run_finished"]
    ));
}

#[test]
fn required_event_fields_are_enforced() {
    let ok = base_event("run_started", 11);
    assert!(required_event_fields_present(&ok));
    let mut bad = ok.clone();
    bad.unix_ms = 0;
    assert!(!required_event_fields_present(&bad));
}

#[test]
fn observability_details_detection_flags_sensitive_material() {
    let mut event = base_event("node_started", 22);
    event.details = json!({"token":"abcd"});
    assert!(event_contains_sensitive_material(&event));
    let mut clean = base_event("node_started", 23);
    clean.details = json!({"msg":"clean"});
    assert!(!event_contains_sensitive_material(&clean));
}

#[test]
fn required_event_name_catalog_contains_core_lifecycle_events() {
    for key in ["run_started", "node_ready", "run_finished"] {
        assert!(REQUIRED_RUNTIME_EVENT_NAMES.contains(&key));
    }
}
