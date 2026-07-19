use crate::simulated_platform::{
    cancellation_delivered_in_time, classify_heartbeat, is_duplicate_dispatch,
    normalize_status_events, recover_lost_lease, should_reassign, worker_alive, HeartbeatClass,
    HeartbeatSemantics, LivenessPolicy, RemoteStatusEvent, TaskLeaseSemantics, WorkLease,
    WorkerHeartbeat,
};
use crate::{
    cancel_batch_attempt, contract_retry_backoff_ms, default_forced_cleanup,
    duplicate_status_delivery_detected, evaluate_retry_decision, retry_observation,
    validate_task_contracts, BackoffStrategy, BatchAttemptState, BatchLifecycleEvent,
    ForcedCancellationCleanup, Graph, InterruptionClass, ManualInterventionRecord, NodeState,
    NodeTransition, OperatorRetryPolicy, ResumePolicy, RetryPolicyV2, RunPausePolicy, RunState,
    RunTransition, RuntimeConfig, RuntimeError, StateConsistencyReport, TaskIsolationMode,
};
use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::hash::{Hash, Hasher};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIsolationNodeReport {
    pub node_id: String,
    pub isolation_mode: TaskIsolationMode,
    pub forced_cleanup: ForcedCancellationCleanup,
    pub idempotency_mode: String,
    pub executor_surface: String,
    pub side_effects: Vec<String>,
    pub sandbox_guards: Vec<String>,
    pub risk_flags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIsolationReport {
    pub total_nodes: usize,
    pub isolation_counts: BTreeMap<String, usize>,
    pub executor_surfaces: BTreeSet<String>,
    pub nodes: Vec<ExecutionIsolationNodeReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyGuardSemanticsReport {
    pub guard: String,
    pub enforcement_mode: String,
    pub guarantee: String,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEnforcementSurfaceReport {
    pub executor_surface: String,
    pub isolation_mode: TaskIsolationMode,
    pub isolation_claim: String,
    pub guards: Vec<PolicyGuardSemanticsReport>,
    pub limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyEnforcementReport {
    pub surfaces: Vec<PolicyEnforcementSurfaceReport>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchKeyRecord {
    pub run_id: String,
    pub node_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchAuditReport {
    pub submitted_dispatches: usize,
    pub duplicate_dispatch_keys: Vec<String>,
    pub remote_status_duplicates: usize,
    pub normalized_remote_statuses: usize,
    pub duplicate_batch_delivery_detected: bool,
    pub idempotent_dispatch_guarantee: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RetryDecisionReport {
    pub node_id: String,
    pub failure_class: String,
    pub attempt: u32,
    pub max_attempts: u32,
    pub retryable: bool,
    pub retry_allowed: bool,
    pub reason: String,
    pub next_attempt: Option<u32>,
    pub backoff_strategy: String,
    pub base_backoff_ms: u64,
    pub deterministic_jitter_ms: u64,
    pub next_wait_ms: Option<u64>,
    pub timeout_retry_policy: String,
    pub retryable_exit_codes: Vec<i32>,
    pub matched_exit_code: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutAuditReport {
    pub node_id: String,
    pub queue_timeout_ms: Option<u64>,
    pub execution_timeout_ms: Option<u64>,
    pub total_budget_timeout_ms: Option<u64>,
    pub queue_triggered: bool,
    pub execution_triggered: bool,
    pub total_budget_triggered: bool,
    pub heartbeat_triggered: bool,
    pub sla_triggered: bool,
    pub primary_timeout: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatAuditReport {
    pub worker_id: String,
    pub heartbeat_class: HeartbeatClass,
    pub worker_alive: bool,
    pub should_reassign: bool,
    pub recoverable_lease_loss: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancellationAuditReport {
    pub isolation_mode: TaskIsolationMode,
    pub forced_cleanup: ForcedCancellationCleanup,
    pub delivered_in_time: bool,
    pub batch_cancel_recorded: bool,
    pub batch_cancelled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PauseResumeAuditReport {
    pub freeze_dispatch: bool,
    pub freeze_ready_queue: bool,
    pub preserve_running_nodes: bool,
    pub has_queued: bool,
    pub has_ready: bool,
    pub has_running: bool,
    pub interruption_class: String,
    pub resume_policy: String,
    pub recommended_action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManualInterventionAuditReport {
    pub operator: String,
    pub action: String,
    pub allowed: bool,
    pub reason_required: bool,
    pub audit_required: bool,
    pub next_manual_attempt: Option<u32>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransitionAuditReport {
    pub node_transition_errors: Vec<String>,
    pub run_transition_errors: Vec<String>,
    pub terminal_audit_events: Vec<crate::TransitionAuditEvent>,
    pub consistency: StateConsistencyReport,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventLogAuditReport {
    pub run_id: String,
    pub event_count: usize,
    pub malformed_events: usize,
    pub missing_required_events: Vec<String>,
    pub singleton_event_violations: Vec<String>,
    pub failure_roots: Vec<String>,
    pub index_entry_count: Option<usize>,
    pub index_in_sync: Option<bool>,
    pub timeline_summary: Option<crate::TimelineTextSummary>,
}

pub fn build_execution_isolation_report(
    graph: &Graph,
    options: &RuntimeConfig,
) -> Result<ExecutionIsolationReport, RuntimeError> {
    let mut isolation_counts = BTreeMap::new();
    let mut executor_surfaces = BTreeSet::new();
    let mut nodes = validate_task_contracts(graph, options)?
        .into_iter()
        .map(|contract| {
            let isolation_label = match contract.isolation_mode {
                TaskIsolationMode::InProcess => "in_process",
                TaskIsolationMode::Subprocess => "subprocess",
                TaskIsolationMode::Container => "container",
                TaskIsolationMode::ExternalAdapter => "external_adapter",
            };
            *isolation_counts.entry(isolation_label.to_string()).or_insert(0) += 1;

            let executor_surface = match contract.isolation_mode {
                TaskIsolationMode::InProcess => "inline-kernel",
                TaskIsolationMode::Subprocess => "local-subprocess",
                TaskIsolationMode::Container => "container-engine",
                TaskIsolationMode::ExternalAdapter => "remote-adapter",
            }
            .to_string();
            executor_surfaces.insert(executor_surface.clone());

            let mut sandbox_guards = Vec::new();
            if contract.sandbox_policy.deny_network {
                sandbox_guards.push("deny-network".to_string());
            }
            if contract.sandbox_policy.deny_env {
                sandbox_guards.push("deny-env".to_string());
            }
            if contract.sandbox_policy.deny_clock {
                sandbox_guards.push("deny-clock".to_string());
            }
            if contract.sandbox_policy.clean_env {
                sandbox_guards.push("clean-env".to_string());
            }

            let mut risk_flags = Vec::new();
            if contract.nondeterministic_allowed {
                risk_flags.push("nondeterministic".to_string());
            }
            if matches!(contract.isolation_mode, TaskIsolationMode::InProcess)
                && !contract.effects.is_empty()
            {
                risk_flags.push("side-effects-without-process-boundary".to_string());
            }
            if matches!(contract.isolation_mode, TaskIsolationMode::ExternalAdapter) {
                risk_flags.push("adapter-boundary".to_string());
            }

            ExecutionIsolationNodeReport {
                node_id: contract.node_id,
                isolation_mode: contract.isolation_mode.clone(),
                forced_cleanup: default_forced_cleanup(&contract.isolation_mode),
                idempotency_mode: format!("{:?}", contract.idempotency_mode).to_lowercase(),
                executor_surface,
                side_effects: contract
                    .effects
                    .iter()
                    .map(|effect| format!("{:?}", effect.effect).to_lowercase())
                    .collect(),
                sandbox_guards,
                risk_flags,
            }
        })
        .collect::<Vec<_>>();
    nodes.sort_by(|a, b| a.node_id.cmp(&b.node_id));

    Ok(ExecutionIsolationReport {
        total_nodes: nodes.len(),
        isolation_counts,
        executor_surfaces,
        nodes,
    })
}

pub fn build_policy_enforcement_report(
    graph: &Graph,
    options: &RuntimeConfig,
) -> Result<PolicyEnforcementReport, RuntimeError> {
    let mut surfaces = validate_task_contracts(graph, options)?
        .into_iter()
        .map(|contract| policy_enforcement_surface(&contract.isolation_mode, options))
        .collect::<Vec<_>>();
    surfaces.sort_by(|left, right| left.executor_surface.cmp(&right.executor_surface));
    surfaces.dedup_by(|left, right| {
        left.executor_surface == right.executor_surface
            && left.isolation_mode == right.isolation_mode
    });
    Ok(PolicyEnforcementReport { surfaces })
}

fn policy_enforcement_surface(
    isolation_mode: &TaskIsolationMode,
    options: &RuntimeConfig,
) -> PolicyEnforcementSurfaceReport {
    match isolation_mode {
        TaskIsolationMode::InProcess => PolicyEnforcementSurfaceReport {
            executor_surface: "inline-kernel".to_string(),
            isolation_mode: TaskIsolationMode::InProcess,
            isolation_claim: "no_process_boundary".to_string(),
            guards: vec![
                effect_gate_guard("deny-network", "network"),
                effect_gate_guard("deny-env", "environment"),
                effect_gate_guard("deny-clock", "clock"),
            ],
            limitations: vec![
                "inline execution does not create a subprocess boundary".to_string(),
                "host side effects remain visible inside the current process".to_string(),
            ],
        },
        TaskIsolationMode::Subprocess => PolicyEnforcementSurfaceReport {
            executor_surface: "local-subprocess".to_string(),
            isolation_mode: TaskIsolationMode::Subprocess,
            isolation_claim: "best_effort_process_boundary".to_string(),
            guards: vec![
                effect_gate_guard("deny-network", "network"),
                effect_gate_guard("deny-env", "environment"),
                effect_gate_guard("deny-clock", "clock"),
                clean_env_guard(),
            ],
            limitations: vec![
                "subprocess mode does not firewall network access".to_string(),
                "subprocess mode does not virtualize clocks or time syscalls".to_string(),
                "subprocess mode does not sandbox arbitrary filesystem reads".to_string(),
                "subprocess mode cannot prevent host-visible side effects after spawn".to_string(),
            ],
        },
        TaskIsolationMode::Container => PolicyEnforcementSurfaceReport {
            executor_surface: "container-engine".to_string(),
            isolation_mode: TaskIsolationMode::Container,
            isolation_claim: "container_runtime_boundary".to_string(),
            guards: vec![
                PolicyGuardSemanticsReport {
                    guard: "deny-network".to_string(),
                    enforcement_mode: "container_runtime_flag".to_string(),
                    guarantee: "passes network isolation flags to the container engine and fails closed when the engine cannot honor them".to_string(),
                    limitations: vec![
                        "network isolation semantics depend on the selected container engine".to_string(),
                        "container networking controls do not claim full host sandboxing".to_string(),
                    ],
                },
                container_image_reference_guard(
                    options.policy.container_image_reference_policy,
                ),
                effect_gate_guard("deny-env", "environment"),
                effect_gate_guard("deny-clock", "clock"),
                clean_env_guard(),
            ],
            limitations: vec![
                "container mode constrains declared mounts and environment but is not a virtual machine".to_string(),
                "clock denial remains declaration-based unless a stronger backend is added".to_string(),
            ],
        },
        TaskIsolationMode::ExternalAdapter => PolicyEnforcementSurfaceReport {
            executor_surface: "remote-adapter".to_string(),
            isolation_mode: TaskIsolationMode::ExternalAdapter,
            isolation_claim: "adapter_defined_boundary".to_string(),
            guards: vec![
                effect_gate_guard("deny-network", "network"),
                effect_gate_guard("deny-env", "environment"),
                effect_gate_guard("deny-clock", "clock"),
                clean_env_guard(),
            ],
            limitations: vec![
                "runtime policy is enforced before adapter handoff, then adapter behavior owns the remaining boundary".to_string(),
                "remote adapters may provide stronger isolation, but this runtime does not claim it without adapter-specific evidence".to_string(),
            ],
        },
    }
}

fn container_image_reference_guard(
    policy: crate::ContainerImageReferencePolicy,
) -> PolicyGuardSemanticsReport {
    match policy {
        crate::ContainerImageReferencePolicy::RequireDigest => PolicyGuardSemanticsReport {
            guard: "container-image-reference".to_string(),
            enforcement_mode: "reference_digest_gate".to_string(),
            guarantee:
                "refuses container nodes whose image reference is not pinned with an @sha256 digest before execution starts".to_string(),
            limitations: vec![
                "validates the declared image reference, not registry signatures or publisher trust".to_string(),
                "trace evidence still depends on the selected engine reporting image identity".to_string(),
            ],
        },
        crate::ContainerImageReferencePolicy::AllowUnpinned => PolicyGuardSemanticsReport {
            guard: "container-image-reference".to_string(),
            enforcement_mode: "operator_override".to_string(),
            guarantee:
                "permits unpinned container image references for this execution profile".to_string(),
            limitations: vec![
                "mutable tags weaken replay identity guarantees compared with digest-pinned references".to_string(),
                "trace evidence still depends on the selected engine reporting image identity".to_string(),
            ],
        },
    }
}

fn effect_gate_guard(guard: &str, effect: &str) -> PolicyGuardSemanticsReport {
    PolicyGuardSemanticsReport {
        guard: guard.to_string(),
        enforcement_mode: "declared_effect_gate".to_string(),
        guarantee: format!("refuses nodes that declare {effect} effects before execution starts"),
        limitations: vec![
            "depends on accurate effect declarations in the DAG".to_string(),
            "does not interpose on syscalls after the executor has started".to_string(),
        ],
    }
}

fn clean_env_guard() -> PolicyGuardSemanticsReport {
    PolicyGuardSemanticsReport {
        guard: "clean-env".to_string(),
        enforcement_mode: "environment_shaping".to_string(),
        guarantee: "starts executors with a stripped environment and optional allowlist"
            .to_string(),
        limitations: vec![
            "does not sandbox filesystem access".to_string(),
            "does not prevent subprocess side effects outside environment variables".to_string(),
        ],
    }
}

pub fn audit_dispatch_discipline(
    dispatches: &[DispatchKeyRecord],
    remote_status_events: &[RemoteStatusEvent],
    batch_events: &[BatchLifecycleEvent],
) -> DispatchAuditReport {
    let mut seen = BTreeSet::new();
    let mut duplicate_dispatch_keys = Vec::new();
    for dispatch in dispatches {
        if is_duplicate_dispatch(&mut seen, &dispatch.run_id, &dispatch.node_id) {
            duplicate_dispatch_keys.push(format!("{}:{}", dispatch.run_id, dispatch.node_id));
        }
    }

    let (normalized_remote_statuses, remote_duplicates) =
        normalize_status_events(remote_status_events);
    let duplicate_batch_delivery_detected = duplicate_status_delivery_detected(batch_events);
    let idempotent_dispatch_guarantee =
        duplicate_dispatch_keys.is_empty() && !duplicate_batch_delivery_detected;

    DispatchAuditReport {
        submitted_dispatches: dispatches.len(),
        duplicate_dispatch_keys,
        remote_status_duplicates: remote_duplicates.len(),
        normalized_remote_statuses: normalized_remote_statuses.len(),
        duplicate_batch_delivery_detected,
        idempotent_dispatch_guarantee,
    }
}

pub fn build_retry_decision_report(
    graph: &Graph,
    options: &RuntimeConfig,
    node_id: &str,
    attempt: u32,
    failure_class: &str,
    exit_code: Option<i32>,
) -> Result<RetryDecisionReport, RuntimeError> {
    let contracts = validate_task_contracts(graph, options)?;
    let contract = contracts
        .into_iter()
        .find(|contract| contract.node_id == node_id)
        .ok_or_else(|| RuntimeError::Executor(format!("unknown node '{node_id}'")))?;

    let decision = evaluate_retry_decision(
        &contract.node_id,
        &contract.retry_policy,
        attempt,
        &retry_observation(failure_class, None, exit_code),
    );
    let base_backoff_ms = contract_retry_backoff_ms(&contract.retry_policy, attempt);
    let deterministic_jitter_ms = deterministic_jitter(
        &contract.node_id,
        attempt,
        failure_class,
        contract.retry_policy.jitter_ms,
    );

    Ok(RetryDecisionReport {
        node_id: contract.node_id,
        failure_class: failure_class.to_string(),
        attempt,
        max_attempts: contract.retry_policy.max_attempts,
        retryable: decision.retryable,
        retry_allowed: decision.retry_allowed,
        reason: decision.reason,
        next_attempt: decision.retry_allowed.then_some(attempt.saturating_add(1)),
        backoff_strategy: format!("{:?}", contract.retry_policy.backoff_strategy).to_lowercase(),
        base_backoff_ms,
        deterministic_jitter_ms,
        next_wait_ms: decision
            .retry_allowed
            .then_some(base_backoff_ms.saturating_add(deterministic_jitter_ms)),
        timeout_retry_policy: format!("{:?}", contract.retry_policy.timeout_retry_policy)
            .to_lowercase(),
        retryable_exit_codes: contract.retry_policy.retryable_exit_codes,
        matched_exit_code: decision.matched_exit_code,
    })
}

pub fn build_timeout_audit_report(
    graph: &Graph,
    options: &RuntimeConfig,
    node_id: &str,
    queue_wait_ms: Option<u64>,
    execution_ms: Option<u64>,
    total_elapsed_ms: Option<u64>,
    heartbeat_gap_ms: Option<u64>,
    heartbeat_timeout_ms: Option<u64>,
    sla_timeout_ms: Option<u64>,
) -> Result<TimeoutAuditReport, RuntimeError> {
    let contracts = validate_task_contracts(graph, options)?;
    let contract = contracts
        .into_iter()
        .find(|contract| contract.node_id == node_id)
        .ok_or_else(|| RuntimeError::Executor(format!("unknown node '{node_id}'")))?;

    let queue_triggered = duration_exceeds(queue_wait_ms, contract.timeout_policy.queue_timeout_ms);
    let execution_triggered =
        duration_exceeds(execution_ms, contract.timeout_policy.execution_timeout_ms);
    let total_budget_triggered =
        duration_exceeds(total_elapsed_ms, contract.timeout_policy.total_budget_timeout_ms);
    let heartbeat_triggered = duration_exceeds(heartbeat_gap_ms, heartbeat_timeout_ms);
    let sla_triggered = duration_exceeds(total_elapsed_ms, sla_timeout_ms);

    let primary_timeout = if queue_triggered {
        Some("queue".to_string())
    } else if heartbeat_triggered {
        Some("heartbeat".to_string())
    } else if execution_triggered {
        Some("execution".to_string())
    } else if total_budget_triggered {
        Some("total_budget".to_string())
    } else if sla_triggered {
        Some("sla".to_string())
    } else {
        None
    };

    Ok(TimeoutAuditReport {
        node_id: contract.node_id,
        queue_timeout_ms: contract.timeout_policy.queue_timeout_ms,
        execution_timeout_ms: contract.timeout_policy.execution_timeout_ms,
        total_budget_timeout_ms: contract.timeout_policy.total_budget_timeout_ms,
        queue_triggered,
        execution_triggered,
        total_budget_triggered,
        heartbeat_triggered,
        sla_triggered,
        primary_timeout,
    })
}

pub fn build_heartbeat_audit_report(
    heartbeat: &WorkerHeartbeat,
    now_unix_ms: u128,
    liveness_policy: &LivenessPolicy,
    heartbeat_semantics: &HeartbeatSemantics,
    lease: Option<&WorkLease>,
    lease_semantics: Option<&TaskLeaseSemantics>,
) -> HeartbeatAuditReport {
    let heartbeat_class = classify_heartbeat(heartbeat, now_unix_ms, heartbeat_semantics);
    let worker_alive = worker_alive(heartbeat, now_unix_ms, liveness_policy);
    let should_reassign = lease.map(|lease| should_reassign(lease, now_unix_ms)).unwrap_or(false);
    let recoverable_lease_loss = lease
        .zip(lease_semantics)
        .map(|(lease, semantics)| recover_lost_lease(lease, now_unix_ms, semantics));

    HeartbeatAuditReport {
        worker_id: heartbeat.worker_id.clone(),
        heartbeat_class,
        worker_alive,
        should_reassign,
        recoverable_lease_loss,
    }
}

pub fn build_cancellation_audit_report(
    isolation_mode: TaskIsolationMode,
    issued_unix_ms: u128,
    delivered_unix_ms: u128,
    deadline_ms: u64,
    batch_state: Option<&BatchAttemptState>,
) -> CancellationAuditReport {
    let mut state = batch_state.cloned();
    if let Some(batch_state) = state.as_mut() {
        cancel_batch_attempt(batch_state);
    }

    let batch_cancel_recorded = state
        .as_ref()
        .map(|batch| batch.events.iter().any(|event| event.status == "cancel-requested"))
        .unwrap_or(false);
    let batch_cancelled = state.as_ref().map(|batch| batch.cancelled).unwrap_or(false);

    CancellationAuditReport {
        forced_cleanup: default_forced_cleanup(&isolation_mode),
        delivered_in_time: cancellation_delivered_in_time(
            issued_unix_ms,
            delivered_unix_ms,
            deadline_ms,
        ),
        isolation_mode,
        batch_cancel_recorded,
        batch_cancelled,
    }
}

pub fn build_pause_resume_audit_report(
    policy: &RunPausePolicy,
    queued_count: usize,
    ready_count: usize,
    running_count: usize,
    interruption_class: &InterruptionClass,
    resume_policy: &ResumePolicy,
) -> PauseResumeAuditReport {
    let state = crate::evaluate_pause_state(policy, queued_count, ready_count, running_count);
    PauseResumeAuditReport {
        freeze_dispatch: *state.get("freeze_dispatch").unwrap_or(&false),
        freeze_ready_queue: *state.get("freeze_ready_queue").unwrap_or(&false),
        preserve_running_nodes: *state.get("preserve_running_nodes").unwrap_or(&false),
        has_queued: *state.get("has_queued").unwrap_or(&false),
        has_ready: *state.get("has_ready").unwrap_or(&false),
        has_running: *state.get("has_running").unwrap_or(&false),
        interruption_class: format!("{:?}", interruption_class).to_lowercase(),
        resume_policy: format!("{:?}", resume_policy).to_lowercase(),
        recommended_action: recommend_resume_action(interruption_class, resume_policy).to_string(),
    }
}

pub fn build_manual_intervention_audit_report(
    record: &ManualInterventionRecord,
    policy: &OperatorRetryPolicy,
    manual_attempts_so_far: u32,
) -> ManualInterventionAuditReport {
    let mut notes = Vec::new();
    let action = record.action.trim().to_lowercase();
    let allowed_actions = ["approve", "skip", "retry", "mark-success"];
    let mut allowed = true;

    if record.operator.trim().is_empty() {
        notes.push("operator must be non-empty".to_string());
        allowed = false;
    }
    if !allowed_actions.contains(&action.as_str()) {
        notes.push("unsupported intervention action".to_string());
        allowed = false;
    }
    if policy.require_reason && record.reason.trim().is_empty() {
        notes.push("reason is required by policy".to_string());
        allowed = false;
    }
    if action == "mark-success"
        && record.node_id.as_deref().map(|node_id| node_id.trim().is_empty()).unwrap_or(true)
    {
        notes.push("mark-success requires a node_id".to_string());
        allowed = false;
    }
    if action == "retry" && manual_attempts_so_far >= policy.max_manual_attempts {
        notes.push("manual retry budget exhausted".to_string());
        allowed = false;
    }
    if policy.requires_audit_record && record.recorded_unix_ms == 0 {
        notes.push("audit timestamp must be recorded".to_string());
        allowed = false;
    }

    ManualInterventionAuditReport {
        operator: record.operator.clone(),
        action,
        allowed,
        reason_required: policy.require_reason,
        audit_required: policy.requires_audit_record,
        next_manual_attempt: (allowed && record.action == "retry")
            .then_some(manual_attempts_so_far.saturating_add(1)),
        notes,
    }
}

pub fn build_transition_audit_report(
    node_transitions: &[NodeTransition],
    run_transitions: &[RunTransition],
    final_run_state: RunState,
    final_node_states: &[NodeState],
    causal_failure_count: usize,
) -> TransitionAuditReport {
    let node_transition_errors = node_transitions
        .iter()
        .filter_map(|transition| crate::validate_node_transition(transition).err())
        .collect::<Vec<_>>();
    let run_transition_errors = run_transitions
        .iter()
        .filter_map(|transition| crate::validate_run_transition(transition).err())
        .collect::<Vec<_>>();
    let consistency = crate::verify_post_run_state_consistency(
        final_run_state,
        final_node_states,
        causal_failure_count,
    );
    let terminal_audit_events =
        crate::terminal_transition_audit_events(node_transitions, run_transitions);

    TransitionAuditReport {
        node_transition_errors,
        run_transition_errors,
        terminal_audit_events,
        consistency,
    }
}

pub fn audit_run_event_log(run_dir: &Path) -> Result<EventLogAuditReport, String> {
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(run_dir.join("manifest.json")).map_err(|err| err.to_string())?,
    )
    .map_err(|err| err.to_string())?;
    let run_id = manifest
        .get("run_id")
        .and_then(|value| value.as_str())
        .unwrap_or("unknown-run")
        .to_string();

    let raw =
        std::fs::read_to_string(run_dir.join("run.log.jsonl")).map_err(|err| err.to_string())?;
    let mut events = Vec::new();
    let mut malformed_events = 0usize;
    for line in raw.lines().filter(|line| !line.trim().is_empty()) {
        let value: serde_json::Value = match serde_json::from_str(line) {
            Ok(value) => value,
            Err(_) => {
                malformed_events += 1;
                continue;
            }
        };
        let Some(name) = value.get("event").and_then(|field| field.as_str()) else {
            malformed_events += 1;
            continue;
        };
        let Some(unix_ms) =
            value.get("ts").and_then(|field| field.as_u64()).map(|value| value as u128).or_else(
                || value.get("unix_ms").and_then(|field| field.as_u64()).map(|value| value as u128),
            )
        else {
            malformed_events += 1;
            continue;
        };
        events.push(crate::EventRecord {
            category: crate::category_from_runtime_event_name(name),
            name: name.to_string(),
            unix_ms,
            node_id: value.get("node_id").and_then(|field| field.as_str()).map(ToString::to_string),
            run_id: Some(run_id.clone()),
            details: value,
        });
    }

    let missing_required_events = crate::validate_required_event_names(&events);
    let singleton_names = ["run_started", "plan_built", "run_finished"];
    let singleton_event_violations = singleton_names
        .into_iter()
        .filter(|name| !crate::event_names_emitted_once(&events, &[*name]))
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let failure_roots = crate::summarize_failure_root_causes(&events);

    let (index_entry_count, index_in_sync) = match std::fs::read(run_dir.join("run-log.index.json"))
    {
        Ok(bytes) => {
            let index: Vec<serde_json::Value> =
                serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
            let in_sync = index.len() == events.len()
                && index.iter().zip(events.iter()).all(|(entry, event)| {
                    entry.get("event").and_then(|value| value.as_str()) == Some(event.name.as_str())
                });
            (Some(index.len()), Some(in_sync))
        }
        Err(_) => (None, None),
    };

    let timeline_summary = match std::fs::read(run_dir.join("observability.timeline.json")) {
        Ok(bytes) => {
            let timeline: crate::TimelineExport =
                serde_json::from_slice(&bytes).map_err(|err| err.to_string())?;
            Some(crate::render_timeline_text(&timeline))
        }
        Err(_) => None,
    };

    Ok(EventLogAuditReport {
        run_id,
        event_count: events.len(),
        malformed_events,
        missing_required_events,
        singleton_event_violations,
        failure_roots,
        index_entry_count,
        index_in_sync,
        timeline_summary,
    })
}

fn retry_policy_semantics(node_id: &str, policy: &RetryPolicyV2) -> crate::RetryPolicySemantics {
    let _ = node_id;
    crate::RetryPolicySemantics {
        max_attempts: policy.max_attempts,
        initial_backoff_ms: policy.backoff_ms,
        exponential: matches!(policy.backoff_strategy, BackoffStrategy::Exponential),
    }
}

fn deterministic_jitter(node_id: &str, attempt: u32, failure_class: &str, jitter_ms: u64) -> u64 {
    if jitter_ms == 0 {
        return 0;
    }
    let mut hasher = DefaultHasher::new();
    node_id.hash(&mut hasher);
    attempt.hash(&mut hasher);
    failure_class.hash(&mut hasher);
    hasher.finish() % jitter_ms.saturating_add(1)
}

fn duration_exceeds(observed_ms: Option<u64>, limit_ms: Option<u64>) -> bool {
    matches!((observed_ms, limit_ms), (Some(observed), Some(limit)) if observed > limit)
}

fn recommend_resume_action(
    interruption_class: &InterruptionClass,
    resume_policy: &ResumePolicy,
) -> &'static str {
    match (interruption_class, resume_policy) {
        (_, ResumePolicy::FailSafeStop) => "halt-and-repair",
        (InterruptionClass::CleanShutdown, ResumePolicy::Reattach) => "reattach-running-nodes",
        (_, ResumePolicy::RerunIncompleteNodes) => "rerun-incomplete-nodes",
        (InterruptionClass::ProcessCrash, ResumePolicy::VerifyAndContinue)
        | (InterruptionClass::WorkerLoss, ResumePolicy::VerifyAndContinue)
        | (InterruptionClass::BackendLoss, ResumePolicy::VerifyAndContinue) => {
            "verify-run-state-then-continue"
        }
        (_, ResumePolicy::Reattach) => "reattach-when-lease-state-is-intact",
        _ => "continue-under-operator-review",
    }
}

#[cfg(test)]
mod tests {
    use super::{
        audit_dispatch_discipline, build_execution_isolation_report,
        build_policy_enforcement_report, DispatchKeyRecord,
    };
    use crate::simulated_platform::{
        HeartbeatClass, HeartbeatSemantics, LivenessPolicy, TaskLeaseSemantics, WorkLease,
        WorkerHeartbeat,
    };
    use crate::{
        BatchAttemptState, BatchLifecycleEvent, ForcedCancellationCleanup, InterruptionClass,
        ManualInterventionRecord, NodeState, NodeTransition, OperatorRetryPolicy, ResumePolicy,
        RunPausePolicy, RunState, RunTransition, RuntimeConfig, TaskIsolationMode,
    };
    use bijux_dag_core::{Edge, FileOutput, Graph, GraphMeta, Node, NodeKind, ParamValue, PortRef};
    use std::collections::BTreeMap;

    fn graph_fixture() -> Graph {
        Graph {
            spec: "bijux-dag/v0.1".to_string(),
            meta: Some(GraphMeta {
                name: "runtime".to_string(),
                description: None,
                owners: Vec::new(),
                tags: Vec::new(),
            }),
            inputs: std::collections::BTreeMap::new(),
            nondeterminism_allowed: false,
            subgraphs: std::collections::BTreeMap::new(),
            subgraph_instances: Vec::new(),
            nodes: vec![
                Node {
                    id: "const1".to_string(),
                    kind: NodeKind::Const,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: Vec::new(),
                    outputs: vec![FileOutput::new("out".to_string(), "a/out".to_string())],
                    params: ParamValue::Object(BTreeMap::from([(
                        "value".to_string(),
                        ParamValue::Literal(serde_json::json!("1")),
                    )])),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: Vec::new(),
                    retry: Default::default(),
                    cache: Default::default(),
                    effects: Vec::new(),
                    env_allowlist: Vec::new(),
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
                Node {
                    id: "shell1".to_string(),
                    kind: NodeKind::Shell,
                    semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
                    inputs: vec!["in".to_string()],
                    outputs: vec![FileOutput::new("out".to_string(), "b/out".to_string())],
                    params: ParamValue::Object(BTreeMap::from([(
                        "argv".to_string(),
                        ParamValue::Array(vec![
                            ParamValue::Literal(serde_json::json!("/bin/sh")),
                            ParamValue::Literal(serde_json::json!("-c")),
                            ParamValue::Literal(serde_json::json!("true")),
                        ]),
                    )])),
                    container: None,
                    timeout_ms: None,
                    resources: None,
                    tags: Vec::new(),
                    retry: Default::default(),
                    cache: Default::default(),
                    effects: vec![bijux_dag_core::Effect::Filesystem],
                    env_allowlist: Vec::new(),
                    group: None,
                    trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
                    branch: None,
                    dynamic: None,
                },
            ],
            edges: vec![Edge {
                id: None,
                kind: bijux_dag_core::EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "const1".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "shell1".to_string(), port: "in".to_string() },
            }],
        }
    }

    #[test]
    fn isolation_report_distinguishes_inline_and_subprocess_nodes() {
        let report = build_execution_isolation_report(&graph_fixture(), &RuntimeConfig::default())
            .expect("report");
        assert_eq!(report.total_nodes, 2);
        assert!(report.isolation_counts.contains_key("in_process"));
        assert!(report.isolation_counts.contains_key("subprocess"));
    }

    #[test]
    fn policy_enforcement_report_marks_subprocess_as_best_effort() {
        let report = build_policy_enforcement_report(&graph_fixture(), &RuntimeConfig::default())
            .expect("policy report");
        let subprocess = report
            .surfaces
            .iter()
            .find(|surface| surface.executor_surface == "local-subprocess")
            .expect("subprocess surface");
        assert_eq!(subprocess.isolation_claim, "best_effort_process_boundary");
        assert!(subprocess
            .limitations
            .iter()
            .any(|entry| entry.contains("does not firewall network access")));
        assert!(subprocess.guards.iter().any(|guard| guard.guard == "deny-network"
            && guard.enforcement_mode == "declared_effect_gate"));
    }

    #[test]
    fn policy_enforcement_report_marks_container_network_as_runtime_enforced() {
        let mut graph = graph_fixture();
        graph.nodes.push(Node {
            id: "container1".to_string(),
            kind: NodeKind::Container,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: Vec::new(),
            outputs: vec![FileOutput::new("out".to_string(), "c/out".to_string())],
            params: ParamValue::default(),
            container: Some(bijux_dag_core::ContainerSpec {
                image: "alpine:3.19".to_string(),
                argv: vec!["echo".to_string(), "ok".to_string()],
                env_allowlist: Vec::new(),
                workdir: None,
                engine: "docker".to_string(),
            }),
            timeout_ms: None,
            resources: None,
            tags: Vec::new(),
            retry: Default::default(),
            cache: Default::default(),
            effects: vec![bijux_dag_core::Effect::Filesystem],
            env_allowlist: Vec::new(),
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        });
        let report = build_policy_enforcement_report(&graph, &RuntimeConfig::default())
            .expect("policy report");
        let container = report
            .surfaces
            .iter()
            .find(|surface| surface.executor_surface == "container-engine")
            .expect("container surface");
        assert_eq!(container.isolation_claim, "container_runtime_boundary");
        assert!(container.guards.iter().any(|guard| {
            guard.guard == "deny-network" && guard.enforcement_mode == "container_runtime_flag"
        }));
        assert!(container.guards.iter().any(|guard| {
            guard.guard == "container-image-reference"
                && guard.enforcement_mode == "reference_digest_gate"
        }));
    }

    #[test]
    fn policy_enforcement_report_reflects_unpinned_container_override() {
        let mut graph = graph_fixture();
        graph.nodes.push(Node {
            id: "container1".to_string(),
            kind: NodeKind::Container,
            semantic_kind: bijux_dag_core::SemanticNodeKind::Task,
            inputs: Vec::new(),
            outputs: vec![FileOutput::new("out".to_string(), "c/out".to_string())],
            params: ParamValue::default(),
            container: Some(bijux_dag_core::ContainerSpec {
                image: "alpine:3.19".to_string(),
                argv: vec!["echo".to_string(), "ok".to_string()],
                env_allowlist: Vec::new(),
                workdir: None,
                engine: "docker".to_string(),
            }),
            timeout_ms: None,
            resources: None,
            tags: Vec::new(),
            retry: Default::default(),
            cache: Default::default(),
            effects: vec![bijux_dag_core::Effect::Filesystem],
            env_allowlist: Vec::new(),
            group: None,
            trigger_rule: bijux_dag_core::TriggerRule::AllSuccess,
            branch: None,
            dynamic: None,
        });
        let report = build_policy_enforcement_report(
            &graph,
            &RuntimeConfig {
                policy: crate::PolicyConfig {
                    container_image_reference_policy:
                        crate::ContainerImageReferencePolicy::AllowUnpinned,
                    ..crate::PolicyConfig::default()
                },
                ..RuntimeConfig::default()
            },
        )
        .expect("policy report");
        let container = report
            .surfaces
            .iter()
            .find(|surface| surface.executor_surface == "container-engine")
            .expect("container surface");
        assert!(container.guards.iter().any(|guard| {
            guard.guard == "container-image-reference"
                && guard.enforcement_mode == "operator_override"
        }));
    }

    #[test]
    fn dispatch_audit_flags_duplicate_dispatch_keys() {
        let report = audit_dispatch_discipline(
            &[
                DispatchKeyRecord { run_id: "run-1".to_string(), node_id: "a".to_string() },
                DispatchKeyRecord { run_id: "run-1".to_string(), node_id: "a".to_string() },
            ],
            &[],
            &[BatchLifecycleEvent {
                scheduler_id: "scheduler".to_string(),
                status: "submitted".to_string(),
                unix_ms: 1,
            }],
        );
        assert!(!report.idempotent_dispatch_guarantee);
        assert_eq!(report.duplicate_dispatch_keys, vec!["run-1:a".to_string()]);
    }

    #[test]
    fn retry_report_uses_backoff_strategy_and_jitter() {
        let mut graph = graph_fixture();
        graph.nodes[1].retry.max_attempts = 4;
        graph.nodes[1].retry.backoff_ms = 10;
        graph.nodes[1].params = bijux_dag_core::ParamValue::Object(BTreeMap::from([
            (
                "argv".to_string(),
                bijux_dag_core::ParamValue::Array(vec![
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("true")),
                ]),
            ),
            (
                "retry_backoff_strategy".to_string(),
                bijux_dag_core::ParamValue::Literal(serde_json::json!("exponential")),
            ),
            (
                "retry_jitter_ms".to_string(),
                bijux_dag_core::ParamValue::Literal(serde_json::json!(7)),
            ),
            (
                "retryable_failure_classes".to_string(),
                bijux_dag_core::ParamValue::Array(vec![
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("execution_transient")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("artifact_transient")),
                ]),
            ),
        ]));

        let report = super::build_retry_decision_report(
            &graph,
            &RuntimeConfig::default(),
            "shell1",
            2,
            "artifact_transient",
            None,
        )
        .expect("report");
        assert!(report.retryable);
        assert!(report.retry_allowed);
        assert_eq!(report.reason, "retryable_failure_class_matched");
        assert_eq!(report.base_backoff_ms, 20);
        assert!(report.deterministic_jitter_ms <= 7);
        assert_eq!(report.next_attempt, Some(3));
    }

    #[test]
    fn linear_retry_backoff_waits_for_the_first_retry_window() {
        let mut graph = graph_fixture();
        graph.nodes[1].retry.max_attempts = 3;
        graph.nodes[1].retry.backoff_ms = 15;
        graph.nodes[1].params = bijux_dag_core::ParamValue::Object(BTreeMap::from([(
            "argv".to_string(),
            bijux_dag_core::ParamValue::Array(vec![
                bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                bijux_dag_core::ParamValue::Literal(serde_json::json!("true")),
            ]),
        )]));

        let report = super::build_retry_decision_report(
            &graph,
            &RuntimeConfig::default(),
            "shell1",
            1,
            "execution_transient",
            None,
        )
        .expect("report");
        assert_eq!(report.base_backoff_ms, 15);
    }

    #[test]
    fn retry_report_explains_exit_code_and_timeout_overrides() {
        let mut graph = graph_fixture();
        graph.nodes[1].retry.max_attempts = 2;
        graph.nodes[1].retry.backoff_ms = 10;
        graph.nodes[1].params = bijux_dag_core::ParamValue::Object(BTreeMap::from([
            (
                "argv".to_string(),
                bijux_dag_core::ParamValue::Array(vec![
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("true")),
                ]),
            ),
            (
                "timeout_retry_policy".to_string(),
                bijux_dag_core::ParamValue::Literal(serde_json::json!("never")),
            ),
            (
                "retryable_exit_codes".to_string(),
                bijux_dag_core::ParamValue::Array(vec![bijux_dag_core::ParamValue::Literal(
                    serde_json::json!(75),
                )]),
            ),
        ]));

        let exit_code_report = super::build_retry_decision_report(
            &graph,
            &RuntimeConfig::default(),
            "shell1",
            1,
            "execution",
            Some(75),
        )
        .expect("exit code report");
        assert!(exit_code_report.retry_allowed);
        assert_eq!(exit_code_report.reason, "retryable_exit_code_matched");
        assert_eq!(exit_code_report.matched_exit_code, Some(75));
        assert_eq!(exit_code_report.retryable_exit_codes, vec![75]);

        let timeout_report = super::build_retry_decision_report(
            &graph,
            &RuntimeConfig::default(),
            "shell1",
            1,
            "timeout",
            Some(124),
        )
        .expect("timeout report");
        assert!(!timeout_report.retryable);
        assert_eq!(timeout_report.reason, "timeout_retry_policy_denies_timeout_retry");
        assert_eq!(timeout_report.timeout_retry_policy, "never");
    }

    #[test]
    fn timeout_report_separates_queue_and_execution_timeouts() {
        let mut graph = graph_fixture();
        graph.nodes[1].timeout_ms = Some(40);
        graph.nodes[1].params = bijux_dag_core::ParamValue::Object(BTreeMap::from([
            (
                "argv".to_string(),
                bijux_dag_core::ParamValue::Array(vec![
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("/bin/sh")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("-c")),
                    bijux_dag_core::ParamValue::Literal(serde_json::json!("true")),
                ]),
            ),
            (
                "queue_timeout_ms".to_string(),
                bijux_dag_core::ParamValue::Literal(serde_json::json!(10)),
            ),
            (
                "total_budget_timeout_ms".to_string(),
                bijux_dag_core::ParamValue::Literal(serde_json::json!(80)),
            ),
        ]));

        let report = super::build_timeout_audit_report(
            &graph,
            &RuntimeConfig::default(),
            "shell1",
            Some(15),
            Some(50),
            Some(90),
            Some(5),
            Some(20),
            Some(100),
        )
        .expect("report");
        assert!(report.queue_triggered);
        assert!(report.execution_triggered);
        assert!(report.total_budget_triggered);
        assert_eq!(report.primary_timeout, Some("queue".to_string()));
    }

    #[test]
    fn heartbeat_report_distinguishes_delayed_and_recoverable_leases() {
        let report = super::build_heartbeat_audit_report(
            &WorkerHeartbeat {
                worker_id: "worker-a".to_string(),
                unix_ms: 1_000,
                inflight_nodes: vec!["node-a".to_string()],
            },
            2_200,
            &LivenessPolicy { heartbeat_timeout_ms: 1_500, grace_retries: 2 },
            &HeartbeatSemantics {
                interval_ms: 500,
                timeout_ms: 2_500,
                delayed_threshold_ms: 1_000,
            },
            Some(&WorkLease {
                lease_id: "lease-1".to_string(),
                run_id: "run-1".to_string(),
                node_id: "node-a".to_string(),
                worker_id: "worker-a".to_string(),
                expires_unix_ms: 1_700,
            }),
            Some(&TaskLeaseSemantics {
                lease_duration_ms: 2_000,
                renew_before_expiry_ms: 500,
                max_renewals: 2,
                recovery_grace_ms: 800,
            }),
        );
        assert_eq!(report.heartbeat_class, HeartbeatClass::Delayed);
        assert!(report.worker_alive);
        assert!(report.should_reassign);
        assert_eq!(report.recoverable_lease_loss, Some(true));
    }

    #[test]
    fn cancellation_report_tracks_delivery_and_batch_recording() {
        let report = super::build_cancellation_audit_report(
            TaskIsolationMode::Container,
            1_000,
            1_300,
            500,
            Some(&BatchAttemptState {
                metadata: crate::BatchJobMetadata {
                    scheduler_id: "scheduler".to_string(),
                    submission_time_unix_ms: 1,
                    run_id: "run-1".to_string(),
                    node_id: "node-a".to_string(),
                    attempt_id: "1".to_string(),
                    resource_request: "cpu=1".to_string(),
                    status_mapping: "sim".to_string(),
                },
                events: vec![BatchLifecycleEvent {
                    scheduler_id: "scheduler".to_string(),
                    status: "submitted".to_string(),
                    unix_ms: 1,
                }],
                cancelled: false,
            }),
        );
        assert!(report.delivered_in_time);
        assert!(report.batch_cancel_recorded);
        assert!(report.batch_cancelled);
        assert_eq!(report.forced_cleanup, ForcedCancellationCleanup::ImmediateTerminate);
    }

    #[test]
    fn pause_resume_report_recommends_state_verification_after_worker_loss() {
        let report = super::build_pause_resume_audit_report(
            &RunPausePolicy {
                mode: crate::RunPauseMode::PauseAllNewDispatch,
                preserve_running_nodes: true,
            },
            2,
            1,
            1,
            &InterruptionClass::WorkerLoss,
            &ResumePolicy::VerifyAndContinue,
        );
        assert!(report.freeze_dispatch);
        assert!(report.freeze_ready_queue);
        assert_eq!(report.recommended_action, "verify-run-state-then-continue");
    }

    #[test]
    fn manual_intervention_report_enforces_reason_and_retry_budget() {
        let report = super::build_manual_intervention_audit_report(
            &ManualInterventionRecord {
                run_id: "run-1".to_string(),
                node_id: Some("node-a".to_string()),
                operator: "operator-a".to_string(),
                action: "retry".to_string(),
                reason: "transient artifact outage".to_string(),
                recorded_unix_ms: 123,
            },
            &OperatorRetryPolicy {
                max_manual_attempts: 2,
                require_reason: true,
                requires_audit_record: true,
            },
            1,
        );
        assert!(report.allowed);
        assert_eq!(report.next_manual_attempt, Some(2));
    }

    #[test]
    fn transition_report_surfaces_invalid_run_trace_and_consistency_gaps() {
        let report = super::build_transition_audit_report(
            &[
                NodeTransition {
                    from: NodeState::Pending,
                    to: NodeState::Eligible,
                    cause: crate::TransitionCause::SchedulerEligible,
                },
                NodeTransition {
                    from: NodeState::Eligible,
                    to: NodeState::Queued,
                    cause: crate::TransitionCause::SchedulerQueued,
                },
                NodeTransition {
                    from: NodeState::Queued,
                    to: NodeState::Running,
                    cause: crate::TransitionCause::ExecutionStarted,
                },
                NodeTransition {
                    from: NodeState::Running,
                    to: NodeState::Success,
                    cause: crate::TransitionCause::ExecutionSucceeded,
                },
            ],
            &[RunTransition {
                from: RunState::Running,
                to: RunState::Failed,
                cause: crate::TransitionCause::ExecutionFailed,
            }],
            RunState::Failed,
            &[NodeState::Success],
            0,
        );
        assert!(report.node_transition_errors.is_empty());
        assert!(report.run_transition_errors.is_empty());
        assert!(!report.consistency.valid);
        assert!(!report.terminal_audit_events.is_empty());
    }

    #[test]
    fn event_log_audit_reconciles_log_index_and_timeline() {
        let dir = tempfile::tempdir().expect("tmp");
        std::fs::write(dir.path().join("manifest.json"), r#"{"run_id":"run-1"}"#)
            .expect("manifest");
        std::fs::write(
            dir.path().join("run.log.jsonl"),
            r#"{"event":"run_started","ts":1}
{"event":"plan_built","ts":2}
{"event":"node_ready","ts":3,"node_id":"n1"}
{"event":"node_scheduled","ts":4,"node_id":"n1"}
{"event":"node_started","ts":5,"node_id":"n1"}
{"event":"node_attempt_started","ts":6,"node_id":"n1"}
{"event":"node_attempt_finished","ts":7,"node_id":"n1"}
{"event":"node_finished","ts":8,"node_id":"n1","status":"failed","reason":"timeout"}
{"event":"run_finished","ts":9}"#,
        )
        .expect("run log");
        std::fs::write(
            dir.path().join("run-log.index.json"),
            r#"[
              {"event":"run_started"},
              {"event":"plan_built"},
              {"event":"node_ready"},
              {"event":"node_scheduled"},
              {"event":"node_started"},
              {"event":"node_attempt_started"},
              {"event":"node_attempt_finished"},
              {"event":"node_finished"},
              {"event":"run_finished"}
            ]"#,
        )
        .expect("index");
        std::fs::write(
            dir.path().join("observability.timeline.json"),
            r#"{"schema_version":"v0.1","entries":[{"unix_ms":1,"category":"start","label":"run","node_id":null}]}"#,
        )
        .expect("timeline");

        let report = super::audit_run_event_log(dir.path()).expect("audit");
        assert_eq!(report.run_id, "run-1");
        assert_eq!(report.event_count, 9);
        assert_eq!(report.malformed_events, 0);
        assert!(report.missing_required_events.is_empty());
        assert_eq!(report.index_in_sync, Some(true));
        assert!(report.timeline_summary.is_some());
    }
}
