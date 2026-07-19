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

use bijux_dag_runtime::simulated_platform::{
    deduplicate_across_replicas, evaluate_ha_conformance, failover_recovery_passes,
    fence_allows_mutation, idempotent_run_creation, ordering_during_failover, DurableRunQueueEntry,
    ScheduleDedupRecord, SchedulerEpoch, SchedulerFenceToken, SchedulerRecoveryObjectives,
};
use std::collections::BTreeMap;

#[test]
fn idempotent_run_creation_and_replica_dedup_are_stable() {
    let mut dedup = BTreeMap::new();
    let first = idempotent_run_creation(&mut dedup, "k1", "run-1");
    let second = idempotent_run_creation(&mut dedup, "k1", "run-2");
    assert_eq!(first, "run-1");
    assert_eq!(second, "run-1");

    let existing = vec![ScheduleDedupRecord {
        dedup_key: "k1".to_string(),
        run_key: "run-1".to_string(),
        epoch: 1,
    }];
    let proposed =
        ScheduleDedupRecord { dedup_key: "k2".to_string(), run_key: "run-2".to_string(), epoch: 1 };
    assert!(deduplicate_across_replicas(&existing, &proposed));
}

#[test]
fn failover_ordering_and_fencing_contracts_hold() {
    let ordered = ordering_during_failover(vec![
        DurableRunQueueEntry {
            queue_key: "q".to_string(),
            tenant_id: None,
            schedule_id: "s2".to_string(),
            run_key: "run-b".to_string(),
            created_unix_ms: 20,
        },
        DurableRunQueueEntry {
            queue_key: "q".to_string(),
            tenant_id: None,
            schedule_id: "s1".to_string(),
            run_key: "run-a".to_string(),
            created_unix_ms: 10,
        },
    ]);
    assert_eq!(ordered[0].run_key, "run-a");

    let epoch = SchedulerEpoch { replica_id: "replica-a".to_string(), epoch: 3 };
    let token = SchedulerFenceToken {
        replica_id: "replica-a".to_string(),
        epoch: 3,
        token: "fence-3".to_string(),
    };
    assert!(fence_allows_mutation(&token, &epoch));
}

#[test]
fn recovery_objectives_and_conformance_reports_are_explicit() {
    let objectives =
        SchedulerRecoveryObjectives { cold_restart_rto_ms: 30_000, failover_rto_ms: 5_000 };
    assert!(failover_recovery_passes(25_000, 3_000, &objectives));

    let report = evaluate_ha_conformance(&["run-1".to_string(), "run-2".to_string()], true, true);
    assert!(report.no_duplicate_runs);
    assert!(report.failures.is_empty());
}
