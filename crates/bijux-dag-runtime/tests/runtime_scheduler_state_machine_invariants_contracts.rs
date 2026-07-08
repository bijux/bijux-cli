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
    build_plan, build_scheduler, deterministic_schedule_order, recovery_action_required,
    scheduler_contract_profile, trace_time_order_ok, validate_node_transition,
    validate_run_transition, verify_post_run_state_consistency, DependencyCounter, NodeState,
    NodeTransition, ReadyNode, ReadyQueue, RecoveryInput, RunState, RunTransition, RuntimeConfig,
    SchedulerPolicy, Selector, SelectorSet, TransitionCause,
};
use std::collections::BTreeMap;
use std::time::Instant;

fn cpu_budget_graph() -> &'static str {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1},"resources":{"cpu":2,"mem_mb":64}},
        {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}],"params":{"value":2},"resources":{"cpu":2,"mem_mb":64}},
        {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3},"resources":{"cpu":1,"mem_mb":64}}
      ],
      "edges": [
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in"}}
      ]
    }"#
}

#[test]
fn scheduler_profile_contract_values_match_reported_surface() {
    let profile = scheduler_contract_profile();
    assert_eq!(format!("{:?}", profile.canonical_unit), "Node");
    assert_eq!(format!("{:?}", profile.model), "EventDriven");
    assert_eq!(format!("{:?}", profile.ready_tie_break), "PriorityCpuMemoryFitThenNodeId");
}

#[test]
fn deterministic_submission_order_is_stable_for_mixed_readiness_groups() {
    let nodes = vec![
        ReadyNode { node_id: "b".to_string(), priority: 2, attempt: 1, ready_unix_ms: 1_000 },
        ReadyNode { node_id: "a".to_string(), priority: 2, attempt: 1, ready_unix_ms: 1_000 },
        ReadyNode { node_id: "z".to_string(), priority: 1, attempt: 2, ready_unix_ms: 900 },
    ];
    let first = deterministic_schedule_order(nodes.clone(), &BTreeMap::new());
    let second = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(first, second);
    assert_eq!(
        first.into_iter().map(|n| n.node_id).collect::<Vec<_>>(),
        vec!["a".to_string(), "b".to_string(), "z".to_string()]
    );
}

#[test]
fn node_and_run_state_machines_reject_illegal_edges_explicitly() {
    let bad_node = NodeTransition {
        from: NodeState::Pending,
        to: NodeState::Running,
        cause: TransitionCause::ExecutionStarted,
    };
    assert!(validate_node_transition(&bad_node).is_err());

    let bad_run = RunTransition {
        from: RunState::Submitted,
        to: RunState::Running,
        cause: TransitionCause::ExecutionStarted,
    };
    assert!(validate_run_transition(&bad_run).is_err());
}

#[test]
fn trace_timestamps_remain_monotonic_under_high_event_volume() {
    let mut last = 0u64;
    for offset in 0..10_000u64 {
        let current = 1_000 + offset;
        assert!(trace_time_order_ok(last, current));
        last = current;
    }
}

#[test]
fn terminal_run_requires_terminal_node_presence() {
    let report = verify_post_run_state_consistency(RunState::Succeeded, &[NodeState::Running], 0);
    assert!(!report.valid);
    assert!(report.violations.iter().any(|line| line.contains("non-terminal node")));
}

#[test]
fn scheduler_emits_backpressure_when_cpu_budget_is_exceeded() {
    let graph = parse_graph_strict(cpu_budget_graph()).expect("graph");
    let mut options = RuntimeConfig::default();
    options.scheduler_policy =
        SchedulerPolicy { max_parallelism: 2, cpu_budget: Some(2), ..SchedulerPolicy::default() };
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision = scheduler.next_batch(&graph, &mut ready, &options, Instant::now(), false);
    assert_eq!(decision.batch, vec!["a".to_string()]);
    assert_eq!(decision.blocked_by_budget, vec!["b".to_string()]);
}

#[test]
fn scheduler_emits_backpressure_when_memory_budget_is_exceeded() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1},"resources":{"cpu":1,"mem_mb":1024}},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}],"params":{"value":2},"resources":{"cpu":1,"mem_mb":1024}},
            {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3},"resources":{"cpu":1,"mem_mb":256}}
          ],
          "edges": [
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let mut options = RuntimeConfig::default();
    options.jobs = 2;
    options.scheduler_policy = SchedulerPolicy {
        max_parallelism: 2,
        cpu_budget: Some(2),
        memory_budget_mb: Some(1024),
        ..SchedulerPolicy::default()
    };
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision = scheduler.next_batch(&graph, &mut ready, &options, Instant::now(), false);
    assert_eq!(decision.batch, vec!["a".to_string()]);
    assert_eq!(decision.blocked_by_budget, vec!["b".to_string()]);
    assert_eq!(decision.blocked_reasons.get("b").map(String::as_str), Some("blocked_by_memory"));
}

#[test]
fn partial_rerun_dependency_closure_keeps_required_upstream_nodes() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
            {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":3}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
          ]
        }"#,
    )
    .expect("graph");
    let options = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("c".to_string())],
            exclude: vec![],
        },
        partial_rerun_dependency_closure: true,
        ..RuntimeConfig::default()
    };
    let plan = build_plan(&graph, &options);
    assert!(plan.order.iter().any(|id| id == "a"));
    assert!(plan.order.iter().any(|id| id == "b"));
    assert!(plan.order.iter().any(|id| id == "c"));
}

#[test]
fn interrupted_runs_require_recovery_when_checkpoint_exists() {
    assert!(recovery_action_required(&RecoveryInput {
        partial_artifacts_present: false,
        terminal_state_seen: false,
        has_checkpoint: true,
    }));
}
