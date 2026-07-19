use bijux_dag_runtime::prelude::{
    build_plan, build_planner_analysis, build_scheduler, PlannerGuardrails, Runtime, RuntimeConfig,
    SchedulerPolicy, SelectorSet,
};
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

fn simple_graph() -> bijux_dag_core::Graph {
    bijux_dag_core::parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"seed","kind":"const","outputs":[{"name":"out","path":"seed/out"}],"params":{"value":"ok"}}],
          "edges":[]
        }"#,
    )
    .expect("graph")
}

#[test]
fn prelude_covers_plan_scheduler_and_runtime_construction() {
    let graph = simple_graph();
    let config = RuntimeConfig::default();
    let plan = build_plan(&graph, &config);
    assert_eq!(plan.order.len(), 1);

    let analysis = build_planner_analysis(
        &graph,
        &config,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("analysis");
    assert_eq!(analysis.plan.order.len(), 1);

    let scheduler = build_scheduler(&SchedulerPolicy::default());
    let _ = scheduler;

    let runtime = Runtime::new();
    let _ = runtime;
}
