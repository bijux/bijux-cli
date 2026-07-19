use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use chrono::{LocalResult, TimeZone};
use chrono_tz::America::New_York;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::{
    deduplicate_trigger_events, detect_cron_conflicts, evaluate_sla_metrics, materialize_next_runs,
    weighted_priority_tie_break_order, CatchUpPolicy, ConcurrencyPolicyLayers, PriorityClass,
    QueueIdentity, ScheduleDefinition, ScheduledSubmission, TriggerSpec, WeightedPriorityPolicy,
};
use std::collections::BTreeMap;

fn cron_schedule(id: &str, expression: &str) -> ScheduleDefinition {
    cron_schedule_in_timezone(id, expression, "UTC")
}

fn cron_schedule_in_timezone(id: &str, expression: &str, timezone: &str) -> ScheduleDefinition {
    ScheduleDefinition {
        id: id.to_string(),
        dag_name: "dag.example".to_string(),
        dag_version_policy: "run-latest".to_string(),
        input_contract: BTreeMap::new(),
        input_bindings: BTreeMap::new(),
        trigger: TriggerSpec::Cron {
            expression: expression.to_string(),
            timezone: timezone.to_string(),
        },
        queue: QueueIdentity { queue_name: "default".to_string(), tenant: None },
        priority: PriorityClass::Standard,
        concurrency: ConcurrencyPolicyLayers {
            per_dag: Some(4),
            per_queue: Some(4),
            per_tenant: None,
            per_node_group: None,
        },
        catch_up: CatchUpPolicy { enabled: true, max_catch_up_runs: 10 },
    }
}

#[test]
fn cron_conflict_detection_groups_equal_expressions() {
    let defs = vec![
        cron_schedule("s1", "0 * * * *"),
        cron_schedule("s2", "0 * * * *"),
        cron_schedule("s3", "15 * * * *"),
    ];
    let conflicts = detect_cron_conflicts(&defs);
    assert_eq!(conflicts.len(), 1);
    assert_eq!(conflicts[0].schedule_ids, vec!["s1".to_string(), "s2".to_string()]);
    assert_eq!(conflicts[0].expression, "0 * * * *");
    assert_eq!(conflicts[0].timezone, "UTC");
}

#[test]
fn cron_conflict_detection_distinguishes_timezones() {
    let defs = vec![
        cron_schedule_in_timezone("utc", "0 2 * * *", "UTC"),
        cron_schedule_in_timezone("new-york", "0 2 * * *", "America/New_York"),
    ];

    assert!(detect_cron_conflicts(&defs).is_empty());
}

#[test]
fn weighted_priority_sort_is_deterministic() {
    let submissions = vec![
        ScheduledSubmission {
            schedule_id: "b".to_string(),
            run_id: "run-2".to_string(),
            created_unix_ms: 10,
            status: bijux_dag_runtime::ScheduleSubmissionStatus::Pending,
        },
        ScheduledSubmission {
            schedule_id: "a".to_string(),
            run_id: "run-1".to_string(),
            created_unix_ms: 10,
            status: bijux_dag_runtime::ScheduleSubmissionStatus::Pending,
        },
    ];
    let mut priorities = BTreeMap::new();
    priorities.insert("a".to_string(), PriorityClass::Critical);
    priorities.insert("b".to_string(), PriorityClass::Low);
    let ordered = weighted_priority_tie_break_order(
        submissions,
        &priorities,
        &WeightedPriorityPolicy {
            critical_weight: 100,
            high_weight: 50,
            standard_weight: 10,
            low_weight: 1,
        },
    );
    assert_eq!(ordered[0].schedule_id, "a");
}

#[test]
fn materialized_preview_yields_n_cron_runs() {
    let schedule = cron_schedule("s1", "0 * * * *");
    let preview = materialize_next_runs(&schedule, 1_000, 3);
    assert_eq!(preview.next_run_unix_ms.len(), 3);
    assert_eq!(preview.next_run_unix_ms[0], 3_600_000);
}

#[test]
fn materialized_preview_keeps_dst_fallback_duplicates() {
    let schedule = cron_schedule_in_timezone("dst-fallback", "30 1 * * *", "America/New_York");
    let start = New_York.with_ymd_and_hms(2024, 11, 3, 0, 0, 0).single().expect("dst start");
    let preview = materialize_next_runs(
        &schedule,
        u128::try_from(start.timestamp_millis()).expect("positive timestamp"),
        3,
    );

    let LocalResult::Ambiguous(first, second) = New_York.with_ymd_and_hms(2024, 11, 3, 1, 30, 0)
    else {
        panic!("expected ambiguous dst fallback instant");
    };
    let next_day = New_York.with_ymd_and_hms(2024, 11, 4, 1, 30, 0).single().expect("next day");

    assert_eq!(
        preview.next_run_unix_ms,
        vec![
            u128::try_from(first.timestamp_millis()).expect("positive timestamp"),
            u128::try_from(second.timestamp_millis()).expect("positive timestamp"),
            u128::try_from(next_day.timestamp_millis()).expect("positive timestamp"),
        ]
    );
}

#[test]
fn trigger_dedup_marks_repeated_keys() {
    let decisions = deduplicate_trigger_events(&[
        "evt-1".to_string(),
        "evt-1".to_string(),
        "evt-2".to_string(),
    ]);
    assert!(!decisions[0].deduplicated);
    assert!(decisions[1].deduplicated);
    assert!(!decisions[2].deduplicated);
}

#[test]
fn sla_metrics_counts_missed_expectations() {
    let metrics = evaluate_sla_metrics(&[(20, 10), (5, 10)], &[(50, 40), (39, 40)], 2, 1);
    assert_eq!(metrics.missed_expected_start, 1);
    assert_eq!(metrics.missed_expected_finish, 1);
    assert_eq!(metrics.queue_saturation_count, 2);
    assert_eq!(metrics.fairness_drift_count, 1);
}
