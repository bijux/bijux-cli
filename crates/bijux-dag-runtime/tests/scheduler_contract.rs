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
    build_plan, build_scheduler, failure_allows_downstream_readiness, replay_scheduler_checkpoint,
    scheduler_contract_profile, scheduler_debug_event_log, scheduler_invariant_violations,
    scheduler_invariants_hold, DependencyCounter, ExecutionCheckpoint, FailurePropagationMode,
    LocalExecutor, ReadyQueue, RuntimeConfig, SchedulerPolicy, SchedulerState, Selector,
    SelectorSet,
};
use std::collections::BTreeMap;

fn graph_text() -> &'static str {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
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
fn deterministic_scheduler_preserves_stable_dispatch_order() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let mut options = RuntimeConfig::default();
    options.scheduler_policy =
        SchedulerPolicy { max_parallelism: 2, cpu_budget: Some(2), ..SchedulerPolicy::default() };
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert_eq!(decision.batch, vec!["a".to_string()]);
}

#[test]
fn local_executor_enforces_bounded_queue_capacity() {
    let mut executor = LocalExecutor::new(2);
    executor.submit("a".to_string()).unwrap();
    executor.submit("b".to_string()).unwrap();
    let error = executor.submit("c".to_string()).unwrap_err();
    assert!(error.contains("queue is full"));
}

#[test]
fn planner_dependency_closure_keeps_upstream_for_partial_rerun() {
    let graph = parse_graph_strict(graph_text()).unwrap();
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
fn scheduler_contract_profile_is_explicit_and_stable() {
    let profile = scheduler_contract_profile();
    assert_eq!(format!("{:?}", profile.canonical_unit), "Node");
    assert_eq!(format!("{:?}", profile.model), "EventDriven");
    assert_eq!(format!("{:?}", profile.priority_model), "StaticHints");
    assert_eq!(format!("{:?}", profile.ready_tie_break), "PriorityCpuMemoryFitThenNodeId");
}

#[test]
fn ready_queue_evolution_is_deterministic_for_fixed_event_sequence() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let options = RuntimeConfig::default();
    let plan = build_plan(&graph, &options);
    let mut state = SchedulerState::from_plan(&plan);

    assert_eq!(state.ready_snapshot(), vec!["a".to_string()]);
    let newly_ready = state.complete_success("a");
    assert_eq!(newly_ready, vec!["b".to_string()]);
    assert_eq!(state.ready_snapshot(), vec!["b".to_string()]);
    state.complete_success("b");
    assert!(state.ready_snapshot().contains(&"c".to_string()));
    assert!(scheduler_invariants_hold(&state));
}

#[test]
fn retry_requeue_preserves_readiness_accounting() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let options = RuntimeConfig::default();
    let plan = build_plan(&graph, &options);
    let mut state = SchedulerState::from_plan(&plan);

    state.queue_retry("a");
    assert_eq!(state.retry_snapshot(), vec!["a".to_string()]);
    state.requeue_retries();
    assert!(state.retry_snapshot().is_empty());
    assert!(state.ready_snapshot().contains(&"a".to_string()));
    assert!(scheduler_invariants_hold(&state));
}

#[test]
fn downstream_node_becomes_ready_exactly_once_with_two_predecessors() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}],"params":{"value":1}},
            {"id":"c","kind":"const","inputs":["in1","in2"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":1}}
          ],
          "edges": [
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in1"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in2"}}
          ]
        }"#,
    )
    .unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);

    let mut ready_hits = 0usize;
    for node in ["a", "b"] {
        let new_nodes = state.complete_success(node);
        if new_nodes.iter().any(|v| v == "c") {
            ready_hits += 1;
        }
    }
    assert_eq!(ready_hits, 1);
}

#[test]
fn cached_and_skipped_predecessors_satisfy_readiness_semantics() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);

    let from_cache = state.complete_cached("a");
    assert_eq!(from_cache, vec!["b".to_string()]);
    let from_skip = state.complete_skipped("b");
    assert_eq!(from_skip, vec!["c".to_string()]);
}

#[test]
fn failure_propagation_modes_drive_downstream_readiness_policy() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());

    let mut fail_fast = SchedulerState::from_plan(&plan);
    let unlocked = fail_fast.complete_failed("a", FailurePropagationMode::FailFast);
    assert!(unlocked.is_empty());
    assert!(!failure_allows_downstream_readiness(FailurePropagationMode::FailFast));

    let mut isolate = SchedulerState::from_plan(&plan);
    let unlocked_isolate = isolate.complete_failed("a", FailurePropagationMode::IsolateBranch);
    assert_eq!(unlocked_isolate, vec!["b".to_string()]);
    assert!(failure_allows_downstream_readiness(FailurePropagationMode::IsolateBranch));
}

#[test]
fn changing_concurrency_budget_does_not_change_node_set_semantics() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let mut low = RuntimeConfig::default();
    low.jobs = 1;
    low.scheduler_policy.max_parallelism = 1;
    let mut high = RuntimeConfig::default();
    high.jobs = 16;
    high.scheduler_policy.max_parallelism = 16;

    let plan_low = build_plan(&graph, &low);
    let plan_high = build_plan(&graph, &high);
    let nodes_low =
        plan_low.nodes.iter().map(|n| n.id.clone()).collect::<std::collections::BTreeSet<_>>();
    let nodes_high =
        plan_high.nodes.iter().map(|n| n.id.clone()).collect::<std::collections::BTreeSet<_>>();
    assert_eq!(nodes_low, nodes_high);
}

#[test]
fn cancellation_prevents_new_scheduling_batches() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let options = RuntimeConfig::default();
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), true);
    assert!(decision.batch.is_empty());
    assert!(decision.cancelled);
}

#[test]
fn deterministic_scheduler_prefers_critical_work_over_standard_work() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"standard","kind":"const","outputs":[{"name":"out","path":"standard/out"}],"params":{"value":1}},
            {"id":"critical","kind":"const","outputs":[{"name":"out","path":"critical/out"}],"tags":["critical"],"params":{"value":2}}
          ],
          "edges": []
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.scheduler_policy.max_parallelism = 1;
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert_eq!(decision.batch, vec!["critical".to_string()]);
}

#[test]
fn deterministic_scheduler_packs_smaller_ready_nodes_within_cpu_budget() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"big","kind":"const","outputs":[{"name":"out","path":"big/out"}],"resources":{"cpu":3,"mem_mb":64},"params":{"value":1}},
            {"id":"small-a","kind":"const","outputs":[{"name":"out","path":"small-a/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":2}},
            {"id":"small-b","kind":"const","outputs":[{"name":"out","path":"small-b/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":3}}
          ],
          "edges": []
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.jobs = 3;
    options.scheduler_policy.max_parallelism = 3;
    options.scheduler_policy.cpu_budget = Some(2);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert_eq!(decision.batch, vec!["small-a".to_string(), "small-b".to_string()]);
    assert!(decision.blocked_by_budget.contains(&"big".to_string()));
    assert_eq!(decision.blocked_reasons.get("big").map(String::as_str), Some("blocked_by_cpu"));
    assert_eq!(decision.tie_break_reason.as_deref(), Some("priority_cpu_memory_fit_then_node_id"));
    assert_eq!(
        decision.ready_candidates,
        vec!["small-a".to_string(), "small-b".to_string(), "big".to_string()]
    );
}

#[test]
fn deterministic_scheduler_packs_smaller_ready_nodes_within_memory_budget() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"memory-heavy","kind":"const","outputs":[{"name":"out","path":"memory-heavy/out"}],"resources":{"cpu":1,"mem_mb":2048},"params":{"value":1}},
            {"id":"memory-light-a","kind":"const","outputs":[{"name":"out","path":"memory-light-a/out"}],"resources":{"cpu":1,"mem_mb":512},"params":{"value":2}},
            {"id":"memory-light-b","kind":"const","outputs":[{"name":"out","path":"memory-light-b/out"}],"resources":{"cpu":1,"mem_mb":512},"params":{"value":3}}
          ],
          "edges": []
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.jobs = 3;
    options.scheduler_policy.max_parallelism = 3;
    options.scheduler_policy.cpu_budget = Some(3);
    options.scheduler_policy.memory_budget_mb = Some(1024);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert_eq!(decision.batch, vec!["memory-light-a".to_string(), "memory-light-b".to_string()]);
    assert!(decision.blocked_by_budget.contains(&"memory-heavy".to_string()));
    assert_eq!(
        decision.blocked_reasons.get("memory-heavy").map(String::as_str),
        Some("blocked_by_memory")
    );
    assert_eq!(decision.tie_break_reason.as_deref(), Some("priority_cpu_memory_fit_then_node_id"));
    assert_eq!(
        decision.ready_candidates,
        vec![
            "memory-light-a".to_string(),
            "memory-light-b".to_string(),
            "memory-heavy".to_string()
        ]
    );
}

#[test]
fn deterministic_scheduler_respects_gpu_device_budget() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"gpu-a","kind":"const","tags":["gpu"],"outputs":[{"name":"out","path":"gpu-a/out"}],"params":{"value":1}},
            {"id":"gpu-b","kind":"const","resources":{"cpu":1,"mem_mb":64,"gpu_devices":1},"outputs":[{"name":"out","path":"gpu-b/out"}],"params":{"value":2}},
            {"id":"cpu","kind":"const","outputs":[{"name":"out","path":"cpu/out"}],"params":{"value":3}}
          ],
          "edges": []
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.jobs = 3;
    options.scheduler_policy.max_parallelism = 3;
    options.scheduler_policy.cpu_budget = Some(3);
    options.scheduler_policy.gpu_device_budget = Some(1);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    let scheduled_gpu_nodes = decision
        .batch
        .iter()
        .filter(|node_id| node_id.starts_with("gpu-"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(scheduled_gpu_nodes.len(), 1);
    assert!(decision.batch.contains(&"cpu".to_string()));
    let blocked_gpu_nodes = decision
        .blocked_by_budget
        .iter()
        .filter(|node_id| node_id.starts_with("gpu-"))
        .cloned()
        .collect::<Vec<_>>();
    assert_eq!(blocked_gpu_nodes.len(), 1);
    let blocked_gpu = &blocked_gpu_nodes[0];
    assert_eq!(
        decision.blocked_reasons.get(blocked_gpu).map(String::as_str),
        Some("blocked_by_gpu")
    );
}

#[test]
fn deterministic_scheduler_respects_named_resource_capacities() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}},"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}},"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}},
            {"id":"cpu","kind":"const","outputs":[{"name":"out","path":"cpu/out"}],"params":{"value":3}}
          ],
          "edges":[]
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.jobs = 3;
    options.scheduler_policy.max_parallelism = 3;
    options.scheduler_policy.cpu_budget = Some(3);
    options.named_resource_capacities =
        std::collections::BTreeMap::from([("database_slot".to_string(), 1)]);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert!(decision.batch.contains(&"a".to_string()));
    assert!(decision.batch.contains(&"cpu".to_string()));
    assert!(!decision.batch.contains(&"b".to_string()));
    assert_eq!(
        decision.blocked_reasons.get("b").map(String::as_str),
        Some("blocked_by_named_resource:database_slot")
    );
}

#[test]
fn deterministic_scheduler_forces_single_progress_for_oversized_root() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"huge","kind":"const","outputs":[{"name":"out","path":"huge/out"}],"resources":{"cpu":9,"mem_mb":64},"params":{"value":1}}
          ],
          "edges": []
        }"#,
    )
    .unwrap();
    let mut options = RuntimeConfig::default();
    options.jobs = 1;
    options.scheduler_policy.max_parallelism = 1;
    options.scheduler_policy.cpu_budget = Some(2);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision =
        scheduler.next_batch(&graph, &mut ready, &options, std::time::Instant::now(), false);
    assert_eq!(decision.batch, vec!["huge".to_string()]);
    assert_eq!(decision.decision_reason, "forced_single_progress");
}

#[test]
fn checkpoint_replay_reconstructs_ready_and_completed_state() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let options = RuntimeConfig::default();
    let plan = build_plan(&graph, &options);
    let checkpoint = ExecutionCheckpoint {
        loop_index: 3,
        ready_queue_depth: 1,
        ready_queue: vec!["c".to_string()],
        inflight: vec!["b".to_string()],
        scheduled: vec!["b".to_string()],
        blocked_by_budget: Vec::new(),
        blocked_reasons: BTreeMap::new(),
        completed_statuses: BTreeMap::from([
            ("a".to_string(), "success".to_string()),
            ("b".to_string(), "cached".to_string()),
        ]),
        failure_propagation_mode: "isolate_branch".to_string(),
        dependency_closure_enabled: true,
        generated_unix_ms: 42,
    };
    let state = replay_scheduler_checkpoint(&plan, &checkpoint).expect("replay");
    assert_eq!(state.ready_snapshot(), vec!["c".to_string()]);
    assert!(scheduler_invariant_violations(&state).is_empty());
}

#[test]
fn timeout_is_distinct_from_failure_path_in_scheduler_decision() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let mut options = RuntimeConfig::default();
    options.run_timeout_ms = Some(0);
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let started = std::time::Instant::now() - std::time::Duration::from_millis(1);
    let decision = scheduler.next_batch(&graph, &mut ready, &options, started, false);
    assert!(decision.timed_out);
    assert!(!decision.cancelled);
    assert!(decision.batch.is_empty());
}

#[test]
fn simultaneous_predecessor_completions_do_not_duplicate_enqueue() {
    let graph = parse_graph_strict(
        r#"{
          "spec": "bijux-dag/v0.1",
          "nodes": [
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}],"params":{"value":1}},
            {"id":"c","kind":"const","inputs":["in1","in2"],"outputs":[{"name":"out","path":"c/out"}],"params":{"value":1}}
          ],
          "edges": [
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in1"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in2"}}
          ]
        }"#,
    )
    .unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);
    let _ = state.complete_success("a");
    let _ = state.complete_success("b");
    let ready_c_count = state.ready_snapshot().into_iter().filter(|n| n == "c").count();
    assert_eq!(ready_c_count, 1);
}

#[test]
fn scheduler_debug_event_log_is_timeline_reconstructable() {
    let graph = parse_graph_strict(graph_text()).unwrap();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let mut state = SchedulerState::from_plan(&plan);
    state.complete_success("a");
    state.complete_cached("b");
    let events = scheduler_debug_event_log(&state);
    assert!(!events.is_empty());
    let mut last = 0;
    for event in events {
        assert!(event.sequence > last);
        last = event.sequence;
    }
}
