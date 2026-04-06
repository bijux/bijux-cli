use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2::{Digest, Sha256};
use tempfile as _;
use thiserror as _;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    apply_backfill_throttling, build_plan, build_scheduler, deduplicate_trigger_events,
    deterministic_tick_order, evaluate_sla_metrics, materialize_next_runs, run_batches,
    scheduler_debug_event_log, scheduler_invariants_hold, trace_event_count_by_category,
    BackfillThrottlingPolicy, CatchUpPolicy, ConcurrencyPolicyLayers, DependencyCounter,
    PriorityClass, QueueIdentity, ReadyNode, ReadyQueue, RunBatchPolicy, RuntimeAuditEvent,
    RuntimeConfig, ScheduleDefinition, ScheduleSubmissionStatus, ScheduledSubmission, Selector,
    SelectorSet, TriggerSpec,
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
