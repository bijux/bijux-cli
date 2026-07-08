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
    build_backfill_plan, build_planner_analysis, build_replay_plan_annotations,
    compare_plan_equivalence, compute_partial_run_closure, diff_plans, explain_plan,
    PlannerEquivalenceClass, PlannerGuardrails, RuntimeConfig, Selector, SelectorSet,
};

fn sample_graph() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
        {"id":"b","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}],"tags":["noop"],"params":{"argv":["echo","b"]}},
        {"id":"c","kind":"shell","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}],"params":{"argv":["echo","c"]}}
      ],
      "edges":[
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
      ]
    }"#
}

fn execution_cost_graph() -> &'static str {
    r#"{
      "spec":"bijux-dag/v0.1",
      "nodes":[
        {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
        {
          "id":"b",
          "kind":"shell",
          "outputs":[{"name":"out","path":"b/out"}],
          "params":{"argv":["echo","b"],"estimated_duration_ms":9000},
          "resources":{"cpu":4,"mem_mb":2048},
          "tags":["gpu:2"],
          "timeout_ms":5000,
          "retry":{"max_attempts":3,"backoff_ms":250},
          "cache":{"enabled":false,"reason":"network-bound"}
        },
        {
          "id":"c",
          "kind":"shell",
          "inputs":["left","right"],
          "outputs":[{"name":"out","path":"c/out"}],
          "params":{"argv":["echo","c"],"estimated_duration_ms":3000},
          "resources":{"cpu":2,"mem_mb":1024}
        }
      ],
      "edges":[
        {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"left"}},
        {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"right"}}
      ]
    }"#
}

#[test]
fn planner_builds_phased_result_and_fingerprint() {
    let graph = parse_graph_strict(sample_graph()).expect("graph should parse");
    let options = RuntimeConfig::default();
    let result = build_planner_analysis(
        &graph,
        &options,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");
    assert!(!result.phases.is_empty());
    assert!(!result.plan_fingerprint.is_empty());
    assert!(!result.annotations.is_empty());
}

#[test]
fn planner_supports_closure_replay_backfill_diff_and_explain() {
    let graph = parse_graph_strict(sample_graph()).expect("graph should parse");
    let options = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("c".to_string())],
            exclude: vec![],
        },
        ..RuntimeConfig::default()
    };
    let before = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: false },
    )
    .expect("before planner should succeed");
    let after = build_planner_analysis(
        &graph,
        &options,
        &options.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after planner should succeed");
    let closure = compute_partial_run_closure(&after.plan, &["c".to_string()]);
    assert!(closure.contains("a"));
    let replay = build_replay_plan_annotations(&after.plan);
    assert!(!replay.is_empty());
    let backfill = build_backfill_plan(1, 10, vec!["p0".to_string(), "p1".to_string()]);
    assert_eq!(backfill.partition_keys.len(), 2);
    let diff = diff_plans(&before, &after);
    assert!(
        !diff.graph_fingerprint_changed
            || diff.execution_affecting_changed
            || diff.metadata_only_changed
    );
    let explain = explain_plan(&after);
    assert!(!explain.phases.is_empty());
}

#[test]
fn planner_execution_cost_estimate_reports_topology_demand_and_exposure() {
    let graph = parse_graph_strict(execution_cost_graph()).expect("graph should parse");
    let options = RuntimeConfig::default();
    let result = build_planner_analysis(
        &graph,
        &options,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let estimate = result.execution_cost_estimate;
    assert_eq!(estimate.node_count, 3);
    assert_eq!(estimate.root_nodes, vec!["a".to_string(), "b".to_string()]);
    assert_eq!(estimate.critical_path_length, 2);
    assert_eq!(estimate.critical_path.node_ids, vec!["b".to_string(), "c".to_string()]);
    assert_eq!(estimate.critical_path.total_duration_ms, 12_000);
    assert_eq!(estimate.critical_path.estimated_duration_nodes, 2);
    assert_eq!(estimate.critical_path.unit_duration_fallback_nodes, 0);
    assert_eq!(estimate.max_parallelism, 2);
    assert_eq!(estimate.demand.cpu_cores_total, 7);
    assert_eq!(estimate.demand.memory_mb_total, 3328);
    assert_eq!(estimate.demand.gpu_devices_total, 2);
    assert_eq!(estimate.demand.cpu_cores_peak_parallel, 5);
    assert_eq!(estimate.demand.memory_mb_peak_parallel, 2304);
    assert_eq!(estimate.demand.gpu_devices_peak_parallel, 2);
    assert_eq!(estimate.cache_exposure.cacheable_nodes, 2);
    assert_eq!(estimate.cache_exposure.non_cacheable_nodes, 1);
    assert_eq!(estimate.cache_exposure.non_cacheable_node_ids, vec!["b".to_string()]);
    assert_eq!(estimate.timeout_exposure.timed_nodes, 1);
    assert_eq!(estimate.timeout_exposure.timed_node_ids, vec!["b".to_string()]);
    assert_eq!(estimate.timeout_exposure.max_timeout_ms, Some(5_000));
    assert_eq!(estimate.timeout_exposure.total_timeout_ms, 5_000);
    assert_eq!(estimate.retry_exposure.retrying_nodes, 1);
    assert_eq!(estimate.retry_exposure.retrying_node_ids, vec!["b".to_string()]);
    assert_eq!(estimate.retry_exposure.max_attempts, 3);
    assert_eq!(estimate.retry_exposure.max_backoff_ms, 250);
    assert_eq!(estimate.retry_exposure.total_retry_attempts, 3);
}

#[test]
fn planner_execution_cost_estimate_tracks_partial_selection() {
    let graph = parse_graph_strict(execution_cost_graph()).expect("graph should parse");
    let options = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::Id("c".to_string())],
            exclude: Vec::new(),
        },
        partial_rerun_dependency_closure: false,
        ..RuntimeConfig::default()
    };
    let result = build_planner_analysis(
        &graph,
        &options,
        &options.selectors,
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let estimate = result.execution_cost_estimate;
    assert_eq!(estimate.node_count, 1);
    assert_eq!(estimate.root_nodes, vec!["c".to_string()]);
    assert_eq!(estimate.critical_path_length, 1);
    assert_eq!(estimate.critical_path.node_ids, vec!["c".to_string()]);
    assert_eq!(estimate.critical_path.total_duration_ms, 3_000);
    assert_eq!(estimate.critical_path.estimated_duration_nodes, 1);
    assert_eq!(estimate.critical_path.unit_duration_fallback_nodes, 0);
    assert_eq!(estimate.max_parallelism, 1);
    assert_eq!(estimate.demand.cpu_cores_total, 2);
    assert_eq!(estimate.demand.memory_mb_total, 1024);
    assert_eq!(estimate.cache_exposure.cacheable_nodes, 1);
    assert_eq!(estimate.cache_exposure.non_cacheable_nodes, 0);
    assert_eq!(estimate.timeout_exposure.timed_nodes, 0);
    assert_eq!(estimate.retry_exposure.retrying_nodes, 0);
}

#[test]
fn planner_critical_path_uses_unit_duration_fallback_when_estimates_are_missing() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"shell","outputs":[{"name":"out","path":"b/out"}],"params":{"argv":["echo","b"]}},
            {"id":"c","kind":"shell","inputs":["left","right"],"outputs":[{"name":"out","path":"c/out"}],"params":{"argv":["echo","c"]}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"left"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"right"}}
          ]
        }"#,
    )
    .expect("graph should parse");

    let options = RuntimeConfig {
        jobs: 2,
        scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
            max_parallelism: 2,
            ..bijux_dag_runtime::SchedulerPolicy::default()
        },
        ..RuntimeConfig::default()
    };
    let result = build_planner_analysis(
        &graph,
        &options,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let estimate = result.execution_cost_estimate;
    assert_eq!(estimate.critical_path_length, 2);
    assert_eq!(estimate.critical_path.node_ids, vec!["a".to_string(), "c".to_string()]);
    assert_eq!(estimate.critical_path.total_duration_ms, 2);
    assert_eq!(estimate.critical_path.estimated_duration_nodes, 0);
    assert_eq!(estimate.critical_path.unit_duration_fallback_nodes, 2);
    assert_eq!(
        estimate.scheduling_simulation.run_bound,
        bijux_dag_runtime::PlannerSchedulingBound::DependencyBound
    );
}

#[test]
fn planner_scheduling_simulation_reports_named_resource_bottlenecks() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"root","kind":"const","outputs":[{"name":"out","path":"root/out"}],"params":{"value":1}},
            {
              "id":"left",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"left/out"}],
              "params":{"argv":["echo","left"],"estimated_duration_ms":10000},
              "resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}}
            },
            {
              "id":"right",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"right/out"}],
              "params":{"argv":["echo","right"],"estimated_duration_ms":10000},
              "resources":{"cpu":1,"mem_mb":64,"named_resources":{"database_slot":1}}
            },
            {
              "id":"join",
              "kind":"shell",
              "inputs":["left","right"],
              "outputs":[{"name":"out","path":"join/out"}],
              "params":{"argv":["echo","join"]}
            }
          ],
          "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"left","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"right","port":"in"}},
            {"from":{"node_id":"left","port":"out"},"to":{"node_id":"join","port":"left"}},
            {"from":{"node_id":"right","port":"out"},"to":{"node_id":"join","port":"right"}}
          ]
        }"#,
    )
    .expect("graph should parse");
    let options = RuntimeConfig {
        jobs: 2,
        named_resource_capacities: std::collections::BTreeMap::from([(
            "database_slot".to_string(),
            1,
        )]),
        scheduler_policy: bijux_dag_runtime::SchedulerPolicy {
            max_parallelism: 2,
            ..bijux_dag_runtime::SchedulerPolicy::default()
        },
        ..RuntimeConfig::default()
    };
    let result = build_planner_analysis(
        &graph,
        &options,
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let estimate = result.execution_cost_estimate;
    assert_eq!(
        estimate.demand.named_resources_total,
        std::collections::BTreeMap::from([("database_slot".to_string(), 2)])
    );
    assert_eq!(
        estimate.demand.named_resources_peak_parallel,
        std::collections::BTreeMap::from([("database_slot".to_string(), 2)])
    );
    assert!(estimate.critical_path.total_duration_ms >= 10000);
    assert_eq!(estimate.scheduling_simulation.scheduled_waves, 4);
    assert!(estimate.scheduling_simulation.projected_makespan_ms >= 20000);
    assert_eq!(
        estimate.scheduling_simulation.resource_delay_ms,
        estimate
            .scheduling_simulation
            .projected_makespan_ms
            .saturating_sub(estimate.critical_path.total_duration_ms)
    );
    assert_eq!(
        estimate.scheduling_simulation.run_bound,
        bijux_dag_runtime::PlannerSchedulingBound::ResourceBound
    );
    assert_eq!(
        estimate.scheduling_simulation.bottlenecks,
        vec![bijux_dag_runtime::PlannerResourceBottleneck {
            resource: "named_resource:database_slot".to_string(),
            blocking_events: 1,
            blocked_node_ids: vec!["right".to_string()],
            blocked_duration_ms: 10000,
        }]
    );
    assert_eq!(
        estimate.scheduling_simulation.blocked_nodes,
        vec![bijux_dag_runtime::PlannerBlockedNodeEstimate {
            node_id: "right".to_string(),
            blocked_by: vec!["blocked_by_named_resource:database_slot".to_string()],
            blocked_waves: 1,
            blocked_duration_ms: 10000,
        }]
    );
}

#[test]
fn planner_diff_detects_added_removed_and_execution_affecting_changes() {
    let before = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"before","owners":[],"tags":[]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","before"]},
              "resources":{"cpu":1,"mem_mb":64},
              "timeout_ms":1000,
              "retry":{"max_attempts":1,"backoff_ms":10}
            }
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("before graph should parse");
    let after = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"after","owners":[],"tags":[]},
          "nodes":[
            {"id":"c","kind":"const","outputs":[{"name":"out","path":"c/out"}],"params":{"value":2}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"result","path":"b/result.json"}],
              "params":{"argv":["echo","after"]},
              "resources":{"cpu":4,"mem_mb":256},
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":50}
            }
          ],
          "edges":[{"from":{"node_id":"c","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("after graph should parse");

    let before_result = build_planner_analysis(
        &before,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("before analysis should succeed");
    let after_result = build_planner_analysis(
        &after,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after analysis should succeed");

    let diff = diff_plans(&before_result, &after_result);
    assert!(diff.graph_fingerprint_changed);
    assert!(diff.execution_affecting_changed);
    assert!(!diff.metadata_only_changed);
    assert_eq!(diff.added_nodes, vec!["c".to_string()]);
    assert_eq!(diff.removed_nodes, vec!["a".to_string()]);
    assert_eq!(diff.changed_params, vec!["b".to_string()]);
    assert_eq!(diff.changed_outputs, vec!["b".to_string()]);
    assert_eq!(diff.changed_resources, vec!["b".to_string()]);
    assert_eq!(diff.changed_retry_timeout, vec!["b".to_string()]);
    assert_eq!(diff.added_dependencies, vec!["data:-:-:c:out->b:in".to_string()]);
    assert_eq!(diff.removed_dependencies, vec!["data:-:-:a:out->b:in".to_string()]);
}

#[test]
fn planner_diff_classifies_graph_meta_drift_as_metadata_only() {
    let before = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"baseline","description":"before","owners":["ops"],"tags":["stable"]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect("before graph should parse");
    let after = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"baseline","description":"after","owners":["platform"],"tags":["reviewed"]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect("after graph should parse");

    let before_result = build_planner_analysis(
        &before,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("before analysis should succeed");
    let after_result = build_planner_analysis(
        &after,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after analysis should succeed");

    let diff = diff_plans(&before_result, &after_result);
    assert!(diff.graph_fingerprint_changed);
    assert!(!diff.execution_fingerprint_changed);
    assert!(diff.metadata_only_changed);
    assert!(!diff.execution_affecting_changed);
    assert_eq!(diff.changed_metadata, vec!["graph_meta".to_string()]);
    assert!(diff.added_nodes.is_empty());
    assert!(diff.changed_params.is_empty());
}

#[test]
fn plan_equivalence_ignores_non_execution_metadata_drift() {
    let before = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"baseline","description":"before","owners":["ops"],"tags":["stable"]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1},"tags":["alpha"],"group":"ops"}
          ],
          "edges":[]
        }"#,
    )
    .expect("before graph should parse");
    let after = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "meta":{"name":"baseline","description":"after","owners":["platform"],"tags":["reviewed"]},
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1},"tags":["beta"],"group":"platform"}
          ],
          "edges":[]
        }"#,
    )
    .expect("after graph should parse");

    let before_result = build_planner_analysis(
        &before,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("before analysis should succeed");
    let after_result = build_planner_analysis(
        &after,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after analysis should succeed");

    let report = compare_plan_equivalence(&before_result, &after_result);
    assert!(report.equivalent);
    assert_eq!(report.equivalence_class, PlannerEquivalenceClass::MetadataDriftEquivalent);
    assert!(!report.graph_identity_equal);
    assert!(report.execution_fingerprint_equal);
    assert_eq!(
        report.ignored_non_execution_drift,
        vec!["graph_meta".to_string(), "node_group:a".to_string(), "node_tags:a".to_string()]
    );
    assert!(report.non_equivalence_causes.is_empty());
}

#[test]
fn plan_equivalence_reports_exact_non_equivalence_causes() {
    let before = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","before"]},
              "resources":{"cpu":1,"mem_mb":64},
              "timeout_ms":1000,
              "retry":{"max_attempts":1,"backoff_ms":10},
              "effects":["filesystem"]
            }
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("before graph should parse");
    let after = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"c","kind":"const","outputs":[{"name":"out","path":"c/out"}],"params":{"value":2}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"result","path":"b/result.json"}],
              "params":{"argv":["echo","after"]},
              "resources":{"cpu":4,"mem_mb":256},
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":50},
              "effects":["network"]
            }
          ],
          "edges":[{"from":{"node_id":"c","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("after graph should parse");

    let before_result = build_planner_analysis(
        &before,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("before analysis should succeed");
    let after_result = build_planner_analysis(
        &after,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after analysis should succeed");

    let report = compare_plan_equivalence(&before_result, &after_result);
    assert!(!report.equivalent);
    assert_eq!(report.equivalence_class, PlannerEquivalenceClass::NotEquivalent);
    assert!(!report.execution_fingerprint_equal);
    assert_eq!(
        report.non_equivalence_causes,
        vec![
            "added_dependency:data:-:-:c:out->b:in".to_string(),
            "added_node:c".to_string(),
            "changed_effects:b".to_string(),
            "changed_outputs:b".to_string(),
            "changed_params:b".to_string(),
            "changed_resources:b".to_string(),
            "changed_retry_timeout:b".to_string(),
            "removed_dependency:data:-:-:a:out->b:in".to_string(),
            "removed_node:a".to_string()
        ]
    );
}

#[test]
fn plan_equivalence_reports_semantic_drift_even_when_execution_fingerprint_matches() {
    let before = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"out","path":"b/out"}],
              "params":{"argv":["echo","before"]},
              "resources":{"cpu":1,"mem_mb":64},
              "timeout_ms":1000,
              "retry":{"max_attempts":1,"backoff_ms":10}
            }
          ],
          "edges":[{"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("before graph should parse");
    let after = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"c","kind":"const","outputs":[{"name":"out","path":"c/out"}],"params":{"value":2}},
            {
              "id":"b",
              "kind":"shell",
              "inputs":["in"],
              "outputs":[{"name":"result","path":"b/result.json"}],
              "params":{"argv":["echo","after"]},
              "resources":{"cpu":4,"mem_mb":256},
              "timeout_ms":5000,
              "retry":{"max_attempts":3,"backoff_ms":50}
            }
          ],
          "edges":[{"from":{"node_id":"c","port":"out"},"to":{"node_id":"b","port":"in"}}]
        }"#,
    )
    .expect("after graph should parse");

    let before_result = build_planner_analysis(
        &before,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("before analysis should succeed");
    let after_result = build_planner_analysis(
        &after,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("after analysis should succeed");

    let report = compare_plan_equivalence(&before_result, &after_result);
    assert!(!report.equivalent);
    assert_eq!(report.equivalence_class, PlannerEquivalenceClass::NotEquivalent);
    assert!(report.execution_fingerprint_equal);
    assert_eq!(
        report.non_equivalence_causes,
        vec![
            "added_dependency:data:-:-:c:out->b:in".to_string(),
            "added_node:c".to_string(),
            "changed_outputs:b".to_string(),
            "changed_params:b".to_string(),
            "changed_resources:b".to_string(),
            "changed_retry_timeout:b".to_string(),
            "removed_dependency:data:-:-:a:out->b:in".to_string(),
            "removed_node:a".to_string()
        ]
    );
}

#[test]
fn planner_rejects_unsupported_runtime_capability_during_lowering() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"resources":{"cpu":1,"mem_mb":64},"params":{"value":1}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let err = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect_err("planner must fail for unsupported capability requirements");
    assert!(err.contains("unsupported runtime capability"));
}

#[test]
fn planner_rejects_impossible_named_resource_requirements() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"licensed",
              "kind":"const",
              "outputs":[{"name":"out","path":"licensed/out"}],
              "resources":{"cpu":1,"mem_mb":64,"named_resources":{"license.render":0}},
              "params":{"value":1}
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let err = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect_err("planner must reject impossible named resource requirements");
    assert!(err.contains("impossible named resource requirement"));
}

#[test]
fn planner_reports_path_previews_when_run_root_is_known() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"shell",
              "kind":"shell",
              "outputs":[{"name":"out","path":"out.txt"}],
              "params":{"argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"]},
              "effects":["filesystem"]
            },
            {
              "id":"container",
              "kind":"container",
              "outputs":[{"name":"out","path":"out.txt"}],
              "container":{
                "image":"alpine:3.20",
                "argv":["cp","{inputs_dir}/seed.txt","{outputs_dir}/result.txt"],
                "workdir":"{work_dir}/scratch",
                "engine":"docker"
              },
              "effects":["filesystem"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let temp = tempfile::tempdir().expect("tmp");
    let result = build_planner_analysis(
        &graph,
        &RuntimeConfig {
            run_root: Some(temp.path().to_path_buf()),
            run_id: Some("preview".to_string()),
            ..RuntimeConfig::default()
        },
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("planner build should succeed");

    let previews = result.path_previews.expect("path previews");
    let shell_preview =
        previews.iter().find(|preview| preview.node_id == "shell").expect("shell preview");
    assert_eq!(shell_preview.execution_surface, "host");
    assert_eq!(
        shell_preview.resolved_argv.as_ref().expect("shell argv"),
        &vec![
            "cp".to_string(),
            temp.path().join("run.tmp-preview/nodes/shell/inputs/seed.txt").display().to_string(),
            temp.path()
                .join("run.tmp-preview/nodes/shell/outputs/result.txt")
                .display()
                .to_string(),
        ]
    );
    assert!(shell_preview.resolved_paths.iter().any(|path| path.key_path == "$.argv[1]"
        && path.resolved_path.contains("/nodes/shell/inputs/seed.txt")));

    let container_preview =
        previews.iter().find(|preview| preview.node_id == "container").expect("container preview");
    assert_eq!(container_preview.execution_surface, "container");
    assert_eq!(
        container_preview.resolved_argv.as_ref().expect("container argv"),
        &vec![
            "cp".to_string(),
            "/bijux/node/inputs/seed.txt".to_string(),
            "/bijux/node/outputs/result.txt".to_string(),
        ]
    );
    assert!(container_preview
        .resolved_paths
        .iter()
        .any(|path| path.key_path == "container.workdir"
            && path.resolved_path == "/bijux/node/work/scratch"));
}

#[test]
fn planner_rejects_unresolved_container_command_templates() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"container",
              "kind":"container",
              "outputs":[{"name":"out","path":"out.txt"}],
              "params":{},
              "container":{
                "image":"alpine:3.20",
                "argv":["tool","{params.missing}"],
                "engine":"docker"
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("graph should parse");

    let err = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect_err("planner must reject unresolved container templates");
    assert!(err.contains("validation failed"));
}
