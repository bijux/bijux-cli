use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    build_plan, build_scheduler, DependencyCounter, LocalExecutor, ReadyQueue, RuntimeConfig,
    SchedulerPolicy, Selector, SelectorSet,
};

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
    options.scheduler_policy = SchedulerPolicy {
        max_parallelism: 2,
        cpu_budget: Some(2),
        ..SchedulerPolicy::default()
    };
    let plan = build_plan(&graph, &options);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&options.scheduler_policy);
    let decision = scheduler.next_batch(
        &graph,
        &mut ready,
        &options,
        std::time::Instant::now(),
        false,
    );
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
    assert!(!plan.filter_reasons.contains_key("a"));
    assert!(!plan.filter_reasons.contains_key("b"));
    assert!(!plan.filter_reasons.contains_key("c"));
}
