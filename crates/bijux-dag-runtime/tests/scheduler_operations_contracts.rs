use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use chrono::{LocalResult, TimeZone, Utc};
use chrono_tz::America::New_York;
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
    deterministic_tick_order, evaluate_schedule_submissions, evaluate_sla_metrics,
    materialize_next_runs, run_batches, scheduler_debug_event_log, scheduler_invariants_hold,
    trace_event_count_by_category, BackfillThrottlingPolicy, CatchUpPolicy,
    ConcurrencyPolicyLayers, DependencyCompletionRecord, DependencyCounter,
    ManualSubmissionRequest, PriorityClass, QueueIdentity, ReadyNode, ReadyQueue, RunBatchPolicy,
    RuntimeAuditEvent, RuntimeConfig, ScheduleDefinition, ScheduleEvaluationInputs,
    ScheduleEventRecord, ScheduleRegistry, ScheduleSubmissionLedger, ScheduleSubmissionLedgerEntry,
    ScheduleSubmissionStatus, ScheduledSubmission, Selector, SelectorSet, SignalRecord,
    SubmissionTriggerKind, TriggerSpec,
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
                    on_status: "success".to_string(),
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
        }],
        events: vec![ScheduleEventRecord {
            event_id: "evt-001".to_string(),
            event_type: "dataset.ready".to_string(),
            source: "catalog".to_string(),
            occurred_unix_ms: 176_000,
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
            },
            ManualSubmissionRequest {
                request_id: "manual-001".to_string(),
                schedule_id: "manual-ops".to_string(),
                requested_unix_ms: 175_000,
            },
        ],
        events: vec![
            ScheduleEventRecord {
                event_id: "evt-001".to_string(),
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
                occurred_unix_ms: 176_000,
            },
            ScheduleEventRecord {
                event_id: "evt-001".to_string(),
                event_type: "dataset.ready".to_string(),
                source: "catalog".to_string(),
                occurred_unix_ms: 176_000,
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
            requested_unix_ms: 175_000,
            created_unix_ms: 170_000,
            run_id: "sched-manual-ops-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Manual,
            dedupe_key: "manual:manual-ops:manual-001".to_string(),
            status: ScheduleSubmissionStatus::Pending,
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
            requested_unix_ms: 2 * 60_000,
            created_unix_ms: 2 * 60_000,
            run_id: "sched-cron-minute-existing".to_string(),
            trigger_kind: SubmissionTriggerKind::Cron,
            dedupe_key: "cron:cron-minute:120000".to_string(),
            status: ScheduleSubmissionStatus::Pending,
        }],
    };

    let report = evaluate_schedule_submissions(&registry, &inputs, &existing);
    let scheduled_slots = report
        .generated_requests
        .iter()
        .map(|request| request.requested_unix_ms)
        .collect::<Vec<_>>();

    assert_eq!(scheduled_slots, vec![3 * 60_000, 4 * 60_000]);
}

#[test]
fn schedule_submit_cron_catch_up_honors_ranges_lists_and_steps() {
    let registry = ScheduleRegistry {
        definitions: vec![ScheduleDefinition {
            id: "weekday-window".to_string(),
            dag_name: "atlas.weekday-window".to_string(),
            dag_version_policy: "run-latest".to_string(),
            trigger: TriggerSpec::Cron {
                expression: "*/15 9-10 * * Mon,Wed,Fri".to_string(),
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
            status: ScheduleSubmissionStatus::Pending,
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
            trigger: TriggerSpec::Cron {
                expression: "30 1 * * *".to_string(),
                timezone: "America/New_York".to_string(),
            },
            queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
            priority: PriorityClass::Standard,
            concurrency: ConcurrencyPolicyLayers {
                per_dag: Some(1),
                per_queue: Some(2),
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
            status: ScheduleSubmissionStatus::Pending,
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

fn schedule_definition(id: &str, trigger: TriggerSpec) -> ScheduleDefinition {
    ScheduleDefinition {
        id: id.to_string(),
        dag_name: format!("atlas.{id}"),
        dag_version_policy: "run-latest".to_string(),
        trigger,
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(1),
            per_queue: Some(2),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: false, max_catch_up_runs: 0 },
    }
}
