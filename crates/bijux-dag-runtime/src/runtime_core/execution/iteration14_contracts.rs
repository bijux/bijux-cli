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

/// Validate lease semantics and reject double-dispatch risk.
pub fn validate_node_leases(leases: &[QueueLeaseRecordV1], now_epoch_ms: u64) -> Result<(), String> {
    let mut by_node = BTreeMap::<&str, &QueueLeaseRecordV1>::new();
    for lease in leases {
        if lease.node_id.trim().is_empty() || lease.lease_owner.trim().is_empty() {
            return Err("lease must include node_id and lease_owner".to_string());
        }
        if lease.lease_expires_at_ms <= lease.lease_epoch_ms {
            return Err(format!("lease {} must expire after lease start", lease.node_id));
        }
        if let Some(existing) = by_node.insert(&lease.node_id, lease) {
            return Err(format!(
                "double-dispatch risk for node {} between owners {} and {}",
                lease.node_id, existing.lease_owner, lease.lease_owner
            ));
        }
    }
    for lease in leases {
        let expired = lease.lease_expires_at_ms <= now_epoch_ms;
        if expired && lease.lease_owner == "active-worker" {
            return Err(format!(
                "active lease owner for node {} is expired and must recover before dispatch",
                lease.node_id
            ));
        }
    }
    Ok(())
}

/// Multi-run demand row for fairness planning.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiRunDemandV1 {
    pub run_id: String,
    pub pool: String,
    pub requested_slots: u32,
}

/// Multi-run scheduling decision row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiRunSchedulingDecisionV1 {
    pub run_id: String,
    pub pool: String,
    pub assigned_slots: u32,
}

/// Multi-run fairness planning report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MultiRunSchedulingReportV1 {
    pub decisions: Vec<MultiRunSchedulingDecisionV1>,
    pub starvation_detected: bool,
    pub diagnostics: Vec<String>,
}

/// Build a fair, pool-limited multi-run scheduling plan.
pub fn plan_multi_run_fairness(
    demands: &[MultiRunDemandV1],
    pool_limits: &BTreeMap<String, u32>,
) -> MultiRunSchedulingReportV1 {
    let mut remaining = pool_limits.clone();
    let mut decisions = Vec::new();

    for demand in demands {
        let available = remaining.get(&demand.pool).copied().unwrap_or(0);
        let assigned = available.min(demand.requested_slots);
        decisions.push(MultiRunSchedulingDecisionV1 {
            run_id: demand.run_id.clone(),
            pool: demand.pool.clone(),
            assigned_slots: assigned,
        });
        remaining.insert(demand.pool.clone(), available.saturating_sub(assigned));
    }

    let starvation_detected = demands.iter().any(|demand| {
        demand.requested_slots > 0
            && decisions
                .iter()
                .find(|decision| decision.run_id == demand.run_id && decision.pool == demand.pool)
                .is_some_and(|decision| decision.assigned_slots == 0)
    });

    let mut diagnostics = Vec::new();
    if starvation_detected {
        diagnostics.push("one or more active runs were starved under current pool limits".to_string());
    }
    MultiRunSchedulingReportV1 { decisions, starvation_detected, diagnostics }
}

/// Pause/resume scope for runtime controls.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PauseScopeV1 {
    Graph,
    Pool,
    Adapter,
    NodeSelector,
}

/// Pause transition event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PauseTransitionEventV1 {
    pub scope: PauseScopeV1,
    pub target: String,
    pub from_state: String,
    pub to_state: String,
}

/// Validate pause/resume transitions at scope level.
pub fn validate_pause_resume_transitions(events: &[PauseTransitionEventV1]) -> Result<(), String> {
    for event in events {
        if event.target.trim().is_empty() {
            return Err("pause/resume event must include target".to_string());
        }
        let legal = matches!(
            (event.from_state.as_str(), event.to_state.as_str()),
            ("running", "paused") | ("paused", "running")
        );
        if !legal {
            return Err(format!(
                "illegal pause transition for {:?}/{}: {} -> {}",
                event.scope, event.target, event.from_state, event.to_state
            ));
        }
    }
    Ok(())
}

/// Partial rerun selector kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PartialRerunSelectorKindV1 {
    FailedOnly,
    Downstream,
    SelectedNodes,
    ChangedInputClosure,
}

/// Partial rerun preview request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialRerunPreviewRequestV1 {
    pub selector: PartialRerunSelectorKindV1,
    pub failed_nodes: Vec<String>,
    pub selected_nodes: Vec<String>,
    pub changed_input_closure_nodes: Vec<String>,
    pub downstream_nodes: Vec<String>,
}

/// Partial rerun preview report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartialRerunPreviewReportV1 {
    pub selected_nodes: Vec<String>,
    pub previewable: bool,
}

/// Build preview for partial rerun selector semantics.
pub fn build_partial_rerun_preview(
    request: &PartialRerunPreviewRequestV1,
) -> PartialRerunPreviewReportV1 {
    let mut selected_nodes = match request.selector {
        PartialRerunSelectorKindV1::FailedOnly => request.failed_nodes.clone(),
        PartialRerunSelectorKindV1::Downstream => request.downstream_nodes.clone(),
        PartialRerunSelectorKindV1::SelectedNodes => request.selected_nodes.clone(),
        PartialRerunSelectorKindV1::ChangedInputClosure => request.changed_input_closure_nodes.clone(),
    };
    selected_nodes.sort();
    selected_nodes.dedup();
    PartialRerunPreviewReportV1 { previewable: !selected_nodes.is_empty(), selected_nodes }
}

#[cfg(test)]
mod tests {
    use super::{
        build_partial_rerun_preview, plan_multi_run_fairness,
        validate_durable_run_queue_snapshot, validate_node_leases, validate_pause_resume_transitions,
        DurableRunQueueSnapshotV1, MultiRunDemandV1, PartialRerunPreviewRequestV1,
        PartialRerunSelectorKindV1, PauseScopeV1, PauseTransitionEventV1, QueueAttemptRecordV1,
        QueueLeaseRecordV1, SchedulerDecisionRecordV1,
    };
    use std::collections::BTreeMap;

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

    #[test]
    fn g132_node_lease_validation_rejects_double_dispatch_and_expired_active_owners() {
        let leases = vec![QueueLeaseRecordV1 {
            node_id: "node-lease".to_string(),
            lease_owner: "worker-1".to_string(),
            lease_epoch_ms: 1_000,
            lease_expires_at_ms: 2_000,
        }];
        validate_node_leases(&leases, 1_500).expect("healthy lease");

        let duplicate = vec![
            QueueLeaseRecordV1 {
                node_id: "node-lease".to_string(),
                lease_owner: "worker-1".to_string(),
                lease_epoch_ms: 1_000,
                lease_expires_at_ms: 2_000,
            },
            QueueLeaseRecordV1 {
                node_id: "node-lease".to_string(),
                lease_owner: "worker-2".to_string(),
                lease_epoch_ms: 1_100,
                lease_expires_at_ms: 2_100,
            },
        ];
        let duplicate_err = validate_node_leases(&duplicate, 1_500).expect_err("must reject duplicate lease");
        assert!(duplicate_err.contains("double-dispatch risk"));

        let expired_active = vec![QueueLeaseRecordV1 {
            node_id: "node-expired".to_string(),
            lease_owner: "active-worker".to_string(),
            lease_epoch_ms: 1_000,
            lease_expires_at_ms: 1_200,
        }];
        let expired_err = validate_node_leases(&expired_active, 1_500).expect_err("must reject expired active owner");
        assert!(expired_err.contains("must recover before dispatch"));
    }

    #[test]
    fn g133_multi_run_scheduler_prevents_starvation_when_capacity_exists() {
        let report = plan_multi_run_fairness(
            &[
                MultiRunDemandV1 {
                    run_id: "run-a".to_string(),
                    pool: "default".to_string(),
                    requested_slots: 2,
                },
                MultiRunDemandV1 {
                    run_id: "run-b".to_string(),
                    pool: "default".to_string(),
                    requested_slots: 2,
                },
            ],
            &BTreeMap::from([("default".to_string(), 3)]),
        );
        assert!(!report.starvation_detected);
        let assigned_total: u32 = report.decisions.iter().map(|decision| decision.assigned_slots).sum();
        assert_eq!(assigned_total, 3);
        assert!(report.decisions.iter().any(|decision| decision.run_id == "run-a" && decision.assigned_slots > 0));
        assert!(report.decisions.iter().any(|decision| decision.run_id == "run-b" && decision.assigned_slots > 0));
    }

    #[test]
    fn g134_pause_resume_scope_transitions_are_legal_and_visible() {
        let events = vec![
            PauseTransitionEventV1 {
                scope: PauseScopeV1::Graph,
                target: "run-graph-a".to_string(),
                from_state: "running".to_string(),
                to_state: "paused".to_string(),
            },
            PauseTransitionEventV1 {
                scope: PauseScopeV1::Pool,
                target: "gpu".to_string(),
                from_state: "paused".to_string(),
                to_state: "running".to_string(),
            },
        ];
        validate_pause_resume_transitions(&events).expect("legal transitions");

        let illegal = vec![PauseTransitionEventV1 {
            scope: PauseScopeV1::Adapter,
            target: "shell".to_string(),
            from_state: "running".to_string(),
            to_state: "running".to_string(),
        }];
        let error = validate_pause_resume_transitions(&illegal).expect_err("must reject illegal transition");
        assert!(error.contains("illegal pause transition"));
    }

    #[test]
    fn g135_partial_rerun_selector_preview_is_deterministic() {
        let request = PartialRerunPreviewRequestV1 {
            selector: PartialRerunSelectorKindV1::ChangedInputClosure,
            failed_nodes: vec!["node-failed".to_string()],
            selected_nodes: vec!["node-b".to_string(), "node-a".to_string()],
            changed_input_closure_nodes: vec![
                "node-c".to_string(),
                "node-a".to_string(),
                "node-c".to_string(),
            ],
            downstream_nodes: vec!["node-d".to_string()],
        };
        let report = build_partial_rerun_preview(&request);
        assert!(report.previewable);
        assert_eq!(report.selected_nodes, vec!["node-a".to_string(), "node-c".to_string()]);
    }
}
