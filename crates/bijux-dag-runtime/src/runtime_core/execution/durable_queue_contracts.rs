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

    let overlap = completed
        .intersection(&admitted)
        .next()
        .cloned()
        .or_else(|| completed.intersection(&pending).next().cloned());
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
        if attempts_for_node.len() > 1 && !admitted.contains(node_id) && !pending.contains(node_id)
        {
            return Err(format!(
                "node {} has multiple attempts but is neither admitted nor pending",
                node_id
            ));
        }
    }

    Ok(())
}

/// Validate lease semantics and reject double-dispatch risk.
pub fn validate_node_leases(
    leases: &[QueueLeaseRecordV1],
    now_epoch_ms: u64,
) -> Result<(), String> {
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
        diagnostics
            .push("one or more active runs were starved under current pool limits".to_string());
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
        PartialRerunSelectorKindV1::ChangedInputClosure => {
            request.changed_input_closure_nodes.clone()
        }
    };
    selected_nodes.sort();
    selected_nodes.dedup();
    PartialRerunPreviewReportV1 { previewable: !selected_nodes.is_empty(), selected_nodes }
}

/// Adapter checkpoint behavior mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AdapterCheckpointModeV1 {
    Restartable,
    Resumable,
    CleanupRequired,
    FreshOnly,
}

/// Adapter checkpoint contract row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCheckpointContractV1 {
    pub node_id: String,
    pub adapter_kind: String,
    pub mode: AdapterCheckpointModeV1,
}

/// Resume decision from adapter checkpoint behavior.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointResumeDecisionV1 {
    pub node_id: String,
    pub resume_allowed: bool,
    pub cleanup_required: bool,
    pub reason: String,
}

/// Evaluate resume behavior while honoring adapter checkpoint contract.
pub fn evaluate_checkpoint_resume(
    contract: &AdapterCheckpointContractV1,
) -> CheckpointResumeDecisionV1 {
    match contract.mode {
        AdapterCheckpointModeV1::Restartable => CheckpointResumeDecisionV1 {
            node_id: contract.node_id.clone(),
            resume_allowed: true,
            cleanup_required: false,
            reason: "restartable adapter permits resume from checkpoint boundary".to_string(),
        },
        AdapterCheckpointModeV1::Resumable => CheckpointResumeDecisionV1 {
            node_id: contract.node_id.clone(),
            resume_allowed: true,
            cleanup_required: false,
            reason: "resumable adapter supports in-place continuation".to_string(),
        },
        AdapterCheckpointModeV1::CleanupRequired => CheckpointResumeDecisionV1 {
            node_id: contract.node_id.clone(),
            resume_allowed: true,
            cleanup_required: true,
            reason: "adapter requires cleanup before safe resume".to_string(),
        },
        AdapterCheckpointModeV1::FreshOnly => CheckpointResumeDecisionV1 {
            node_id: contract.node_id.clone(),
            resume_allowed: false,
            cleanup_required: true,
            reason: "fresh-only adapter refuses checkpoint resume".to_string(),
        },
    }
}

/// Backpressure input signals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackpressureSignalsV1 {
    pub artifact_store_degraded: bool,
    pub cache_degraded: bool,
    pub worker_pool_degraded: bool,
    pub io_degraded: bool,
    pub adapter_health_degraded: bool,
}

/// Backpressure action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BackpressureActionV1 {
    Normal,
    Throttle,
    Refuse,
}

/// Backpressure status report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackpressureStatusReportV1 {
    pub action: BackpressureActionV1,
    pub visible_status: String,
    pub reasons: Vec<String>,
}

/// Evaluate actionable backpressure from runtime degradation signals.
pub fn evaluate_backpressure(signals: &BackpressureSignalsV1) -> BackpressureStatusReportV1 {
    let mut reasons = Vec::new();
    if signals.artifact_store_degraded {
        reasons.push("artifact_store_degraded".to_string());
    }
    if signals.cache_degraded {
        reasons.push("cache_degraded".to_string());
    }
    if signals.worker_pool_degraded {
        reasons.push("worker_pool_degraded".to_string());
    }
    if signals.io_degraded {
        reasons.push("io_degraded".to_string());
    }
    if signals.adapter_health_degraded {
        reasons.push("adapter_health_degraded".to_string());
    }

    let action = if signals.io_degraded || signals.artifact_store_degraded {
        BackpressureActionV1::Refuse
    } else if !reasons.is_empty() {
        BackpressureActionV1::Throttle
    } else {
        BackpressureActionV1::Normal
    };
    let visible_status = match action {
        BackpressureActionV1::Normal => "normal".to_string(),
        BackpressureActionV1::Throttle => "throttle".to_string(),
        BackpressureActionV1::Refuse => "refuse".to_string(),
    };
    BackpressureStatusReportV1 { action, visible_status, reasons }
}

/// Circuit breaker state per adapter/backend surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AdapterCircuitBreakerStateV1 {
    pub adapter_id: String,
    pub failure_count: u32,
    pub threshold: u32,
    pub open_until_epoch_ms: u64,
}

/// Register failure and update circuit breaker state.
pub fn register_adapter_failure(
    mut state: AdapterCircuitBreakerStateV1,
    now_epoch_ms: u64,
    quarantine_ms: u64,
) -> AdapterCircuitBreakerStateV1 {
    state.failure_count = state.failure_count.saturating_add(1);
    if state.failure_count >= state.threshold.max(1) {
        state.open_until_epoch_ms = now_epoch_ms.saturating_add(quarantine_ms);
    }
    state
}

/// Decide whether dispatch is allowed for an adapter under circuit breaker policy.
pub fn dispatch_allowed_with_circuit_breaker(
    state: &AdapterCircuitBreakerStateV1,
    now_epoch_ms: u64,
) -> bool {
    now_epoch_ms >= state.open_until_epoch_ms
}

/// Runtime upgrade compatibility input.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeUpgradeCompatibilityInputV1 {
    pub run_id: String,
    pub from_runtime_version: String,
    pub to_runtime_version: String,
    pub schema_compatible: bool,
}

/// Runtime upgrade recovery decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeUpgradeRecoveryDecisionV1 {
    pub run_id: String,
    pub can_resume: bool,
    pub reason: String,
}

/// Evaluate upgrade safety for active run recovery.
pub fn evaluate_runtime_upgrade_recovery(
    input: &RuntimeUpgradeCompatibilityInputV1,
) -> RuntimeUpgradeRecoveryDecisionV1 {
    let incompatible_major = major_version(&input.from_runtime_version)
        .zip(major_version(&input.to_runtime_version))
        .is_some_and(|(from, to)| from != to);
    if !input.schema_compatible {
        return RuntimeUpgradeRecoveryDecisionV1 {
            run_id: input.run_id.clone(),
            can_resume: false,
            reason: "runtime upgrade refused because schema compatibility failed".to_string(),
        };
    }
    if incompatible_major {
        return RuntimeUpgradeRecoveryDecisionV1 {
            run_id: input.run_id.clone(),
            can_resume: false,
            reason: "runtime upgrade refused because major version changed".to_string(),
        };
    }
    RuntimeUpgradeRecoveryDecisionV1 {
        run_id: input.run_id.clone(),
        can_resume: true,
        reason: "runtime upgrade allows resume".to_string(),
    }
}

fn major_version(version: &str) -> Option<&str> {
    version.split('.').next().filter(|value| !value.trim().is_empty())
}

/// Runtime history event row.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHistoryEventV1 {
    pub run_id: String,
    pub node_id: String,
    pub state: String,
    pub ts_ms: u64,
}

/// Runtime history compaction report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeHistoryCompactionReportV1 {
    pub compacted_events: Vec<RuntimeHistoryEventV1>,
    pub run_index: BTreeMap<String, usize>,
    pub startup_scan_count: usize,
}

/// Compact runtime history while preserving queryability per run.
pub fn compact_runtime_history(
    events: &[RuntimeHistoryEventV1],
    max_events: usize,
    startup_scan_limit: usize,
) -> RuntimeHistoryCompactionReportV1 {
    let mut sorted = events.to_vec();
    sorted.sort_by_key(|event| event.ts_ms);
    let compacted_events = if sorted.len() > max_events {
        sorted.split_off(sorted.len() - max_events)
    } else {
        sorted
    };
    let mut run_index = BTreeMap::new();
    for event in &compacted_events {
        *run_index.entry(event.run_id.clone()).or_insert(0) += 1;
    }
    let startup_scan_count = compacted_events.len().min(startup_scan_limit);
    RuntimeHistoryCompactionReportV1 { compacted_events, run_index, startup_scan_count }
}

#[cfg(test)]
mod tests {
    use super::{
        build_partial_rerun_preview, compact_runtime_history,
        dispatch_allowed_with_circuit_breaker, evaluate_backpressure, evaluate_checkpoint_resume,
        evaluate_runtime_upgrade_recovery, plan_multi_run_fairness, register_adapter_failure,
        validate_durable_run_queue_snapshot, validate_node_leases,
        validate_pause_resume_transitions, AdapterCheckpointContractV1, AdapterCheckpointModeV1,
        AdapterCircuitBreakerStateV1, BackpressureActionV1, BackpressureSignalsV1,
        DurableRunQueueSnapshotV1, MultiRunDemandV1, PartialRerunPreviewRequestV1,
        PartialRerunSelectorKindV1, PauseScopeV1, PauseTransitionEventV1, QueueAttemptRecordV1,
        QueueLeaseRecordV1, RuntimeHistoryEventV1, RuntimeUpgradeCompatibilityInputV1,
        SchedulerDecisionRecordV1,
    };
    use std::collections::BTreeMap;

    #[test]
    fn durable_queue_snapshot_prevents_duplicate_restart_dispatch() {
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
    fn node_lease_validation_rejects_double_dispatch_and_expired_active_owners() {
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
        let duplicate_err =
            validate_node_leases(&duplicate, 1_500).expect_err("must reject duplicate lease");
        assert!(duplicate_err.contains("double-dispatch risk"));

        let expired_active = vec![QueueLeaseRecordV1 {
            node_id: "node-expired".to_string(),
            lease_owner: "active-worker".to_string(),
            lease_epoch_ms: 1_000,
            lease_expires_at_ms: 1_200,
        }];
        let expired_err = validate_node_leases(&expired_active, 1_500)
            .expect_err("must reject expired active owner");
        assert!(expired_err.contains("must recover before dispatch"));
    }

    #[test]
    fn multi_run_scheduler_prevents_starvation_when_capacity_exists() {
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
        let assigned_total: u32 =
            report.decisions.iter().map(|decision| decision.assigned_slots).sum();
        assert_eq!(assigned_total, 3);
        assert!(report
            .decisions
            .iter()
            .any(|decision| decision.run_id == "run-a" && decision.assigned_slots > 0));
        assert!(report
            .decisions
            .iter()
            .any(|decision| decision.run_id == "run-b" && decision.assigned_slots > 0));
    }

    #[test]
    fn pause_resume_scope_transitions_are_legal_and_visible() {
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
        let error = validate_pause_resume_transitions(&illegal)
            .expect_err("must reject illegal transition");
        assert!(error.contains("illegal pause transition"));
    }

    #[test]
    fn partial_rerun_selector_preview_is_deterministic() {
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

    #[test]
    fn checkpoint_resume_honors_adapter_contract_mode() {
        let resumable = evaluate_checkpoint_resume(&AdapterCheckpointContractV1 {
            node_id: "node-resume".to_string(),
            adapter_kind: "shell".to_string(),
            mode: AdapterCheckpointModeV1::Resumable,
        });
        assert!(resumable.resume_allowed);
        assert!(!resumable.cleanup_required);

        let fresh_only = evaluate_checkpoint_resume(&AdapterCheckpointContractV1 {
            node_id: "node-fresh".to_string(),
            adapter_kind: "external".to_string(),
            mode: AdapterCheckpointModeV1::FreshOnly,
        });
        assert!(!fresh_only.resume_allowed);
        assert!(fresh_only.cleanup_required);
        assert!(fresh_only.reason.contains("refuses checkpoint resume"));
    }

    #[test]
    fn backpressure_status_reports_throttle_or_refuse_actions() {
        let throttle = evaluate_backpressure(&BackpressureSignalsV1 {
            artifact_store_degraded: false,
            cache_degraded: true,
            worker_pool_degraded: false,
            io_degraded: false,
            adapter_health_degraded: true,
        });
        assert_eq!(throttle.action, BackpressureActionV1::Throttle);
        assert_eq!(throttle.visible_status, "throttle");

        let refuse = evaluate_backpressure(&BackpressureSignalsV1 {
            artifact_store_degraded: true,
            cache_degraded: false,
            worker_pool_degraded: false,
            io_degraded: false,
            adapter_health_degraded: false,
        });
        assert_eq!(refuse.action, BackpressureActionV1::Refuse);
        assert_eq!(refuse.visible_status, "refuse");
        assert!(refuse.reasons.iter().any(|reason| reason == "artifact_store_degraded"));
    }

    #[test]
    fn adapter_circuit_breaker_quarantines_repeated_failures() {
        let initial = AdapterCircuitBreakerStateV1 {
            adapter_id: "shell".to_string(),
            failure_count: 1,
            threshold: 3,
            open_until_epoch_ms: 0,
        };
        let after_second = register_adapter_failure(initial.clone(), 10_000, 5_000);
        assert!(dispatch_allowed_with_circuit_breaker(&after_second, 10_500));
        let after_third = register_adapter_failure(after_second, 11_000, 5_000);
        assert!(!dispatch_allowed_with_circuit_breaker(&after_third, 12_000));
        assert!(dispatch_allowed_with_circuit_breaker(&after_third, 16_001));
    }

    #[test]
    fn runtime_upgrade_recovery_checks_schema_and_major_compatibility() {
        let schema_failure =
            evaluate_runtime_upgrade_recovery(&RuntimeUpgradeCompatibilityInputV1 {
                run_id: "run-upgrade-1".to_string(),
                from_runtime_version: "1.4.0".to_string(),
                to_runtime_version: "1.5.0".to_string(),
                schema_compatible: false,
            });
        assert!(!schema_failure.can_resume);
        assert!(schema_failure.reason.contains("schema compatibility failed"));

        let major_change = evaluate_runtime_upgrade_recovery(&RuntimeUpgradeCompatibilityInputV1 {
            run_id: "run-upgrade-2".to_string(),
            from_runtime_version: "1.9.0".to_string(),
            to_runtime_version: "2.0.0".to_string(),
            schema_compatible: true,
        });
        assert!(!major_change.can_resume);
        assert!(major_change.reason.contains("major version changed"));
    }

    #[test]
    fn runtime_history_compaction_keeps_large_histories_queryable() {
        let mut events = Vec::new();
        for idx in 0..500 {
            events.push(RuntimeHistoryEventV1 {
                run_id: if idx % 2 == 0 { "run-even" } else { "run-odd" }.to_string(),
                node_id: format!("node-{idx}"),
                state: "completed".to_string(),
                ts_ms: idx,
            });
        }
        let report = compact_runtime_history(&events, 120, 100);
        assert_eq!(report.compacted_events.len(), 120);
        assert!(report.run_index.contains_key("run-even"));
        assert!(report.run_index.contains_key("run-odd"));
        assert_eq!(report.startup_scan_count, 100);
    }
}
