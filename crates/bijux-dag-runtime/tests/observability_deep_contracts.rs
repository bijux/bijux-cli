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

use bijux_dag_runtime::simulated_platform::build_investigation_bundle;
use bijux_dag_runtime::{
    build_diagnostics, build_topology_overlay, detect_metric_drift, observability_contract_status,
    redact_event_details, render_timeline_text, root_cause_graph, sample_events, EventCategory,
    EventRecord, RedactionPolicy, SamplingPolicy, TimelineEntry, TimelineExport,
};
use serde_json::json;
use std::collections::BTreeMap;

#[test]
fn diagnostics_and_root_cause_graph_are_stable() {
    let events = vec![
        EventRecord {
            category: EventCategory::Failure,
            name: "node_failed".to_string(),
            unix_ms: 1,
            node_id: Some("load".to_string()),
            run_id: Some("run-1".to_string()),
            details: json!({"upstream": "transform"}),
        },
        EventRecord {
            category: EventCategory::Timeout,
            name: "run_timeout".to_string(),
            unix_ms: 2,
            node_id: None,
            run_id: Some("run-1".to_string()),
            details: json!({}),
        },
    ];
    let diagnostics = build_diagnostics(&events);
    assert_eq!(diagnostics.len(), 2);
    let graph = root_cause_graph(&events);
    assert_eq!(graph.get("load").cloned().unwrap_or_default(), vec!["transform".to_string()]);
}

#[test]
fn timeline_and_overlay_renderers_produce_deterministic_output() {
    let timeline = TimelineExport {
        schema_version: "v0.1".to_string(),
        entries: vec![
            TimelineEntry {
                unix_ms: 2,
                category: "retry".to_string(),
                label: "node_attempt_started".to_string(),
                node_id: Some("n1".to_string()),
                status: None,
                reason: None,
                source_event: None,
            },
            TimelineEntry {
                unix_ms: 1,
                category: "schedule".to_string(),
                label: "cache_hit".to_string(),
                node_id: Some("n1".to_string()),
                status: None,
                reason: None,
                source_event: None,
            },
        ],
    };
    let text = render_timeline_text(&timeline);
    assert_eq!(text.lines.len(), 2);
    assert!(text.lines[0].starts_with("1 "));
    let overlay = build_topology_overlay(&timeline.entries);
    assert_eq!(overlay.nodes.len(), 1);
    assert_eq!(overlay.nodes[0].cache_status, "hit");
}

#[test]
fn redaction_sampling_bundle_and_drift_contracts() {
    let event = EventRecord {
        category: EventCategory::Failure,
        name: "node_failed".to_string(),
        unix_ms: 1,
        node_id: Some("n1".to_string()),
        run_id: Some("run-1".to_string()),
        details: json!({"params": {"a": 1}, "env": {"SECRET": "x"}, "token": "x"}),
    };
    let redacted = redact_event_details(
        &event,
        &RedactionPolicy {
            hide_node_params: true,
            hide_env_values: true,
            hide_artifact_metadata_keys: vec!["token".to_string()],
        },
    );
    assert!(redacted.details.get("params").is_none());
    assert!(redacted.details.get("env").is_none());
    assert!(redacted.details.get("token").is_none());

    let sampled = sample_events(
        &[event.clone(), event.clone(), event.clone(), event.clone()],
        &SamplingPolicy { enabled: true, max_spans_per_run: 10, max_events_per_run: 2 },
    );
    assert_eq!(sampled.len(), 2);

    let bundle = build_investigation_bundle("run-1");
    assert_eq!(bundle.run_id, "run-1");

    let mut current = BTreeMap::new();
    current.insert("makespan_ms".to_string(), 200.0);
    let mut baseline = BTreeMap::new();
    baseline.insert("makespan_ms".to_string(), 100.0);
    let drift = detect_metric_drift(&current, &baseline, "dag-a", "baseline-a");
    assert_eq!(drift.dag_name, "dag-a");
    assert_eq!(drift.drift_findings.len(), 1);

    let contract = observability_contract_status(&sampled, 3);
    assert!(contract.emits_events);
    assert!(contract.emits_metrics);
}
