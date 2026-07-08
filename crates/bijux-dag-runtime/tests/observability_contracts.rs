use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    event_contains_sensitive_material, event_names_emitted_once, reconstruct_timeline_from_events,
    required_event_fields_present, validate_required_event_names, verify_event_log_completeness,
    EventCategory, EventRecord, TimelineExport, REQUIRED_RUNTIME_EVENT_NAMES,
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
        base_event("node_finished", 7),
        base_event("run_finished", 8),
    ];
    let missing = validate_required_event_names(&events);
    assert!(missing.is_empty(), "missing events: {missing:?}");
    assert!(event_names_emitted_once(&events, &["run_started", "run_finished"]));
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

#[test]
fn timeline_reconstruction_is_stable_from_event_log_only() {
    let events = vec![
        base_event("run_finished", 3),
        base_event("node_ready", 2),
        base_event("run_started", 1),
    ];
    let timeline = reconstruct_timeline_from_events(&events);
    assert_eq!(timeline.schema_version, "v0.1");
    assert_eq!(timeline.entries.len(), 3);
    assert_eq!(timeline.entries[0].label, "run_started");
    assert_eq!(timeline.entries[1].label, "node_ready");
    assert_eq!(timeline.entries[1].category, "ready");
    assert_eq!(timeline.entries[2].label, "run_completed");
    assert_eq!(timeline.entries[2].source_event.as_deref(), Some("run_finished"));
}

#[test]
fn timeline_reconstruction_normalizes_terminal_node_outcomes() {
    let mut failed = base_event("node_finished", 4);
    failed.category = EventCategory::Failure;
    failed.details = json!({"status":"failed","reason":"exit_code"});
    let mut cached = base_event("node_finished", 3);
    cached.category = EventCategory::CacheHit;
    cached.details = json!({"status":"cached"});
    let mut cancelled = base_event("node_skipped", 2);
    cancelled.category = EventCategory::Failure;
    cancelled.details = json!({"reason":"cancelled"});

    let timeline = reconstruct_timeline_from_events(&[failed, cached, cancelled]);
    assert_eq!(timeline.entries[0].label, "node_cancelled");
    assert_eq!(timeline.entries[0].category, "cancel");
    assert_eq!(timeline.entries[1].label, "node_cached");
    assert_eq!(timeline.entries[1].status.as_deref(), Some("cached"));
    assert_eq!(timeline.entries[2].label, "node_failed");
    assert_eq!(timeline.entries[2].reason.as_deref(), Some("exit_code"));
}

#[test]
fn completeness_verifier_accepts_monotonic_reconstructible_event_log() {
    let events = vec![
        base_event("run_started", 1),
        base_event("node_ready", 2),
        base_event("node_started", 3),
        base_event("node_attempt_started", 4),
        base_event("node_attempt_finished", 5),
        base_event("node_scheduled", 6),
        base_event("node_finished", 7),
        base_event("run_finished", 8),
    ];
    let timeline = reconstruct_timeline_from_events(&events);
    let report = verify_event_log_completeness(&events, Some(&timeline));
    assert!(report.complete);
    assert!(report.required_names_present);
    assert!(report.required_event_field_gaps.is_empty());
    assert!(report.missing_required_names.is_empty());
    assert!(report.monotonic_timestamps);
    assert!(report.timeline_matches_reconstruction);
}

#[test]
fn completeness_verifier_flags_missing_names_and_timeline_drift() {
    let events = vec![base_event("run_started", 2), base_event("run_finished", 1)];
    let mismatched_timeline =
        TimelineExport { schema_version: "v0.1".to_string(), entries: vec![] };
    let report = verify_event_log_completeness(&events, Some(&mismatched_timeline));
    assert!(!report.complete);
    assert!(!report.required_names_present);
    assert!(report.missing_required_names.iter().any(|name| name == "node_ready"));
    assert!(!report.monotonic_timestamps);
    assert!(!report.timeline_matches_reconstruction);
}
