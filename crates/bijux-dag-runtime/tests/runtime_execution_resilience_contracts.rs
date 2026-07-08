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
    build_plan, classify_failure, run_manifest_valid, verify_post_run_state_consistency,
    DependencyCounter, FailurePropagationMode, ManifestVerificationInput, NodeState, ReadyQueue,
    RunState, Runtime, RuntimeConfig, RuntimeError, SchedulerEventKind, SchedulerState,
};

fn graph_text() -> &'static str {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
        {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
        {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3}}
      ],
      "edges": [
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
      ]
    }"#
}

#[test]
fn scheduler_state_tracks_mixed_cached_skipped_retry_and_scheduled_events() {
    let graph = parse_graph_strict(graph_text()).expect("graph parse");
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);

    state.complete_cached("a");
    state.complete_skipped("b");
    state.queue_retry("c");
    state.queue_retry("c");
    state.requeue_retries();
    state.mark_scheduled("c");

    let event_kinds = state.events().iter().map(|event| event.kind.clone()).collect::<Vec<_>>();
    assert!(event_kinds.contains(&SchedulerEventKind::NodeCached));
    assert!(event_kinds.contains(&SchedulerEventKind::NodeSkipped));
    assert!(
        event_kinds.iter().filter(|kind| **kind == SchedulerEventKind::NodeRetryQueued).count()
            == 1
    );
    assert!(event_kinds.contains(&SchedulerEventKind::NodeRetryRequeued));
    assert!(event_kinds.contains(&SchedulerEventKind::NodeScheduled));
}

#[test]
fn downstream_failure_mode_controls_readiness_release() {
    let graph = parse_graph_strict(graph_text()).expect("graph parse");
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut fail_fast = SchedulerState::from_plan(&plan);
    let mut branch_isolation = SchedulerState::from_plan(&plan);

    let fail_fast_unlocked = fail_fast.complete_failed("a", FailurePropagationMode::FailFast);
    let branch_unlocked =
        branch_isolation.complete_failed("a", FailurePropagationMode::IsolateBranch);
    assert!(fail_fast_unlocked.is_empty());
    assert_eq!(branch_unlocked, vec!["b".to_string()]);
}

#[test]
fn timeout_and_exit_failure_classes_remain_distinct() {
    let timeout = classify_failure(false, false, false, true, false, false);
    let non_zero_exit = classify_failure(false, true, false, false, false, false);
    assert_ne!(format!("{timeout:?}"), format!("{non_zero_exit:?}"));
}

#[test]
fn terminal_run_rejects_non_terminal_nodes() {
    let report = verify_post_run_state_consistency(
        RunState::Succeeded,
        &[NodeState::Success, NodeState::Running],
        0,
    );
    assert!(!report.valid);
    assert!(report.violations.iter().any(|line| line.contains("non-terminal node")));
}

#[test]
fn manifest_validation_rejects_tampered_completion_state() {
    assert!(!run_manifest_valid(&ManifestVerificationInput {
        has_run_header: true,
        has_trace_index: true,
        has_outputs_index: false,
        totals_consistent: true,
    }));
    assert!(!run_manifest_valid(&ManifestVerificationInput {
        has_run_header: true,
        has_trace_index: true,
        has_outputs_index: true,
        totals_consistent: false,
    }));
}

#[test]
fn run_directory_creation_failures_surface_as_runtime_io_errors() {
    let graph = parse_graph_strict(graph_text()).expect("graph parse");
    let temp = tempfile::tempdir().expect("tempdir");
    let invalid_out_dir = temp.path().join("out-is-a-file");
    std::fs::write(&invalid_out_dir, b"not-a-directory").expect("create file");

    let runtime = Runtime::new();
    let err = runtime
        .run(&graph, &invalid_out_dir, RuntimeConfig::default())
        .expect_err("run should fail when output root is not a directory");
    let rendered = format!("{err}");
    assert!(
        matches!(err, RuntimeError::Io(_) | RuntimeError::Executor(_) | RuntimeError::Artifact(_)),
        "unexpected error class: {rendered}"
    );
}

#[test]
fn deterministic_scheduler_queue_is_stable_with_retry_requeue() {
    let graph = parse_graph_strict(graph_text()).expect("graph parse");
    let options = RuntimeConfig::default();
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());

    ready.insert("z".to_string());
    ready.insert("m".to_string());
    ready.insert("a".to_string());
    assert_eq!(ready.pop_deterministic().as_deref(), Some("a"));
    assert_eq!(ready.pop_deterministic().as_deref(), Some("m"));
}
