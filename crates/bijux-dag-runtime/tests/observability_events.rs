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
    category_from_runtime_event_name, summarize_failure_root_causes, EventCategory, EventRecord,
};
use serde_json::json;

#[test]
fn maps_major_runtime_events_to_stable_categories() {
    assert_eq!(category_from_runtime_event_name("plan_built"), EventCategory::Plan);
    assert_eq!(category_from_runtime_event_name("node_scheduled"), EventCategory::Schedule);
    assert_eq!(category_from_runtime_event_name("node_started"), EventCategory::Start);
    assert_eq!(category_from_runtime_event_name("node_attempt_started"), EventCategory::Retry);
    assert_eq!(category_from_runtime_event_name("node_retry_scheduled"), EventCategory::Retry);
    assert_eq!(category_from_runtime_event_name("run_timeout"), EventCategory::Timeout);
    assert_eq!(category_from_runtime_event_name("cache_hit"), EventCategory::CacheHit);
    assert_eq!(category_from_runtime_event_name("policy_denied"), EventCategory::Failure);
}

#[test]
fn summarizes_root_causes_from_failure_events() {
    let events = vec![
        EventRecord {
            category: EventCategory::Failure,
            name: "node_failed".to_string(),
            unix_ms: 1,
            node_id: Some("extract".to_string()),
            run_id: Some("run-1".to_string()),
            details: json!({"reason": "timeout"}),
        },
        EventRecord {
            category: EventCategory::Failure,
            name: "node_failed".to_string(),
            unix_ms: 2,
            node_id: Some("load".to_string()),
            run_id: Some("run-1".to_string()),
            details: json!({"reason": "dependency_failed"}),
        },
    ];
    let roots = summarize_failure_root_causes(&events);
    assert_eq!(roots, vec!["extract:timeout", "load:dependency_failed"]);
}
