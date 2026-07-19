use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use chrono::{LocalResult, TimeZone, Utc};
use chrono_tz::America::New_York;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2::{Digest, Sha256};
use tempfile as _;
use thiserror as _;

use bijux_dag_core::{parse_graph_strict, GraphInputSpec};
use bijux_dag_runtime::{
    advance_backfill_operation, apply_backfill_throttling, apply_submission_status_updates,
    build_plan, build_schedule_override_status, build_schedule_queue_state, build_scheduler,
    cancel_backfill_operation, compile_backfill_operation, compile_submission_request,
    deduplicate_trigger_events, deterministic_tick_order, dispatch_schedule_queue_runs,
    evaluate_schedule_submissions, evaluate_schedule_submissions_with_overrides,
    evaluate_sla_metrics, materialize_next_runs, pause_backfill_operation, pause_schedule,
    record_schedule_override, resume_backfill_operation, resume_schedule,
    retry_failed_backfill_runs, run_batches, scheduler_debug_event_log, scheduler_invariants_hold,
    summarize_backfill_operation, trace_event_count_by_category, validate_schedule_registry,
    BackfillAdvanceRequest, BackfillFailurePolicy, BackfillLifecycleStatus, BackfillRequest,
    BackfillRunStatus, BackfillStatusUpdate, BackfillThrottlingPolicy, CatchUpPolicy,
    ConcurrencyPolicyLayers, DependencyCompletionRecord, DependencyCounter,
    DependencyTriggerCondition, ManualSubmissionRequest, PriorityClass, QueueIdentity, ReadyNode,
    ReadyQueue, RunBatchPolicy, RuntimeAuditEvent, RuntimeConfig, ScheduleDefinition,
    ScheduleEvaluationInputs, ScheduleEventLineage, ScheduleEventRecord, ScheduleInputSource,
    ScheduleOverrideAction, ScheduleOverrideRecord, ScheduleOverrideState,
    SchedulePriorityDispatchPolicy, ScheduleRegistry, ScheduleSubmissionLedger,
    ScheduleSubmissionLedgerEntry, ScheduleSubmissionStatus, ScheduleSubmissionStatusUpdate,
    ScheduledSubmission, Selector, SelectorSet, SignalRecord, StarvationPreventionPolicy,
    SubmissionTriggerKind, TriggerSpec, WeightedPriorityPolicy,
};
use std::collections::{BTreeMap, VecDeque};
use std::fs;
use std::time::Instant;

#[test]
fn scheduler_tick_order_is_reproducible_across_runs() {
    let base =
        vec![sub("sched-z", "r2", 200), sub("sched-a", "r1", 100), sub("sched-a", "r0", 100)];
    let first = deterministic_tick_order(base.clone());
    let second = deterministic_tick_order(base);
    assert_eq!(first, second);
}

#[test]
fn scheduler_backpressure_throttles_backfill_when_live_load_is_high() {
    let policy = BackfillThrottlingPolicy {
        max_backfill_submissions_per_tick: 10,
        reserve_live_capacity_percent: 40,
    };
    let (allowed, pending_live) = apply_backfill_throttling(10, 20, &policy);
    assert!(allowed < 10);
    assert_eq!(pending_live, 20);
}

#[test]
fn scheduler_backfill_operation_expands_time_window_and_partition_list() {
    let schedule = schedule_definition(
        "historical-catalog",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 121_000,
            partition_by: Some("dataset".to_string()),
            partition_keys: vec!["sample-a".to_string(), "sample-b".to_string()],
            max_parallelism: 2,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
    );

    let operation =
        compile_backfill_operation(&schedule, Some("catalog-backfill"), 500).expect("operation");
    let coordinates = operation
        .runs
        .iter()
        .map(|run| (run.requested_unix_ms, run.partition_key.clone()))
        .collect::<Vec<_>>();

    assert_eq!(operation.lifecycle, BackfillLifecycleStatus::Active);
    assert_eq!(coordinates.len(), 6);
    assert_eq!(
        coordinates,
        vec![
            (1_000, Some("sample-a".to_string())),
            (1_000, Some("sample-b".to_string())),
            (61_000, Some("sample-a".to_string())),
            (61_000, Some("sample-b".to_string())),
            (121_000, Some("sample-a".to_string())),
            (121_000, Some("sample-b".to_string())),
        ]
    );
}

#[test]
fn scheduler_backfill_operation_honors_parallelism_and_throttling() {
    let schedule = schedule_definition(
        "historical-catalog",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 181_000,
            partition_by: Some("dataset".to_string()),
            partition_keys: vec!["sample-a".to_string()],
            max_parallelism: 2,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
    );
    let operation = compile_backfill_operation(&schedule, None, 500).expect("operation");

    let first = advance_backfill_operation(
        &operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_500,
            pending_live_runs: 2,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("advance");
    assert_eq!(first.dispatched_requests.len(), 2);
    assert_eq!(first.active_runs, 2);
    assert_eq!(first.queued_runs, 2);

    let second = advance_backfill_operation(
        &first.operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 2_000,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: vec![
                BackfillStatusUpdate {
                    run_id: first.dispatched_requests[0].run_id.clone(),
                    status: BackfillRunStatus::Completed,
                    updated_unix_ms: 1_700,
                },
                BackfillStatusUpdate {
                    run_id: first.dispatched_requests[1].run_id.clone(),
                    status: BackfillRunStatus::Running,
                    updated_unix_ms: 1_800,
                },
            ],
        },
    )
    .expect("advance after completion");
    assert_eq!(second.dispatched_requests.len(), 1);
    assert_eq!(second.active_runs, 2);
    assert_eq!(second.queued_runs, 1);
}

#[test]
fn scheduler_backfill_operation_can_pause_resume_and_cancel() {
    let schedule = schedule_definition(
        "historical-catalog",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 61_000,
            partition_by: None,
            partition_keys: Vec::new(),
            max_parallelism: 1,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
    );
    let mut operation = compile_backfill_operation(&schedule, None, 500).expect("operation");

    pause_backfill_operation(&mut operation, 1_000, Some("operator hold".to_string()))
        .expect("pause");
    assert_eq!(operation.lifecycle, BackfillLifecycleStatus::Paused);
    let paused = advance_backfill_operation(
        &operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_100,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("paused advance");
    assert!(paused.dispatched_requests.is_empty());

    resume_backfill_operation(&mut operation, 1_200).expect("resume");
    let resumed = advance_backfill_operation(
        &operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_300,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("resumed advance");
    assert_eq!(resumed.dispatched_requests.len(), 1);

    let mut cancelled = resumed.operation;
    cancel_backfill_operation(&mut cancelled, 1_400, Some("operator stop".to_string()))
        .expect("cancel");
    assert_eq!(cancelled.lifecycle, BackfillLifecycleStatus::Cancelled);
    assert!(cancelled.runs.iter().any(|run| matches!(run.status, BackfillRunStatus::Cancelled)));
}

#[test]
fn scheduler_backfill_failure_policy_pauses_or_cancels_remaining_work() {
    let pause_schedule = schedule_definition(
        "historical-catalog-pause",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 61_000,
            partition_by: None,
            partition_keys: Vec::new(),
            max_parallelism: 2,
            failure_policy: BackfillFailurePolicy::Pause,
        }),
    );
    let cancel_schedule = schedule_definition(
        "historical-catalog-cancel",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 121_000,
            partition_by: None,
            partition_keys: Vec::new(),
            max_parallelism: 2,
            failure_policy: BackfillFailurePolicy::Cancel,
        }),
    );
    let pause_operation = compile_backfill_operation(&pause_schedule, None, 500).expect("pause op");
    let cancel_operation =
        compile_backfill_operation(&cancel_schedule, None, 500).expect("cancel op");

    let pause_dispatched = advance_backfill_operation(
        &pause_operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_500,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("pause dispatch");
    let pause_result = advance_backfill_operation(
        &pause_dispatched.operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 2_000,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: vec![BackfillStatusUpdate {
                run_id: pause_dispatched.dispatched_requests[0].run_id.clone(),
                status: BackfillRunStatus::Failed,
                updated_unix_ms: 1_900,
            }],
        },
    )
    .expect("pause result");
    assert_eq!(pause_result.operation.lifecycle, BackfillLifecycleStatus::Paused);
    assert!(pause_result.dispatched_requests.is_empty());

    let cancel_dispatched = advance_backfill_operation(
        &cancel_operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_500,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("cancel dispatch");
    let cancel_result = advance_backfill_operation(
        &cancel_dispatched.operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 2_000,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: vec![BackfillStatusUpdate {
                run_id: cancel_dispatched.dispatched_requests[0].run_id.clone(),
                status: BackfillRunStatus::Failed,
                updated_unix_ms: 1_900,
            }],
        },
    )
    .expect("cancel result");
    assert_eq!(cancel_result.operation.lifecycle, BackfillLifecycleStatus::Cancelled);
    assert!(
        cancel_result
            .operation
            .runs
            .iter()
            .filter(|run| matches!(run.status, BackfillRunStatus::Queued))
            .count()
            == 0
    );
}

#[test]
fn scheduler_backfill_retry_requeues_failed_partition_with_attempt_history() {
    let schedule = schedule_definition(
        "historical-catalog",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 61_000,
            partition_by: Some("dataset".to_string()),
            partition_keys: vec!["sample-a".to_string()],
            max_parallelism: 1,
            failure_policy: BackfillFailurePolicy::Pause,
        }),
    );
    let operation =
        compile_backfill_operation(&schedule, Some("catalog-backfill"), 500).expect("operation");
    let dispatched = advance_backfill_operation(
        &operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 1_500,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("dispatch");
    let failed_run_id = dispatched.dispatched_requests[0].run_id.clone();
    let failed = advance_backfill_operation(
        &dispatched.operation,
        &BackfillAdvanceRequest {
            now_unix_ms: 2_000,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: vec![BackfillStatusUpdate {
                run_id: failed_run_id.clone(),
                status: BackfillRunStatus::Failed,
                updated_unix_ms: 1_900,
            }],
        },
    )
    .expect("fail operation");
    assert_eq!(failed.operation.lifecycle, BackfillLifecycleStatus::Paused);

    let mut retried = failed.operation;
    let retried_count = retry_failed_backfill_runs(&mut retried, 2_100).expect("retry");
    assert_eq!(retried_count, 1);
    assert_eq!(retried.lifecycle, BackfillLifecycleStatus::Active);
    assert_eq!(retried.lifecycle_reason, None);

    let retried_run = retried
        .runs
        .iter()
        .find(|run| run.partition_key.as_deref() == Some("sample-a"))
        .expect("retried partition");
    assert_eq!(retried_run.status, BackfillRunStatus::Queued);
    assert_eq!(retried_run.attempt, 2);
    assert_eq!(retried_run.previous_run_ids, vec![failed_run_id.clone()]);
    assert_ne!(retried_run.run_id, failed_run_id);

    let resumed = advance_backfill_operation(
        &retried,
        &BackfillAdvanceRequest {
            now_unix_ms: 2_200,
            pending_live_runs: 0,
            throttling_policy: BackfillThrottlingPolicy {
                max_backfill_submissions_per_tick: 10,
                reserve_live_capacity_percent: 0,
            },
            status_updates: Vec::new(),
        },
    )
    .expect("resume dispatch");
    assert_eq!(resumed.dispatched_requests.len(), 1);
    assert_eq!(resumed.dispatched_requests[0].requested_unix_ms, 1_000);
    assert_eq!(resumed.dispatched_requests[0].run_id, retried_run.run_id);
}

#[test]
fn scheduler_backfill_summary_reports_partition_statuses_and_retry_totals() {
    let schedule = schedule_definition(
        "historical-catalog",
        TriggerSpec::Backfill(BackfillRequest {
            window_start_unix_ms: 1_000,
            window_end_unix_ms: 121_000,
            partition_by: Some("dataset".to_string()),
            partition_keys: vec!["sample-a".to_string()],
            max_parallelism: 1,
            failure_policy: BackfillFailurePolicy::Continue,
        }),
    );
    let mut operation =
        compile_backfill_operation(&schedule, Some("catalog-backfill"), 500).expect("operation");

    operation.runs[0].status = BackfillRunStatus::Completed;
    operation.runs[1].status = BackfillRunStatus::Running;
    operation.runs[1].attempt = 3;
    operation.runs[1].previous_run_ids = vec![
        "sched-historical-catalog-old-1".to_string(),
        "sched-historical-catalog-old-2".to_string(),
    ];
    operation.runs[2].status = BackfillRunStatus::Cancelled;
    operation.lifecycle = BackfillLifecycleStatus::Paused;
    operation.lifecycle_reason = Some("operator review".to_string());

    let summary = summarize_backfill_operation(&operation);
    assert_eq!(summary.backfill_id, "catalog-backfill");
    assert_eq!(summary.schedule_id, "historical-catalog");
    assert_eq!(summary.lifecycle, BackfillLifecycleStatus::Paused);
    assert_eq!(summary.lifecycle_reason.as_deref(), Some("operator review"));
    assert_eq!(summary.total_runs, 3);
    assert_eq!(summary.queued_runs, 0);
    assert_eq!(summary.submitted_runs, 0);
    assert_eq!(summary.running_runs, 1);
    assert_eq!(summary.completed_runs, 1);
    assert_eq!(summary.failed_runs, 0);
    assert_eq!(summary.cancelled_runs, 1);
    assert_eq!(summary.total_retry_attempts, 2);
    assert_eq!(summary.partitions.len(), 3);
    assert_eq!(summary.partitions[1].attempt, 3);
    assert_eq!(
        summary.partitions[1].previous_run_ids,
        vec![
            "sched-historical-catalog-old-1".to_string(),
            "sched-historical-catalog-old-2".to_string()
        ]
    );
}

#[test]
fn scheduler_cancellation_prevents_batch_scheduling() {
    let graph = simple_graph();
    let cfg = RuntimeConfig::default();
    let plan = build_plan(&graph, &cfg);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&cfg.scheduler_policy);

    let decision = scheduler.next_batch(&graph, &mut ready, &cfg, Instant::now(), true);
    assert!(decision.cancelled);
    assert!(decision.batch.is_empty());
}

#[test]
fn scheduler_retry_queue_requeue_roundtrip_is_stable() {
    let graph = simple_graph();
    let cfg = RuntimeConfig::default();
    let plan = build_plan(&graph, &cfg);
    let mut state = bijux_dag_runtime::SchedulerState::from_plan(&plan);

    state.queue_retry("a");
    state.queue_retry("a");
    assert_eq!(state.retry_snapshot(), vec!["a".to_string()]);

    state.requeue_retries();
    assert!(state.retry_snapshot().is_empty());
    assert!(state.ready_snapshot().contains(&"a".to_string()));
    assert!(scheduler_invariants_hold(&state));
}

#[test]
fn scheduler_partial_rerun_keeps_dependency_closure() {
    let graph = chain_graph();
    let cfg = RuntimeConfig {
        selectors: SelectorSet {
            include: vec![Selector::IdPrefix("c".to_string())],
            exclude: vec![],
        },
        partial_rerun_dependency_closure: true,
        ..RuntimeConfig::default()
    };
    let plan = build_plan(&graph, &cfg);
    assert!(plan.order.contains(&"a".to_string()));
    assert!(plan.order.contains(&"b".to_string()));
    assert!(plan.order.contains(&"c".to_string()));
}

#[test]
fn scheduler_queue_depth_and_grouping_metrics_are_visible() {
    let queue = VecDeque::from(vec![
        "r1".to_string(),
        "r2".to_string(),
        "r3".to_string(),
        "r4".to_string(),
    ]);
    let grouped = run_batches(
        queue,
        &RunBatchPolicy { allow_grouping: true, max_group_size: 2, require_same_dag: true },
    );
    assert_eq!(grouped.len(), 2);
    assert_eq!(grouped[0].len(), 2);
}

#[test]
fn scheduler_runtime_sla_metrics_surface_all_counters() {
    let m = evaluate_sla_metrics(&[(2, 1), (1, 1)], &[(5, 4)], 7, 3);
    assert_eq!(m.missed_expected_start, 1);
    assert_eq!(m.missed_expected_finish, 1);
    assert_eq!(m.queue_saturation_count, 7);
    assert_eq!(m.fairness_drift_count, 3);
}

#[test]
fn scheduler_trace_capture_contains_retry_and_ready_events() {
    let graph = simple_graph();
    let cfg = RuntimeConfig::default();
    let plan = build_plan(&graph, &cfg);
    let mut state = bijux_dag_runtime::SchedulerState::from_plan(&plan);

    state.complete_success("a");
    state.queue_retry("b");
    state.requeue_retries();

    let log = scheduler_debug_event_log(&state);
    let kinds = log.iter().map(|e| format!("{:?}", e.kind)).collect::<Vec<_>>();
    assert!(kinds.iter().any(|k| k.contains("NodeReady")));
    assert!(kinds.iter().any(|k| k.contains("NodeRetryQueued")));
    assert!(kinds.iter().any(|k| k.contains("NodeRetryRequeued")));
}

#[test]
fn scheduler_performance_regression_guard_for_large_ready_sets() {
    let mut ready = Vec::new();
    for i in 0..20_000u32 {
        ready.push(ReadyNode {
            node_id: format!("n{i}"),
            priority: 5,
            attempt: 0,
            ready_unix_ms: i as u128,
        });
    }

    let start = Instant::now();
    let ordered = bijux_dag_runtime::deterministic_schedule_order(ready, &BTreeMap::new());
    let elapsed = start.elapsed();

    assert_eq!(ordered.len(), 20_000);
    assert!(elapsed.as_millis() < 6_000, "scheduler ordering too slow: {elapsed:?}");
}

#[test]
fn scheduler_regression_corpus_ordering_remains_stable() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = root.join("tests/fixtures/performance/scheduler_regression_corpus.json");
    let raw = fs::read_to_string(path).expect("read fixture");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("fixture json");

    let submissions = value["submissions"]
        .as_array()
        .expect("submissions")
        .iter()
        .map(|v| ScheduledSubmission {
            schedule_id: v["schedule_id"].as_str().unwrap_or_default().to_string(),
            run_id: v["run_id"].as_str().unwrap_or_default().to_string(),
            created_unix_ms: v["created_unix_ms"].as_u64().unwrap_or_default() as u128,
            status: ScheduleSubmissionStatus::Pending,
        })
        .collect::<Vec<_>>();
    let expected = value["expected_order"]
        .as_array()
        .expect("expected")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>();

    let ordered = deterministic_tick_order(submissions);
    let got = ordered.iter().map(|s| format!("{}:{}", s.schedule_id, s.run_id)).collect::<Vec<_>>();

    assert_eq!(got, expected);

    let mut h = Sha256::new();
    h.update(got.join("\n").as_bytes());
    let digest = format!("{:x}", h.finalize());
    assert_eq!(digest.len(), 64);
}

fn sub(schedule_id: &str, run_id: &str, created_unix_ms: u128) -> ScheduledSubmission {
    ScheduledSubmission {
        schedule_id: schedule_id.to_string(),
        run_id: run_id.to_string(),
        created_unix_ms,
        status: ScheduleSubmissionStatus::Pending,
    }
}

fn simple_graph() -> bijux_dag_core::Graph {
    parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}]},
            {"id":"b","kind":"const","inputs":["in"],"outputs":[{"name":"out","path":"b/out"}]}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}}
          ]
        }"#,
    )
    .expect("graph")
}

fn chain_graph() -> bijux_dag_core::Graph {
    parse_graph_strict(
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
fn scheduler_materialization_and_trigger_dedup_helpers_are_consistent() {
    let definition = ScheduleDefinition {
        id: "sched-1".to_string(),
        dag_name: "dag".to_string(),
        dag_version_policy: "pinned".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: "* * * * *".to_string(),
            timezone: "UTC".to_string(),
        },
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(2),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    };

    let preview = materialize_next_runs(&definition, 1_000, 3);
    assert_eq!(preview.next_run_unix_ms.len(), 3);

    let keys = vec!["a".to_string(), "b".to_string(), "a".to_string()];
    let dedup = deduplicate_trigger_events(&keys);
    assert_eq!(dedup.iter().filter(|d| d.deduplicated).count(), 1);

    let mut audits = Vec::new();
    audits.push(RuntimeAuditEvent {
        event_id: "1".to_string(),
        run_id: "r".to_string(),
        node_id: Some("n".to_string()),
        category: "schedule".to_string(),
        details: BTreeMap::new(),
    });
    audits.push(RuntimeAuditEvent {
        event_id: "2".to_string(),
        run_id: "r".to_string(),
        node_id: Some("n".to_string()),
        category: "schedule".to_string(),
        details: BTreeMap::new(),
    });
    let grouped = trace_event_count_by_category(&audits);
    assert_eq!(grouped.get("schedule"), Some(&2));
}

#[test]
fn schedule_submit_evaluates_manual_cron_event_dependency_and_signal_triggers() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition("manual-ops", TriggerSpec::Manual),
            schedule_definition(
                "cron-minute",
                TriggerSpec::Cron {
                    expression: "* * * * *".to_string(),
                    timezone: "UTC".to_string(),
                },
            ),
            schedule_definition(
                "event-ingest",
                TriggerSpec::Event {
                    event_type: "dataset.ready".to_string(),
                    source: "catalog".to_string(),
                },
            ),
            schedule_definition(
                "dependency-publish",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::Success,
                },
            ),
            schedule_definition(
                "signal-refresh",
                TriggerSpec::Signal {
                    signal_name: "refresh-cache".to_string(),
                    payload_schema: None,
                },
            ),
        ],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 180_000,
        manual_requests: vec![ManualSubmissionRequest {
            request_id: "manual-001".to_string(),
            schedule_id: "manual-ops".to_string(),
            requested_unix_ms: 175_000,
            arguments: BTreeMap::new(),
        }],
        events: vec![ScheduleEventRecord {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
            payload: None,
        }],
        dependencies: vec![DependencyCompletionRecord {
            upstream_run_id: "atlas-run-7".to_string(),
            dag_name: "atlas.ingest".to_string(),
            status: "SUCCESS".to_string(),
            finished_unix_ms: 177_000,
        }],
        signals: vec![SignalRecord {
            signal_id: "sig-001".to_string(),
            signal_name: "refresh-cache".to_string(),
            occurred_unix_ms: 178_000,
            payload: None,
        }],
    };
    let existing = ScheduleSubmissionLedger::default();

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    let generated = report
        .generated_requests
        .iter()
        .map(|request| {
            (request.schedule_id.clone(), request.requested_unix_ms, request.trigger_kind.clone())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        generated,
        vec![
            ("manual-ops".to_string(), 175_000, SubmissionTriggerKind::Manual),
            ("event-ingest".to_string(), 176_000, SubmissionTriggerKind::Event),
            ("dependency-publish".to_string(), 177_000, SubmissionTriggerKind::Dependency),
            ("signal-refresh".to_string(), 178_000, SubmissionTriggerKind::Signal),
            ("cron-minute".to_string(), 180_000, SubmissionTriggerKind::Cron),
        ]
    );
    assert!(report.duplicate_suppressions.is_empty());
    assert_eq!(report.recorded_submissions.len(), 5);
    assert!(report.generated_requests.iter().all(|request| request.run_id.starts_with("sched-")));

    let event_request = report
        .generated_requests
        .iter()
        .find(|request| request.schedule_id == "event-ingest")
        .expect("event request");
    assert_eq!(
        event_request.event_lineage,
        Some(ScheduleEventLineage {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
        })
    );

    let event_submission = report
        .recorded_submissions
        .iter()
        .find(|entry| entry.schedule_id == "event-ingest")
        .expect("event ledger entry");
    assert_eq!(event_submission.event_lineage, event_request.event_lineage);
}

#[test]
fn schedule_submit_preserves_event_lineage_when_deduplicating_existing_event() {
    let registry = ScheduleRegistry {
        definitions: vec![schedule_definition(
            "event-ingest",
            TriggerSpec::Event {
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
            },
        )],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 200_000,
        events: vec![ScheduleEventRecord {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
            payload: Some(serde_json::json!({"tenant":"atlas"})),
        }],
        ..ScheduleEvaluationInputs::default()
    };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "event-ingest".to_string(),
            dag_name: "atlas.event-ingest".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity {
                queue_name: "catalog".to_string(),
                tenant: Some("atlas".to_string()),
            },
            priority: PriorityClass::High,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: 176_000,
            created_unix_ms: 176_000,
            run_id: "sched-event-ingest-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Event,
            dedupe_key: "event:event-ingest:evt-001".to_string(),
            event_lineage: Some(ScheduleEventLineage {
                event_id: "evt-001".to_string(),
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
                occurred_unix_ms: 176_000,
            }),
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    assert!(report.generated_requests.is_empty());
    assert_eq!(report.duplicate_suppressions.len(), 1);
    assert_eq!(report.recorded_submissions.len(), 1);
    assert_eq!(
        report.recorded_submissions[0].event_lineage,
        Some(ScheduleEventLineage {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
        })
    );
}

#[test]
fn schedule_submit_matches_dependency_success_failure_and_any_terminal_conditions() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition(
                "dependency-on-success",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::Success,
                },
            ),
            schedule_definition(
                "dependency-on-failure",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::Failure,
                },
            ),
            schedule_definition(
                "dependency-on-terminal",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::AnyTerminal,
                },
            ),
        ],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 220_000,
        dependencies: vec![
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-success".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "SUCCEEDED".to_string(),
                finished_unix_ms: 210_000,
            },
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-failure".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "timed out".to_string(),
                finished_unix_ms: 211_000,
            },
        ],
        ..ScheduleEvaluationInputs::default()
    };

    let report =
        evaluate_schedule_submissions(&registry, &inputs, &ScheduleSubmissionLedger::default());
    let by_schedule = report
        .generated_requests
        .iter()
        .map(|request| (request.schedule_id.as_str(), request))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(report.generated_requests.len(), 4);
    assert_eq!(
        by_schedule["dependency-on-success"].graph_inputs,
        BTreeMap::<String, serde_json::Value>::new()
    );
    assert_eq!(by_schedule["dependency-on-success"].requested_unix_ms, 210_000);
    assert_eq!(by_schedule["dependency-on-failure"].requested_unix_ms, 211_000);
    let terminal_timestamps = report
        .generated_requests
        .iter()
        .filter(|request| request.schedule_id == "dependency-on-terminal")
        .map(|request| request.requested_unix_ms)
        .collect::<Vec<_>>();
    assert_eq!(terminal_timestamps, vec![210_000, 211_000]);
}

#[test]
fn schedule_submit_deduplicates_dependency_aliases_by_trigger_condition() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition(
                "dependency-on-failure",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::Failure,
                },
            ),
            schedule_definition(
                "dependency-on-terminal",
                TriggerSpec::Dependency {
                    dag_name: "atlas.ingest".to_string(),
                    on_status: DependencyTriggerCondition::AnyTerminal,
                },
            ),
        ],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 220_000,
        dependencies: vec![
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-7".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "failed".to_string(),
                finished_unix_ms: 210_000,
            },
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-7".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "failure".to_string(),
                finished_unix_ms: 210_000,
            },
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-8".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "cancelled".to_string(),
                finished_unix_ms: 211_000,
            },
            DependencyCompletionRecord {
                upstream_run_id: "atlas-run-8".to_string(),
                dag_name: "atlas.ingest".to_string(),
                status: "timeout".to_string(),
                finished_unix_ms: 211_000,
            },
        ],
        ..ScheduleEvaluationInputs::default()
    };

    let report =
        evaluate_schedule_submissions(&registry, &inputs, &ScheduleSubmissionLedger::default());

    let failure_requests = report
        .generated_requests
        .iter()
        .filter(|request| request.schedule_id == "dependency-on-failure")
        .collect::<Vec<_>>();
    let terminal_requests = report
        .generated_requests
        .iter()
        .filter(|request| request.schedule_id == "dependency-on-terminal")
        .collect::<Vec<_>>();

    assert_eq!(failure_requests.len(), 2);
    assert_eq!(terminal_requests.len(), 2);
    assert_eq!(report.duplicate_suppressions.len(), 4);
}

#[test]
fn schedule_submit_suppresses_paused_schedule_until_explicit_resume() {
    let registry = ScheduleRegistry {
        definitions: vec![schedule_definition("manual-ops", TriggerSpec::Manual)],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 200_000,
        manual_requests: vec![ManualSubmissionRequest {
            request_id: "manual-001".to_string(),
            schedule_id: "manual-ops".to_string(),
            requested_unix_ms: 175_000,
            arguments: BTreeMap::new(),
        }],
        ..ScheduleEvaluationInputs::default()
    };
    let mut overrides = ScheduleOverrideState::default();
    pause_schedule(
        &mut overrides,
        "manual-ops",
        "atlas-ops",
        180_000,
        Some("hold while downstream validation is degraded".to_string()),
    )
    .expect("pause schedule");

    let paused = evaluate_schedule_submissions_with_overrides(
        &registry,
        &inputs,
        &ScheduleSubmissionLedger::default(),
        &overrides,
    );
    assert!(paused.generated_requests.is_empty());
    assert_eq!(paused.paused_suppressions.len(), 1);

    resume_schedule(
        &mut overrides,
        "manual-ops",
        "atlas-ops",
        190_000,
        Some("validation recovered".to_string()),
    )
    .expect("resume schedule");

    let resumed = evaluate_schedule_submissions_with_overrides(
        &registry,
        &inputs,
        &ScheduleSubmissionLedger::default(),
        &overrides,
    );
    assert_eq!(resumed.generated_requests.len(), 1);
    assert!(resumed.paused_suppressions.is_empty());
}

#[test]
fn schedule_override_status_reflects_latest_operator_action() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition("manual-ops", TriggerSpec::Manual),
            schedule_definition("event-ingest", TriggerSpec::Manual),
        ],
    };
    let mut overrides = ScheduleOverrideState::default();
    record_schedule_override(
        &mut overrides,
        ScheduleOverrideRecord {
            schedule_id: "manual-ops".to_string(),
            operator: "atlas-ops".to_string(),
            action: ScheduleOverrideAction::Pause,
            reason: Some("hold".to_string()),
            created_unix_ms: 180_000,
        },
    )
    .expect("record pause");
    record_schedule_override(
        &mut overrides,
        ScheduleOverrideRecord {
            schedule_id: "manual-ops".to_string(),
            operator: "atlas-ops".to_string(),
            action: ScheduleOverrideAction::Resume,
            reason: Some("clear".to_string()),
            created_unix_ms: 190_000,
        },
    )
    .expect("record resume");

    let statuses = build_schedule_override_status(&registry, &overrides);
    let by_schedule = statuses
        .iter()
        .map(|status| (status.schedule_id.as_str(), status))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(by_schedule["manual-ops"].paused, false);
    assert_eq!(by_schedule["manual-ops"].operator.as_deref(), Some("atlas-ops"));
    assert_eq!(by_schedule["manual-ops"].reason.as_deref(), Some("clear"));
    assert_eq!(by_schedule["manual-ops"].updated_unix_ms, Some(190_000));
    assert_eq!(by_schedule["event-ingest"].paused, false);
    assert_eq!(by_schedule["event-ingest"].updated_unix_ms, None);
}

#[test]
fn schedule_override_status_ignores_override_file_order_for_equal_timestamps() {
    let registry = ScheduleRegistry {
        definitions: vec![schedule_definition("manual-ops", TriggerSpec::Manual)],
    };
    let overrides = ScheduleOverrideState {
        records: vec![
            ScheduleOverrideRecord {
                schedule_id: "manual-ops".to_string(),
                operator: "atlas-ops".to_string(),
                action: ScheduleOverrideAction::Resume,
                reason: Some("clear".to_string()),
                created_unix_ms: 190_000,
            },
            ScheduleOverrideRecord {
                schedule_id: "manual-ops".to_string(),
                operator: "atlas-ops".to_string(),
                action: ScheduleOverrideAction::Pause,
                reason: Some("hold".to_string()),
                created_unix_ms: 190_000,
            },
        ],
    };

    let statuses = build_schedule_override_status(&registry, &overrides);
    assert_eq!(statuses[0].paused, false);
    assert_eq!(statuses[0].reason.as_deref(), Some("clear"));
}

#[test]
fn schedule_submit_binds_trigger_values_into_typed_graph_inputs() {
    let registry = ScheduleRegistry {
        definitions: vec![
            ScheduleDefinition {
                input_contract: schedule_input_contract(),
                input_bindings: BTreeMap::from([
                    ("requested_at".to_string(), ScheduleInputSource::RequestedUnixMs),
                    (
                        "manual_region".to_string(),
                        ScheduleInputSource::ManualArgument { key: "region".to_string() },
                    ),
                ]),
                ..schedule_definition("manual-ops", TriggerSpec::Manual)
            },
            ScheduleDefinition {
                input_contract: schedule_input_contract(),
                input_bindings: BTreeMap::from([
                    ("requested_at".to_string(), ScheduleInputSource::RequestedUnixMs),
                    (
                        "event_tenant".to_string(),
                        ScheduleInputSource::EventPayload { pointer: Some("/tenant".to_string()) },
                    ),
                    (
                        "event_payload".to_string(),
                        ScheduleInputSource::EventPayload { pointer: None },
                    ),
                ]),
                ..schedule_definition(
                    "event-ingest",
                    TriggerSpec::Event {
                        event_type: "dataset.ready".to_string(),
                        source: "catalog".to_string(),
                    },
                )
            },
            ScheduleDefinition {
                input_contract: schedule_input_contract(),
                input_bindings: BTreeMap::from([
                    ("requested_at".to_string(), ScheduleInputSource::RequestedUnixMs),
                    ("dependency_status".to_string(), ScheduleInputSource::DependencyStatus),
                    ("dependency_run_id".to_string(), ScheduleInputSource::DependencyUpstreamRunId),
                ]),
                ..schedule_definition(
                    "dependency-publish",
                    TriggerSpec::Dependency {
                        dag_name: "atlas.ingest".to_string(),
                        on_status: DependencyTriggerCondition::Success,
                    },
                )
            },
            ScheduleDefinition {
                input_contract: schedule_input_contract(),
                input_bindings: BTreeMap::from([
                    ("requested_at".to_string(), ScheduleInputSource::RequestedUnixMs),
                    (
                        "signal_tenant".to_string(),
                        ScheduleInputSource::SignalPayload { pointer: Some("/tenant".to_string()) },
                    ),
                ]),
                ..schedule_definition(
                    "signal-refresh",
                    TriggerSpec::Signal {
                        signal_name: "refresh-cache".to_string(),
                        payload_schema: None,
                    },
                )
            },
            ScheduleDefinition {
                input_contract: schedule_input_contract(),
                input_bindings: BTreeMap::from([
                    ("requested_at".to_string(), ScheduleInputSource::RequestedUnixMs),
                    ("window_start".to_string(), ScheduleInputSource::BackfillWindowStartUnixMs),
                    ("window_end".to_string(), ScheduleInputSource::BackfillWindowEndUnixMs),
                    ("partition_key".to_string(), ScheduleInputSource::BackfillPartitionKey),
                ]),
                ..schedule_definition(
                    "historical-catalog",
                    TriggerSpec::Backfill(BackfillRequest {
                        window_start_unix_ms: 100,
                        window_end_unix_ms: 60_100,
                        partition_by: Some("dataset".to_string()),
                        partition_keys: vec!["sample-a".to_string()],
                        max_parallelism: 2,
                        failure_policy: BackfillFailurePolicy::Continue,
                    }),
                )
            },
        ],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 180_000,
        manual_requests: vec![ManualSubmissionRequest {
            request_id: "manual-001".to_string(),
            schedule_id: "manual-ops".to_string(),
            requested_unix_ms: 175_000,
            arguments: BTreeMap::from([("region".to_string(), serde_json::json!("eu-west-1"))]),
        }],
        events: vec![ScheduleEventRecord {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
            payload: Some(serde_json::json!({
                "tenant":"atlas",
                "batch":7
            })),
        }],
        dependencies: vec![DependencyCompletionRecord {
            upstream_run_id: "atlas-run-7".to_string(),
            dag_name: "atlas.ingest".to_string(),
            status: "SUCCESS".to_string(),
            finished_unix_ms: 177_000,
        }],
        signals: vec![SignalRecord {
            signal_id: "sig-001".to_string(),
            signal_name: "refresh-cache".to_string(),
            occurred_unix_ms: 178_000,
            payload: Some(serde_json::json!({"tenant":"atlas"})),
        }],
    };

    let report =
        evaluate_schedule_submissions(&registry, &inputs, &ScheduleSubmissionLedger::default());
    let by_schedule = report
        .generated_requests
        .iter()
        .map(|request| (request.schedule_id.as_str(), &request.graph_inputs))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(by_schedule["manual-ops"]["requested_at"], serde_json::json!(175000u128));
    assert_eq!(by_schedule["manual-ops"]["manual_region"], "eu-west-1");
    assert_eq!(by_schedule["event-ingest"]["requested_at"], serde_json::json!(176000u128));
    assert_eq!(by_schedule["event-ingest"]["event_tenant"], "atlas");
    assert_eq!(by_schedule["event-ingest"]["event_payload"]["batch"], 7);
    assert_eq!(by_schedule["dependency-publish"]["dependency_status"], "success");
    assert_eq!(by_schedule["dependency-publish"]["dependency_run_id"], "atlas-run-7");
    assert_eq!(by_schedule["signal-refresh"]["signal_tenant"], "atlas");

    let backfill_request = report
        .generated_requests
        .iter()
        .find(|request| request.schedule_id == "historical-catalog")
        .expect("backfill request");
    assert_eq!(backfill_request.graph_inputs["window_start"], serde_json::json!(100u128));
    assert_eq!(backfill_request.graph_inputs["window_end"], serde_json::json!(60100u128));
    assert_eq!(backfill_request.graph_inputs["partition_key"], "sample-a");
}

#[test]
fn schedule_compile_binds_requested_timestamp_into_typed_graph_inputs() {
    let schedule = ScheduleDefinition {
        input_contract: serde_json::from_value(serde_json::json!({
            "requested_at":{"type":"integer","required":true}
        }))
        .expect("input contract"),
        input_bindings: BTreeMap::from([(
            "requested_at".to_string(),
            ScheduleInputSource::RequestedUnixMs,
        )]),
        ..schedule_definition(
            "nightly-catalog",
            TriggerSpec::Cron { expression: "0 2 * * *".to_string(), timezone: "UTC".to_string() },
        )
    };

    let request = compile_submission_request(&schedule, 176_000).expect("compiled submission");

    assert_eq!(request.schedule_id, "nightly-catalog");
    assert_eq!(request.graph_inputs["requested_at"], serde_json::json!(176000u128));
}

#[test]
fn schedule_compile_rejects_unavailable_manual_argument_binding() {
    let schedule = ScheduleDefinition {
        input_contract: serde_json::from_value(serde_json::json!({
            "manual_region":{"type":"string","required":true}
        }))
        .expect("input contract"),
        input_bindings: BTreeMap::from([(
            "manual_region".to_string(),
            ScheduleInputSource::ManualArgument { key: "region".to_string() },
        )]),
        ..schedule_definition("manual-ops", TriggerSpec::Manual)
    };

    let error = compile_submission_request(&schedule, 176_000).unwrap_err();

    assert!(error.contains("missing manual argument 'region'"));
}

#[test]
fn schedule_submit_rejects_invalid_input_mapping_before_submission() {
    let registry = ScheduleRegistry {
        definitions: vec![ScheduleDefinition {
            input_contract: serde_json::from_value(serde_json::json!({
                "event_tenant":{"type":"integer","required":true}
            }))
            .expect("input contract"),
            input_bindings: BTreeMap::from([(
                "event_tenant".to_string(),
                ScheduleInputSource::EventPayload { pointer: Some("/tenant".to_string()) },
            )]),
            ..schedule_definition(
                "event-ingest",
                TriggerSpec::Event {
                    event_type: "dataset.ready".to_string(),
                    source: "catalog".to_string(),
                },
            )
        }],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 180_000,
        events: vec![ScheduleEventRecord {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
            payload: Some(serde_json::json!({"tenant":"atlas"})),
        }],
        ..ScheduleEvaluationInputs::default()
    };

    let report =
        evaluate_schedule_submissions(&registry, &inputs, &ScheduleSubmissionLedger::default());

    assert!(report.generated_requests.is_empty());
    assert!(report.audits.iter().any(|audit| audit.decision == "mapping_rejected"
        && audit.reason.as_deref().is_some_and(|reason| reason.contains("expected integer"))));
}

#[test]
fn schedule_submit_prevents_duplicates_from_existing_ledger_and_current_tick() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition("manual-ops", TriggerSpec::Manual),
            schedule_definition(
                "event-ingest",
                TriggerSpec::Event {
                    event_type: "dataset.ready".to_string(),
                    source: "catalog".to_string(),
                },
            ),
        ],
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 200_000,
        manual_requests: vec![
            ManualSubmissionRequest {
                request_id: "manual-001".to_string(),
                schedule_id: "manual-ops".to_string(),
                requested_unix_ms: 175_000,
                arguments: BTreeMap::new(),
            },
            ManualSubmissionRequest {
                request_id: "manual-001".to_string(),
                schedule_id: "manual-ops".to_string(),
                requested_unix_ms: 175_000,
                arguments: BTreeMap::new(),
            },
        ],
        events: vec![
            ScheduleEventRecord {
                event_id: "evt-001".to_string(),
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
                occurred_unix_ms: 176_000,
                payload: None,
            },
            ScheduleEventRecord {
                event_id: "evt-001".to_string(),
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
                occurred_unix_ms: 176_000,
                payload: None,
            },
        ],
        dependencies: Vec::new(),
        signals: Vec::new(),
    };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "manual-ops".to_string(),
            dag_name: "atlas.manual-ops".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity {
                queue_name: "catalog".to_string(),
                tenant: Some("atlas".to_string()),
            },
            priority: PriorityClass::High,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: 175_000,
            created_unix_ms: 170_000,
            run_id: "sched-manual-ops-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: "manual:manual-ops:manual-001".to_string(),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);

    assert_eq!(report.generated_requests.len(), 1);
    assert_eq!(report.generated_requests[0].schedule_id, "event-ingest");
    assert_eq!(report.generated_requests[0].dedupe_key, "event:event-ingest:evt-001");
    assert_eq!(report.duplicate_suppressions.len(), 3);
    assert_eq!(report.recorded_submissions.len(), 2);
}

#[test]
fn schedule_submit_cron_catch_up_respects_existing_submissions_and_cap() {
    let registry = ScheduleRegistry {
        definitions: vec![ScheduleDefinition {
            id: "cron-minute".to_string(),
            dag_name: "atlas.cron-minute".to_string(),
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
                per_dag: Some(1),
                per_queue: Some(2),
                per_tenant: None,
                per_node_group: None,
            },
            catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 2 },
        }],
    };
    let inputs =
        ScheduleEvaluationInputs { now_unix_ms: 5 * 60_000, ..ScheduleEvaluationInputs::default() };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "cron-minute".to_string(),
            dag_name: "atlas.cron-minute".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: 2 * 60_000,
            created_unix_ms: 2 * 60_000,
            run_id: "sched-cron-minute-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: "cron:cron-minute:120000".to_string(),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    let scheduled_slots = report
        .generated_requests
        .iter()
        .map(|request| request.requested_unix_ms)
        .collect::<Vec<_>>();

    assert_eq!(scheduled_slots, vec![3 * 60_000]);
    assert_eq!(report.queue_suppressions.len(), 1);
}

#[test]
fn schedule_submit_tenant_caps_are_scoped_per_queue() {
    let registry = ScheduleRegistry {
        definitions: vec![
            schedule_definition("catalog-alpha", TriggerSpec::Manual),
            schedule_definition("catalog-beta", TriggerSpec::Manual),
        ]
        .into_iter()
        .enumerate()
        .map(|(index, mut definition)| {
            definition.queue = QueueIdentity {
                queue_name: if index == 0 { "alpha".to_string() } else { "beta".to_string() },
                tenant: Some("atlas".to_string()),
            };
            definition.concurrency.per_queue = Some(2);
            definition.concurrency.per_tenant = Some(1);
            definition
        })
        .collect(),
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: 200_000,
        manual_requests: vec![ManualSubmissionRequest {
            request_id: "manual-001".to_string(),
            schedule_id: "catalog-beta".to_string(),
            requested_unix_ms: 190_000,
            arguments: BTreeMap::new(),
        }],
        ..ScheduleEvaluationInputs::default()
    };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "catalog-alpha".to_string(),
            dag_name: "atlas.catalog-alpha".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity {
                queue_name: "alpha".to_string(),
                tenant: Some("atlas".to_string()),
            },
            priority: PriorityClass::Standard,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: 180_000,
            created_unix_ms: 180_000,
            run_id: "sched-catalog-alpha-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: "manual:catalog-alpha:manual-000".to_string(),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Running,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);

    assert_eq!(report.generated_requests.len(), 1);
    assert_eq!(report.generated_requests[0].schedule_id, "catalog-beta");
    assert!(report.queue_suppressions.is_empty());
}

#[test]
fn schedule_submit_cron_catch_up_honors_ranges_lists_and_steps() {
    let registry = ScheduleRegistry {
        definitions: vec![ScheduleDefinition {
            id: "weekday-window".to_string(),
            dag_name: "atlas.weekday-window".to_string(),
            dag_version_policy: "run-latest".to_string(),
            input_contract: BTreeMap::new(),
            input_bindings: BTreeMap::new(),
            trigger: TriggerSpec::Cron {
                expression: "*/15 9-10 * * Mon,Wed,Fri".to_string(),
                timezone: "UTC".to_string(),
            },
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            concurrency: ConcurrencyPolicyLayers {
                per_dag: Some(1),
                per_queue: Some(5),
                per_tenant: None,
                per_node_group: None,
            },
            catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 4 },
        }],
    };
    let last_requested =
        Utc.with_ymd_and_hms(2024, 1, 3, 9, 15, 0).single().expect("last requested");
    let scheduled_0930 = Utc.with_ymd_and_hms(2024, 1, 3, 9, 30, 0).single().expect("09:30");
    let scheduled_0945 = Utc.with_ymd_and_hms(2024, 1, 3, 9, 45, 0).single().expect("09:45");
    let scheduled_1000 = Utc.with_ymd_and_hms(2024, 1, 3, 10, 0, 0).single().expect("10:00");
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: u128::try_from(scheduled_1000.timestamp_millis()).expect("positive timestamp"),
        ..ScheduleEvaluationInputs::default()
    };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "weekday-window".to_string(),
            dag_name: "atlas.weekday-window".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: u128::try_from(last_requested.timestamp_millis())
                .expect("positive timestamp"),
            created_unix_ms: u128::try_from(last_requested.timestamp_millis())
                .expect("positive timestamp"),
            run_id: "sched-weekday-window-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: format!(
                "cron:weekday-window:{}",
                u128::try_from(last_requested.timestamp_millis()).expect("positive timestamp")
            ),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    let scheduled_slots = report
        .generated_requests
        .iter()
        .map(|request| request.requested_unix_ms)
        .collect::<Vec<_>>();

    assert_eq!(
        scheduled_slots,
        vec![
            u128::try_from(scheduled_0930.timestamp_millis()).expect("positive timestamp"),
            u128::try_from(scheduled_0945.timestamp_millis()).expect("positive timestamp"),
            u128::try_from(scheduled_1000.timestamp_millis()).expect("positive timestamp"),
        ]
    );
}

#[test]
fn schedule_submit_cron_catch_up_preserves_dst_fallback_duplicates() {
    let registry = ScheduleRegistry {
        definitions: vec![ScheduleDefinition {
            id: "dst-fallback".to_string(),
            dag_name: "atlas.dst-fallback".to_string(),
            dag_version_policy: "run-latest".to_string(),
            input_contract: BTreeMap::new(),
            input_bindings: BTreeMap::new(),
            trigger: TriggerSpec::Cron {
                expression: "30 1 * * *".to_string(),
                timezone: "America/New_York".to_string(),
            },
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            concurrency: ConcurrencyPolicyLayers {
                per_dag: Some(1),
                per_queue: Some(5),
                per_tenant: None,
                per_node_group: None,
            },
            catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 3 },
        }],
    };
    let last_requested =
        New_York.with_ymd_and_hms(2024, 11, 2, 1, 30, 0).single().expect("prior day");
    let LocalResult::Ambiguous(first, second) = New_York.with_ymd_and_hms(2024, 11, 3, 1, 30, 0)
    else {
        panic!("expected ambiguous dst fallback instant");
    };
    let inputs = ScheduleEvaluationInputs {
        now_unix_ms: u128::try_from(second.timestamp_millis()).expect("positive timestamp"),
        ..ScheduleEvaluationInputs::default()
    };
    let existing = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "dst-fallback".to_string(),
            dag_name: "atlas.dst-fallback".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: u128::try_from(last_requested.timestamp_millis())
                .expect("positive timestamp"),
            created_unix_ms: u128::try_from(last_requested.timestamp_millis())
                .expect("positive timestamp"),
            run_id: "sched-dst-fallback-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: format!(
                "cron:dst-fallback:{}",
                u128::try_from(last_requested.timestamp_millis()).expect("positive timestamp")
            ),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Pending,
            starvation_ticks: 0,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    let scheduled_slots = report
        .generated_requests
        .iter()
        .map(|request| request.requested_unix_ms)
        .collect::<Vec<_>>();

    assert_eq!(
        scheduled_slots,
        vec![
            u128::try_from(first.timestamp_millis()).expect("positive timestamp"),
            u128::try_from(second.timestamp_millis()).expect("positive timestamp"),
        ]
    );
}

#[test]
fn schedule_queue_state_reports_active_runs_and_available_slots() {
    let mut definition = schedule_definition("catalog-sync", TriggerSpec::Manual);
    definition.queue.tenant = Some("atlas".to_string());
    definition.concurrency.per_queue = Some(2);
    definition.concurrency.per_tenant = Some(2);
    let registry = ScheduleRegistry { definitions: vec![definition] };
    let ledger = ScheduleSubmissionLedger {
        entries: vec![
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("atlas".to_string()),
                },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 101,
                created_unix_ms: 101,
                run_id: "sched-atlas-pending".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:atlas-pending".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("atlas".to_string()),
                },
                priority: PriorityClass::Standard,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 102,
                created_unix_ms: 102,
                run_id: "sched-atlas-running".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:atlas-running".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Running,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("zeus".to_string()),
                },
                priority: PriorityClass::Low,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 103,
                created_unix_ms: 103,
                run_id: "sched-zeus-running".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:zeus-running".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Running,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("zeus".to_string()),
                },
                priority: PriorityClass::Low,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 104,
                created_unix_ms: 104,
                run_id: "sched-zeus-completed".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:zeus-completed".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Completed,
                starvation_ticks: 0,
            },
        ],
    };

    let state = build_schedule_queue_state(&registry, &ledger).expect("queue state");

    assert_eq!(state.queues.len(), 1);
    let queue = &state.queues[0];
    assert_eq!(queue.queue_name, "default");
    assert_eq!(queue.per_queue_cap, 2);
    assert_eq!(queue.active_runs, 3);
    assert_eq!(queue.available_slots, 0);
    assert_eq!(queue.runs.len(), 3);
    assert_eq!(queue.runs[0].run_id, "sched-atlas-pending");
    assert_eq!(queue.runs[0].starvation_ticks, 0);
    assert_eq!(queue.tenants.len(), 2);
    assert_eq!(queue.tenants[0].tenant, "atlas");
    assert_eq!(queue.tenants[0].active_runs, 2);
    assert_eq!(queue.tenants[0].available_slots, Some(0));
    assert_eq!(queue.tenants[1].tenant, "zeus");
    assert_eq!(queue.tenants[1].active_runs, 1);
    assert_eq!(queue.tenants[1].available_slots, Some(1));
}

#[test]
fn schedule_queue_dispatch_respects_priority_classes() {
    let mut ledger = ScheduleSubmissionLedger {
        entries: vec![
            ScheduleSubmissionLedgerEntry {
                schedule_id: "low".to_string(),
                dag_name: "atlas.low".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::Low,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 101,
                created_unix_ms: 101,
                run_id: "run-low".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:low:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "critical".to_string(),
                dag_name: "atlas.critical".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::Critical,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 102,
                created_unix_ms: 102,
                run_id: "run-critical".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:critical:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "standard".to_string(),
                dag_name: "atlas.standard".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::Standard,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 103,
                created_unix_ms: 103,
                run_id: "run-standard".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:standard:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "high".to_string(),
                dag_name: "atlas.high".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 104,
                created_unix_ms: 104,
                run_id: "run-high".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:high:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
        ],
    };

    let report =
        dispatch_schedule_queue_runs(&mut ledger, 4, &SchedulePriorityDispatchPolicy::default());

    assert_eq!(
        report.dispatched_runs.iter().map(|run| run.schedule_id.as_str()).collect::<Vec<_>>(),
        vec!["critical", "high", "standard", "low"]
    );
    assert!(ledger.entries.iter().all(|entry| entry.status == ScheduleSubmissionStatus::Running));
}

#[test]
fn schedule_queue_dispatch_breaks_equal_priority_deterministically() {
    let mut ledger = ScheduleSubmissionLedger {
        entries: vec![
            ScheduleSubmissionLedgerEntry {
                schedule_id: "beta".to_string(),
                dag_name: "atlas.beta".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 201,
                created_unix_ms: 201,
                run_id: "run-b".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:beta:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "alpha".to_string(),
                dag_name: "atlas.alpha".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 200,
                created_unix_ms: 200,
                run_id: "run-a".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:alpha:001".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "alpha".to_string(),
                dag_name: "atlas.alpha".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 200,
                created_unix_ms: 200,
                run_id: "run-c".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:alpha:002".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
        ],
    };

    let report =
        dispatch_schedule_queue_runs(&mut ledger, 3, &SchedulePriorityDispatchPolicy::default());

    assert_eq!(
        report.dispatched_runs.iter().map(|run| run.run_id.as_str()).collect::<Vec<_>>(),
        vec!["run-a", "run-c", "run-b"]
    );
}

#[test]
fn schedule_queue_dispatch_promotes_starved_runs() {
    let mut ledger = ScheduleSubmissionLedger {
        entries: vec![
            ScheduleSubmissionLedgerEntry {
                schedule_id: "critical".to_string(),
                dag_name: "atlas.critical".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::Critical,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 300,
                created_unix_ms: 300,
                run_id: "run-critical".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:critical:002".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "low".to_string(),
                dag_name: "atlas.low".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
                priority: PriorityClass::Low,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 299,
                created_unix_ms: 299,
                run_id: "run-low".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:low:002".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 3,
            },
        ],
    };

    let report = dispatch_schedule_queue_runs(
        &mut ledger,
        1,
        &SchedulePriorityDispatchPolicy {
            weights: WeightedPriorityPolicy::default(),
            starvation: StarvationPreventionPolicy {
                max_ticks_without_dispatch: 3,
                priority_boost_after_ticks: 1,
            },
        },
    );

    assert_eq!(report.dispatched_runs[0].run_id, "run-low");
    assert!(report.dispatched_runs[0].starvation_guard_applied);
    let by_run_id = ledger
        .entries
        .iter()
        .map(|entry| (entry.run_id.as_str(), entry))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(by_run_id["run-critical"].starvation_ticks, 1);
    assert_eq!(by_run_id["run-critical"].status, ScheduleSubmissionStatus::Pending);
    assert_eq!(by_run_id["run-low"].starvation_ticks, 0);
    assert_eq!(by_run_id["run-low"].status, ScheduleSubmissionStatus::Running);
}

#[test]
fn schedule_submission_status_updates_change_active_queue_state() {
    let registry = ScheduleRegistry {
        definitions: vec![schedule_definition("catalog-sync", TriggerSpec::Manual)],
    };
    let mut ledger = ScheduleSubmissionLedger {
        entries: vec![
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("atlas".to_string()),
                },
                priority: PriorityClass::Standard,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 101,
                created_unix_ms: 101,
                run_id: "sched-atlas-pending".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:atlas-pending".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Pending,
                starvation_ticks: 0,
            },
            ScheduleSubmissionLedgerEntry {
                schedule_id: "catalog-sync".to_string(),
                dag_name: "atlas.catalog-sync".to_string(),
                dag_version_policy: "run-latest".to_string(),
                queue: QueueIdentity {
                    queue_name: "default".to_string(),
                    tenant: Some("atlas".to_string()),
                },
                priority: PriorityClass::High,
                graph_inputs: BTreeMap::new(),
                requested_unix_ms: 102,
                created_unix_ms: 102,
                run_id: "sched-atlas-running".to_string(),
                trigger_kind: SubmissionTriggerKind::Manual,
                dedupe_key: "manual:catalog-sync:atlas-running".to_string(),
                event_lineage: None,
                status: ScheduleSubmissionStatus::Running,
                starvation_ticks: 0,
            },
        ],
    };

    apply_submission_status_updates(
        &mut ledger,
        &[
            ScheduleSubmissionStatusUpdate {
                run_id: "sched-atlas-running".to_string(),
                status: ScheduleSubmissionStatus::Completed,
                updated_unix_ms: 300,
            },
            ScheduleSubmissionStatusUpdate {
                run_id: "sched-atlas-pending".to_string(),
                status: ScheduleSubmissionStatus::Running,
                updated_unix_ms: 200,
            },
        ],
    )
    .expect("apply status updates");

    let state = build_schedule_queue_state(&registry, &ledger).expect("queue state");

    assert_eq!(state.queues[0].active_runs, 1);
    assert_eq!(state.queues[0].tenants[0].active_runs, 1);
    assert_eq!(ledger.entries[0].run_id, "sched-atlas-pending");
    assert_eq!(ledger.entries[0].status, ScheduleSubmissionStatus::Running);
    assert_eq!(ledger.entries[1].run_id, "sched-atlas-running");
    assert_eq!(ledger.entries[1].status, ScheduleSubmissionStatus::Completed);
}

#[test]
fn schedule_submission_status_updates_reject_invalid_transitions() {
    let mut ledger = ScheduleSubmissionLedger {
        entries: vec![ScheduleSubmissionLedgerEntry {
            schedule_id: "catalog-sync".to_string(),
            dag_name: "atlas.catalog-sync".to_string(),
            dag_version_policy: "run-latest".to_string(),
            queue: QueueIdentity {
                queue_name: "default".to_string(),
                tenant: Some("atlas".to_string()),
            },
            priority: PriorityClass::Standard,
            graph_inputs: BTreeMap::new(),
            requested_unix_ms: 101,
            created_unix_ms: 101,
            run_id: "sched-atlas-completed".to_string(),
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: "manual:catalog-sync:atlas-completed".to_string(),
            event_lineage: None,
            status: ScheduleSubmissionStatus::Completed,
            starvation_ticks: 0,
        }],
    };

    let error = apply_submission_status_updates(
        &mut ledger,
        &[ScheduleSubmissionStatusUpdate {
            run_id: "sched-atlas-completed".to_string(),
            status: ScheduleSubmissionStatus::Running,
            updated_unix_ms: 200,
        }],
    )
    .unwrap_err();

    assert!(error.contains("cannot transition"));
}

#[test]
fn schedule_validate_rejects_conflicting_queue_caps_for_same_queue() {
    let mut primary = schedule_definition("catalog-primary", TriggerSpec::Manual);
    primary.queue = QueueIdentity { queue_name: "catalog".to_string(), tenant: None };
    primary.concurrency.per_queue = Some(2);
    let mut replica = schedule_definition("catalog-replica", TriggerSpec::Manual);
    replica.queue = QueueIdentity { queue_name: "catalog".to_string(), tenant: None };
    replica.concurrency.per_queue = Some(4);

    let error =
        validate_schedule_registry(&ScheduleRegistry { definitions: vec![primary, replica] })
            .unwrap_err();

    assert!(error.contains("conflicting per_queue caps"));
}

#[test]
fn schedule_submission_ledger_defaults_legacy_queue_fields() {
    let ledger: ScheduleSubmissionLedger = serde_json::from_str(
        r#"{
          "entries": [
            {
              "schedule_id": "catalog-sync",
              "dag_name": "atlas.catalog-sync",
              "dag_version_policy": "run-latest",
              "graph_inputs": {},
              "requested_unix_ms": 101,
              "created_unix_ms": 101,
              "run_id": "sched-legacy",
              "trigger_kind": "manual",
              "dedupe_key": "manual:catalog-sync:legacy",
              "status": "Pending"
            }
          ]
        }"#,
    )
    .expect("parse legacy ledger");

    assert_eq!(ledger.entries[0].queue.queue_name, "default");
    assert_eq!(ledger.entries[0].queue.tenant, None);
    assert_eq!(ledger.entries[0].priority, PriorityClass::Standard);
    assert_eq!(ledger.entries[0].starvation_ticks, 0);
}

fn schedule_definition(id: &str, trigger: TriggerSpec) -> ScheduleDefinition {
    ScheduleDefinition {
        id: id.to_string(),
        dag_name: format!("atlas.{id}"),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger,
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(10),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    }
}

fn schedule_input_contract() -> BTreeMap<String, GraphInputSpec> {
    serde_json::from_value(serde_json::json!({
        "requested_at":{"type":"integer"},
        "manual_region":{"type":"string"},
        "event_tenant":{"type":"string"},
        "event_payload":{"type":"object"},
        "signal_tenant":{"type":"string"},
        "dependency_status":{"type":"string"},
        "dependency_run_id":{"type":"string"},
        "window_start":{"type":"integer"},
        "window_end":{"type":"integer"},
        "partition_key":{"type":"string"}
    }))
    .expect("input contract")
}
