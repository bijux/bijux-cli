use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Durable queue lease record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueLeaseRecordV1 {
    pub node_id: String,
    pub lease_owner: String,
    pub lease_epoch_ms: u64,
    pub lease_expires_at_ms: u64,
}

/// Durable queue attempt record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueAttemptRecordV1 {
    pub node_id: String,
    pub attempt: u32,
    pub started_at_ms: u64,
}

/// Durable scheduler decision record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerDecisionRecordV1 {
    pub node_id: String,
    pub decision: String,
    pub decided_at_ms: u64,
}

/// Durable run queue snapshot.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableRunQueueSnapshotV1 {
    pub run_id: String,
    pub admitted_nodes: Vec<String>,
    pub pending_nodes: Vec<String>,
    pub completed_nodes: Vec<String>,
    pub leases: Vec<QueueLeaseRecordV1>,
    pub attempts: Vec<QueueAttemptRecordV1>,
    pub scheduler_decisions: Vec<SchedulerDecisionRecordV1>,
}

/// Validate queue durability invariants needed for restart-safe scheduling.
pub fn validate_durable_run_queue_snapshot(
    snapshot: &DurableRunQueueSnapshotV1,
) -> Result<(), String> {
    if snapshot.run_id.trim().is_empty() {
        return Err("durable queue snapshot must include run_id".to_string());
    }
    let completed = snapshot.completed_nodes.iter().cloned().collect::<BTreeSet<_>>();
    let admitted = snapshot.admitted_nodes.iter().cloned().collect::<BTreeSet<_>>();
    let pending = snapshot.pending_nodes.iter().cloned().collect::<BTreeSet<_>>();

    let overlap = completed.intersection(&admitted).next().cloned().or_else(|| {
        completed.intersection(&pending).next().cloned()
    });
    if let Some(node_id) = overlap {
        return Err(format!(
            "completed node {} must not be re-admitted or pending after restart",
            node_id
        ));
    }

    let mut lease_nodes = BTreeSet::new();
    for lease in &snapshot.leases {
        if lease.node_id.trim().is_empty() || lease.lease_owner.trim().is_empty() {
            return Err("lease records must include node_id and lease_owner".to_string());
        }
        if lease.lease_expires_at_ms <= lease.lease_epoch_ms {
            return Err(format!(
                "lease for node {} must expire after its lease epoch",
                lease.node_id
            ));
        }
        if !lease_nodes.insert(lease.node_id.clone()) {
            return Err(format!("duplicate lease record for node {}", lease.node_id));
        }
    }

    let mut attempts: BTreeMap<&str, BTreeSet<u32>> = BTreeMap::new();
    for attempt in &snapshot.attempts {
        if attempt.node_id.trim().is_empty() || attempt.attempt == 0 {
            return Err("attempt records must include node_id and positive attempt".to_string());
        }
        attempts.entry(&attempt.node_id).or_default().insert(attempt.attempt);
    }
    for (node_id, attempts_for_node) in attempts {
        if attempts_for_node.len() > 1 && !admitted.contains(node_id) && !pending.contains(node_id) {
            return Err(format!(
                "node {} has multiple attempts but is neither admitted nor pending",
                node_id
            ));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        validate_durable_run_queue_snapshot, DurableRunQueueSnapshotV1, QueueAttemptRecordV1,
        QueueLeaseRecordV1, SchedulerDecisionRecordV1,
    };

    #[test]
    fn g131_durable_queue_snapshot_prevents_duplicate_restart_dispatch() {
        let valid = DurableRunQueueSnapshotV1 {
            run_id: "run-131".to_string(),
            admitted_nodes: vec!["node-b".to_string()],
            pending_nodes: vec!["node-c".to_string()],
            completed_nodes: vec!["node-a".to_string()],
            leases: vec![QueueLeaseRecordV1 {
                node_id: "node-b".to_string(),
                lease_owner: "worker-1".to_string(),
                lease_epoch_ms: 1_000,
                lease_expires_at_ms: 2_000,
            }],
            attempts: vec![
                QueueAttemptRecordV1 {
                    node_id: "node-a".to_string(),
                    attempt: 1,
                    started_at_ms: 900,
                },
                QueueAttemptRecordV1 {
                    node_id: "node-b".to_string(),
                    attempt: 2,
                    started_at_ms: 1_500,
                },
            ],
            scheduler_decisions: vec![SchedulerDecisionRecordV1 {
                node_id: "node-b".to_string(),
                decision: "queued".to_string(),
                decided_at_ms: 1_500,
            }],
        };
        validate_durable_run_queue_snapshot(&valid).expect("valid durable queue");

        let invalid = DurableRunQueueSnapshotV1 {
            completed_nodes: vec!["node-a".to_string()],
            admitted_nodes: vec!["node-a".to_string()],
            ..valid
        };
        let error = validate_durable_run_queue_snapshot(&invalid).expect_err("must reject overlap");
        assert!(error.contains("must not be re-admitted or pending"));
    }
}
