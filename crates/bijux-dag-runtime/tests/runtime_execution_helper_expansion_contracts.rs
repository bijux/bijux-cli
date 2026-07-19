use bijux_dag_artifacts as _;
use bijux_dag_runtime::{
    apply_backfill_throttling, build_backfill_plan, build_plan, build_planner_analysis,
    build_scheduler, classify_failure, compute_partial_run_closure, deduplicate_trigger_events,
    deterministic_schedule_order, diff_plans, evaluate_sla_metrics, explain_plan,
    failure_allows_downstream_readiness, failure_propagation_is_deterministic, fingerprint_plan,
    node_transition_allowed, run_batches, run_transition_allowed, scheduler_contract_profile,
    scheduler_invariants_hold, validate_cron_expression, validate_schedule_policy_combination,
    validate_schedule_registry, BackfillFailurePolicy, BackfillRequest, BackfillThrottlingPolicy,
    CatchUpPolicy, ConcurrencyPolicyLayers, FailurePropagationMode, NodeLifecycleState,
    PlannerGuardrails, PlannerPhase, PriorityClass, QueueIdentity, QueueIsolationPolicy, ReadyNode,
    RetryPolicySemantics, RunBatchPolicy, RunLifecycleState, RuntimeConfig, ScheduleDefinition,
    ScheduleRegistry, ScheduleSubmissionStatus, ScheduledSubmission, SchedulerFairness,
    SchedulerPolicy, SelectorSet, TriggerSpec,
};
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::{BTreeMap, VecDeque};
use tempfile as _;
use thiserror as _;

fn tiny_graph() -> bijux_dag_core::Graph {
    bijux_dag_core::parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}]},
            {"id":"c","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"c/out"}]}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"c","port":"in"}}
          ]
        }"#,
    )
    .expect("graph")
}

#[test]
fn scheduler_equal_priority_ready_sets_are_deterministic() {
    let nodes = vec![
        ReadyNode { node_id: "b".to_string(), priority: 5, attempt: 0, ready_unix_ms: 100 },
        ReadyNode { node_id: "a".to_string(), priority: 5, attempt: 0, ready_unix_ms: 100 },
    ];

    let ordered = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(ordered[0].node_id, "a");
    assert_eq!(ordered[1].node_id, "b");
}

#[test]
fn schedule_validation_accepts_ranges_lists_steps_and_timezone() {
    let schedule = ScheduleDefinition {
        id: "weekday-window".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: "*/15 9-17 * * 1,3,5".to_string(),
            timezone: "America/New_York".to_string(),
        },
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 4 },
    };

    validate_cron_expression("*/15 9-17 * * 1,3,5").expect("cron");
    validate_schedule_registry(&ScheduleRegistry { definitions: vec![schedule] })
        .expect("registry");
}

#[test]
fn schedule_validation_rejects_unknown_cron_timezone() {
    let schedule = ScheduleDefinition {
        id: "bad-timezone".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: "0 1 * * *".to_string(),
            timezone: "Mars/Olympus".to_string(),
        },
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let err = validate_schedule_registry(&ScheduleRegistry { definitions: vec![schedule] })
        .expect_err("invalid timezone");
    assert!(err.contains("unsupported cron timezone"));
}

#[test]
fn scheduler_mixed_cache_and_fresh_paths_keep_order_stable() {
    let nodes = vec![
        ReadyNode { node_id: "fresh".to_string(), priority: 4, attempt: 0, ready_unix_ms: 100 },
        ReadyNode { node_id: "cached".to_string(), priority: 4, attempt: 1, ready_unix_ms: 100 },
    ];

    let ordered = deterministic_schedule_order(nodes, &BTreeMap::new());
    assert_eq!(ordered[0].node_id, "fresh");
    assert_eq!(ordered[1].node_id, "cached");
}

#[test]
fn scheduler_retry_influenced_paths_prioritize_starved_nodes() {
    let nodes = vec![
        ReadyNode { node_id: "retry-a".to_string(), priority: 1, attempt: 3, ready_unix_ms: 100 },
        ReadyNode { node_id: "new-b".to_string(), priority: 9, attempt: 0, ready_unix_ms: 100 },
    ];
    let starvation = BTreeMap::from([("retry-a".to_string(), 20)]);

    let ordered = deterministic_schedule_order(nodes, &starvation);
    assert_eq!(ordered[0].node_id, "retry-a");
}

#[test]
fn scheduler_workload_cpu_memory_budget_accounting_fields_are_preserved() {
    let policy = BackfillThrottlingPolicy {
        max_backfill_submissions_per_tick: 10,
        reserve_live_capacity_percent: 30,
    };
    let (allowed_backfill, live_pending) = apply_backfill_throttling(10, 20, &policy);
    assert_eq!(allowed_backfill, 4);
    assert_eq!(live_pending, 20);

    let metrics = evaluate_sla_metrics(&[(10, 9)], &[(20, 19)], 3, 1);
    assert_eq!(metrics.queue_saturation_count, 3);
    assert_eq!(metrics.fairness_drift_count, 1);
}

#[test]
fn scheduler_workload_bounded_queueing_decisions_remain_stable() {
    let queue = VecDeque::from(vec!["a".to_string(), "b".to_string(), "c".to_string()]);
    let grouped = run_batches(
        queue,
        &RunBatchPolicy { allow_grouping: true, max_group_size: 2, require_same_dag: true },
    );
    assert_eq!(grouped, vec![vec!["a".to_string(), "b".to_string()], vec!["c".to_string()]]);
}

#[test]
fn state_machine_cancel_after_start_transition_is_allowed() {
    assert!(node_transition_allowed(NodeLifecycleState::Running, NodeLifecycleState::Cancelled));
    assert!(run_transition_allowed(RunLifecycleState::Running, RunLifecycleState::Cancelled));
}

#[test]
fn state_machine_timeout_after_start_is_classified_distinctly() {
    let timeout = classify_failure(false, false, false, true, false, false);
    let cancelled = classify_failure(false, true, false, false, false, false);
    assert_ne!(format!("{timeout:?}"), format!("{cancelled:?}"));
}

#[test]
fn state_machine_partial_rerun_closure_stays_dependency_complete() {
    let graph = tiny_graph();
    let result = build_planner_analysis(
        &graph,
        &RuntimeConfig::default(),
        &SelectorSet::default(),
        &PlannerGuardrails { allow_semantic_optimizations: true },
    )
    .expect("analysis");

    let closure = compute_partial_run_closure(&result.plan, &["c".to_string()]);
    assert!(closure.contains("a"));
    assert!(closure.contains("b"));
    assert!(closure.contains("c"));
}

#[test]
fn semantics_governance_sacred_rules_match_trace_expectations() {
    let policy =
        RetryPolicySemantics { max_attempts: 3, initial_backoff_ms: 10, exponential: true };
    assert!(bijux_dag_runtime::retry_allowed(1, &policy));
    assert!(!bijux_dag_runtime::retry_allowed(3, &policy));
    assert!(!failure_propagation_is_deterministic(true, true));
    assert!(!failure_propagation_is_deterministic(true, false));
    assert!(failure_propagation_is_deterministic(false, true));
}

#[test]
fn planner_analysis_diagnostics_and_fingerprints_are_deterministic() {
    let graph = tiny_graph();
    let config = RuntimeConfig::default();
    let selector = SelectorSet::default();
    let guardrails = PlannerGuardrails { allow_semantic_optimizations: true };

    let first = build_planner_analysis(&graph, &config, &selector, &guardrails).expect("first");
    let second = build_planner_analysis(&graph, &config, &selector, &guardrails).expect("second");
    assert_eq!(first.phases, second.phases);
    assert_eq!(first.annotations.len(), second.annotations.len());
    assert!(!first.plan_fingerprint.is_empty());
    assert!(!second.plan_fingerprint.is_empty());

    let diff = diff_plans(&first, &second);
    assert!(!diff.graph_fingerprint_changed);
    assert!(!diff.execution_fingerprint_changed);
    assert!(!diff.execution_affecting_changed);
    assert!(!diff.metadata_only_changed);
    assert!(diff.added_nodes.is_empty());
    assert!(diff.changed_params.is_empty());

    let explain = explain_plan(&first);
    assert!(explain.phases.contains(&PlannerPhase::ScheduleReadyTransform));
    let fp = fingerprint_plan(&first.plan, &first.annotations).expect("fp");
    assert_eq!(fp, first.plan_fingerprint);
}

#[test]
fn scheduler_backpressure_and_registry_validation_paths_are_exercised() {
    let schedule = ScheduleDefinition {
        id: "sched-1".to_string(),
        dag_name: "dag".to_string(),
        dag_version_policy: "pinned".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
        },
        queue: bijux_dag_runtime::QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: bijux_dag_runtime::ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: bijux_dag_runtime::CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let registry = ScheduleRegistry { definitions: vec![schedule] };
    validate_schedule_registry(&registry).expect("registry");
    validate_cron_expression("* * * * *").expect("cron");

    let decisions = deduplicate_trigger_events(&[
        "dag:1".to_string(),
        "dag:1".to_string(),
        "dag:2".to_string(),
    ]);
    assert!(decisions[1].deduplicated);

    let scheduler = build_scheduler(&SchedulerPolicy {
        max_parallelism: 1,
        cpu_budget: Some(1),
        memory_budget_mb: None,
        gpu_device_budget: None,
        named_resource_capacities: std::collections::BTreeMap::new(),
        fairness: SchedulerFairness::Deterministic,
        queue_isolation: QueueIsolationPolicy::SingleQueue,
        bounded_executor_capacity: 1,
        prefer_throughput_scheduler: false,
    });
    let profile = scheduler_contract_profile();
    assert_eq!(format!("{:?}", profile.ready_tie_break), "PriorityCpuMemoryFitThenNodeId");
    assert!(failure_allows_downstream_readiness(FailurePropagationMode::ContinueIndependent));

    let graph = tiny_graph();
    let plan = build_plan(&graph, &RuntimeConfig::default());
    let state = bijux_dag_runtime::SchedulerState::from_plan(&plan);
    assert!(scheduler_invariants_hold(&state));
    let _ = scheduler;
}

#[test]
fn schedule_validation_rejects_blank_queue_and_noncron_catchup() {
    let schedule = ScheduleDefinition {
        id: "manual-catchup".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Manual,
        queue: QueueIdentity {
            queue_name: "   ".to_string(),
            tenant: Some("tenant-a".to_string()),
        },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 1 },
    };

    let err = validate_schedule_policy_combination(&schedule).expect_err("invalid schedule");
    assert!(err.contains("queue_name") || err.contains("catch-up"));
}

#[test]
fn schedule_validation_rejects_zero_or_inconsistent_concurrency_layers() {
    let schedule = ScheduleDefinition {
        id: "zero-cap".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
        },
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(0),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let err = validate_schedule_policy_combination(&schedule).expect_err("invalid concurrency");
    assert!(err.contains("greater than zero"));
}

#[test]
fn schedule_validation_rejects_backfill_that_exceeds_queue_capacity() {
    let schedule = ScheduleDefinition {
        id: "backfill-over-cap".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 100,
            window_end_unix_ms: 200,
            partition_by: Some("sample".to_string()),
            partition_keys: Vec::new(),
            max_parallelism: 4,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::High,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(2),
            per_queue: Some(2),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let err = validate_schedule_policy_combination(&schedule).expect_err("invalid backfill");
    assert!(err.contains("exceeds queue concurrency cap"));
}

#[test]
fn schedule_validation_rejects_partition_list_without_partition_name() {
    let schedule = ScheduleDefinition {
        id: "backfill-partition-list".to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 100,
            window_end_unix_ms: 200,
            partition_by: None,
            partition_keys: vec!["sample-a".to_string()],
            max_parallelism: 1,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::High,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(1),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let err = validate_schedule_policy_combination(&schedule).expect_err("invalid partition list");
    assert!(err.contains("partition_keys require partition_by"));
}

#[test]
fn planner_backfill_and_submission_ordering_helpers_are_stable() {
    let bf = build_backfill_plan(1_000, 1_120, vec!["p1".to_string(), "p2".to_string()]);
    assert_eq!(bf.window_start_unix_ms, 1_000);
    assert_eq!(bf.partition_keys.len(), 2);

    let mut by_prio = BTreeMap::new();
    by_prio.insert("s1".to_string(), PriorityClass::High);
    by_prio.insert("s2".to_string(), PriorityClass::Low);
    let ordered = bijux_dag_runtime::weighted_priority_tie_break_order(
        vec![
            ScheduledSubmission {
                schedule_id: "s2".to_string(),
                run_id: "r2".to_string(),
                created_unix_ms: 2,
                status: ScheduleSubmissionStatus::Pending,
            },
            ScheduledSubmission {
                schedule_id: "s1".to_string(),
                run_id: "r1".to_string(),
                created_unix_ms: 1,
                status: ScheduleSubmissionStatus::Pending,
            },
        ],
        &by_prio,
        &bijux_dag_runtime::WeightedPriorityPolicy {
            critical_weight: 100,
            high_weight: 50,
            standard_weight: 10,
            low_weight: 1,
        },
    );
    assert_eq!(ordered[0].schedule_id, "s1");
}
