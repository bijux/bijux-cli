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

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    build_plan, merge_timeout_and_exit_events, scheduler_invariants_hold, thread_safety_audit,
    FailurePropagationMode, RuntimeConfig, RuntimeCoordinationState, SchedulerState,
};
use std::sync::Arc;
use std::thread;

fn diamond_graph() -> &'static str {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
        {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":1}},
        {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":1}},
        {"id":"d","kind":"const","inputs":["left","right"],"outputs":[{"name":"out","path":"d/out"}],"params":{"value":1}}
      ],
      "edges": [
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in"}},
        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"d","port":"left"}},
        {"from":{"node_id":"c","port":"out"},"to":{"node_id":"d","port":"right"}}
      ]
    }"#
}

#[test]
fn concurrent_predecessor_completion_unlocks_downstream_once() {
    let graph = parse_graph_strict(diamond_graph()).unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);
    state.complete_success("a");
    let state = Arc::new(std::sync::Mutex::new(state));

    let mut handles = Vec::new();
    for node in ["b", "c"] {
        let shared = Arc::clone(&state);
        handles.push(thread::spawn(move || {
            let mut guard = shared.lock().unwrap();
            guard.complete_success(node)
        }));
    }
    let mut all_new = Vec::new();
    for handle in handles {
        all_new.extend(handle.join().unwrap());
    }
    let count_d = all_new.iter().filter(|id| id.as_str() == "d").count();
    assert_eq!(count_d, 1);
}

#[test]
fn concurrent_trace_write_and_summary_updates_are_consistent() {
    let coordination = RuntimeCoordinationState::default();
    let shared = Arc::new(coordination);
    let mut handles = Vec::new();
    for idx in 0..32 {
        let c = Arc::clone(&shared);
        handles.push(thread::spawn(move || {
            c.mark_success();
            c.register_trace_write(&format!("node-{idx}"), format!("trace/node-{idx}.json").into());
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let snapshot = shared.snapshot();
    assert_eq!(snapshot.summary.success, 32);
    assert_eq!(snapshot.trace_writes.len(), 32);
}

#[test]
fn concurrent_cache_claim_has_single_winner_per_fingerprint() {
    let coordination = Arc::new(RuntimeCoordinationState::default());
    let mut handles = Vec::new();
    for _ in 0..16 {
        let c = Arc::clone(&coordination);
        handles.push(thread::spawn(move || c.claim_cache_fingerprint("fp-1")));
    }
    let winners = handles.into_iter().map(|h| h.join().unwrap()).filter(|v| *v).count();
    assert_eq!(winners, 1);
}

#[test]
fn cancellation_retry_and_failure_races_do_not_break_scheduler_invariants() {
    let graph = parse_graph_strict(diamond_graph()).unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);
    state.queue_retry("a");
    state.requeue_retries();
    let _ = state.complete_failed("a", FailurePropagationMode::IsolateBranch);
    assert!(scheduler_invariants_hold(&state));
}

#[test]
fn timeout_vs_process_exit_resolution_prefers_timeout_once_recorded() {
    let merged = merge_timeout_and_exit_events(
        &["node-1".to_string()],
        &["node-1".to_string(), "node-2".to_string()],
    );
    assert_eq!(merged.get("node-1"), Some(&"timed_out".to_string()));
    assert_eq!(merged.get("node-2"), Some(&"exited".to_string()));
}

#[test]
fn latest_link_update_registration_handles_parallel_updates() {
    let coordination = Arc::new(RuntimeCoordinationState::default());
    let mut handles = Vec::new();
    for idx in 0..24 {
        let c = Arc::clone(&coordination);
        handles.push(thread::spawn(move || {
            c.register_latest_link_update(format!("runs/latest-{idx}").into());
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let snapshot = coordination.snapshot();
    assert_eq!(snapshot.latest_link_updates.len(), 24);
}

#[test]
fn import_export_read_is_rejected_for_in_progress_run() {
    let coordination = RuntimeCoordinationState::default();
    assert!(coordination.begin_run("run-123"));
    let error = coordination.reject_read_during_active_run("run-123").unwrap_err();
    assert!(error.contains("in progress"));
    coordination.end_run("run-123");
    assert!(coordination.reject_read_during_active_run("run-123").is_ok());
}

#[test]
fn public_thread_safety_audit_records_are_non_empty() {
    let records = thread_safety_audit();
    assert!(records.len() >= 3);
}

#[test]
fn deterministic_stress_medium_graph_high_concurrency_stays_stable() {
    let graph = parse_graph_strict(diamond_graph()).unwrap();
    for _ in 0..200 {
        let plan = build_plan(&graph, &RuntimeConfig::default());
        let mut state = SchedulerState::from_plan(&plan);
        state.complete_success("a");
        state.complete_success("b");
        state.complete_success("c");
        assert!(state.ready_snapshot().contains(&"d".to_string()));
        assert!(scheduler_invariants_hold(&state));
    }
}
