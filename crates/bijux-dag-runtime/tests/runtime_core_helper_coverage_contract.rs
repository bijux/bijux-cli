use bijux_dag_artifacts as _;
use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    apply_backfill_throttling, build_planner_analysis, cache_entry_valid,
    deterministic_schedule_order, PlannerGuardrails, ReadyNode, RetryPolicySemantics,
    RuntimeConfig, SelectorSet,
};
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeMap;
use tempfile as _;
use thiserror as _;

#[test]
fn scheduler_workload_helpers_are_exercised_for_coverage_gate() {
    let (allowed_backfill, live) = apply_backfill_throttling(
        12,
        20,
        &bijux_dag_runtime::BackfillThrottlingPolicy {
            max_backfill_submissions_per_tick: 10,
            reserve_live_capacity_percent: 20,
        },
    );
    assert!(allowed_backfill <= 10);
    assert_eq!(live, 20);

    let ordered = deterministic_schedule_order(
        vec![
            ReadyNode { node_id: "a".to_string(), priority: 1, attempt: 1, ready_unix_ms: 20 },
            ReadyNode { node_id: "b".to_string(), priority: 3, attempt: 1, ready_unix_ms: 10 },
        ],
        &BTreeMap::new(),
    );
    assert_eq!(ordered.len(), 2);
    assert_eq!(ordered[0].node_id, "b");

    assert!(bijux_dag_runtime::retry_allowed(
        1,
        &RetryPolicySemantics { max_attempts: 3, initial_backoff_ms: 100, exponential: true },
    ));
    assert!(cache_entry_valid(&bijux_dag_runtime::CacheValidationInput {
        fingerprint_matches: true,
        schema_matches: true,
        proof_present: true,
    }));
}

#[test]
fn planner_analysis_helpers_are_exercised_for_coverage_gate() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"const","inputs":[],"outputs":[{"name":"out","path":"b/out"}],"params":{"value":2}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph parse");

    let built = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build");

    let explain = bijux_dag_runtime::explain_plan(&built);
    assert!(!explain.phases.is_empty());
    assert!(!built.plan_fingerprint.is_empty());
}
