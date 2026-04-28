use crate::commands::{DagCli, FleetCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::derive_autoscaling_hint;
use bijux_dag_runtime::simulated_platform::{
    cancellation_delivered_in_time, check_scheduler_admission,
    check_worker_version_compatibility, classify_heartbeat, validate_worker_identity, worker_alive,
    validate_task_lease_semantics, worker_pool_satisfies_capability_request, LivenessPolicy,
    HeartbeatClass, HeartbeatSemantics, MutualAuthDesignNote, PlacementHint, QueuePartition,
    ReassignmentRule, SchedulerScalingPlan, TaskLeaseSemantics, TenantConcurrencyQuota,
    TenantQueueIsolationPolicy, TenantSchedulerAdmission, TrustDomain, WorkLease,
    WorkerBootstrapTrustFlow, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity, WorkerPool,
    WorkerPoolCapabilityRequest, WorkerRegistration, WorkerVersionCompatibilityRule,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct RegistrationSimulation {
    registration: WorkerRegistration,
    heartbeat: WorkerHeartbeat,
    liveness_policy: LivenessPolicy,
    version_rule: WorkerVersionCompatibilityRule,
    now_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct RegistrationReport {
    worker_id: String,
    backend_kind: String,
    labels: std::collections::BTreeMap<String, String>,
    inflight_nodes: Vec<String>,
    alive: bool,
    version_compatible: bool,
    gaps: Vec<String>,
    registration_ready: bool,
}

#[derive(Debug, Deserialize)]
struct CapabilitySimulation {
    worker_id: String,
    capabilities: WorkerCapabilities,
    request: WorkerPoolCapabilityRequest,
    pool: WorkerPool,
    placement_hint: PlacementHint,
}

#[derive(Debug, Serialize)]
struct CapabilityReport {
    worker_id: String,
    pool_id: String,
    pool_class: String,
    worker_count: usize,
    capability_match: bool,
    preferred_pool_match: bool,
    preferred_label_match: bool,
    gaps: Vec<String>,
    capability_ready: bool,
}

#[derive(Debug, Deserialize)]
struct DrainSimulation {
    worker_id: String,
    heartbeat: WorkerHeartbeat,
    lease_semantics: TaskLeaseSemantics,
    #[serde(default)]
    leases: Vec<WorkLease>,
    draining_started_unix_ms: u128,
    now_unix_ms: u128,
    new_dispatch_blocked: bool,
    replacement_pool_ready: bool,
}

#[derive(Debug, Serialize)]
struct DrainReport {
    worker_id: String,
    inflight_nodes: usize,
    active_leases: usize,
    expired_leases: usize,
    recoverable_leases: usize,
    new_dispatch_blocked: bool,
    replacement_pool_ready: bool,
    gaps: Vec<String>,
    drain_ready: bool,
}

#[derive(Debug, Deserialize)]
struct AutoscaleSimulation {
    queue_partition: QueuePartition,
    scaling_plan: SchedulerScalingPlan,
    queue_depth: usize,
    dispatch_lag_seconds: u32,
    saturation_pct: u32,
    current_replicas: usize,
}

#[derive(Debug, Serialize)]
struct AutoscaleReport {
    queue_name: String,
    target_component: String,
    current_replicas: usize,
    recommended_replicas: usize,
    worker_count_declared: u32,
    sharding_key: String,
    gaps: Vec<String>,
    autoscale_ready: bool,
}

#[derive(Debug, Deserialize)]
struct WarmPoolSimulation {
    pool: WorkerPool,
    target_runtime_class: String,
    #[serde(default)]
    warm_worker_ids: Vec<String>,
    #[serde(default)]
    preloaded_profiles: Vec<String>,
    cold_start_ms: u64,
    warm_start_ms: u64,
    monthly_cost_estimate: f64,
    policy_id: String,
}

#[derive(Debug, Serialize)]
struct WarmPoolReport {
    pool_id: String,
    target_runtime_class: String,
    warm_worker_count: usize,
    preloaded_profiles: Vec<String>,
    startup_improvement_ms: u64,
    monthly_cost_estimate: f64,
    policy_id: String,
    gaps: Vec<String>,
    warm_pool_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IsolationSimulation {
    tenant_id: String,
    queue_policy: TenantQueueIsolationPolicy,
    quota: TenantConcurrencyQuota,
    scheduler_admission: TenantSchedulerAdmission,
    #[serde(default)]
    queue_partitions: Vec<QueuePartition>,
    queued_runs: usize,
    pending_dispatches: usize,
    #[serde(default)]
    observed_foreign_tenants: Vec<String>,
}

#[derive(Debug, Serialize)]
struct IsolationReport {
    tenant_id: String,
    isolated_queues: Vec<String>,
    scheduler_admitted: bool,
    hard_isolation: bool,
    foreign_tenants_observed: Vec<String>,
    gaps: Vec<String>,
    isolation_ready: bool,
}

#[derive(Debug, Deserialize)]
struct PreemptionSimulation {
    node_id: String,
    side_effect_class: String,
    checkpointing_supported: bool,
    preemptible_backend: bool,
    cancellation_issued_unix_ms: u128,
    cancellation_delivered_unix_ms: u128,
    cancellation_deadline_ms: u64,
    lease_semantics: TaskLeaseSemantics,
    reassignment_rule: ReassignmentRule,
}

#[derive(Debug, Serialize)]
struct PreemptionReport {
    node_id: String,
    side_effect_class: String,
    checkpointing_supported: bool,
    preemptible_backend: bool,
    cancellation_delivered_in_time: bool,
    preserve_attempt_lineage: bool,
    gaps: Vec<String>,
    preemption_ready: bool,
}

#[derive(Debug, Deserialize)]
struct TrustSimulation {
    identity: WorkerIdentity,
    bootstrap: WorkerBootstrapTrustFlow,
    trust_domain: TrustDomain,
    mutual_auth: MutualAuthDesignNote,
    enrollment_approved: bool,
    attested_image: bool,
}

#[derive(Debug, Serialize)]
struct TrustReport {
    worker_id: String,
    trust_domain: String,
    transport: String,
    enrollment_approved: bool,
    attested_image: bool,
    mutual_auth_required: bool,
    gaps: Vec<String>,
    trust_ready: bool,
}

#[derive(Debug, Deserialize)]
struct GossipSimulation {
    #[serde(default)]
    heartbeats: Vec<WorkerHeartbeat>,
    heartbeat_semantics: HeartbeatSemantics,
    now_unix_ms: u128,
    max_peer_fanout: u32,
    observed_peer_fanout: u32,
    #[serde(default)]
    authoritative_worker_ids: Vec<String>,
    #[serde(default)]
    gossip_worker_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
struct GossipReport {
    healthy_workers: usize,
    delayed_workers: usize,
    lost_workers: usize,
    authoritative_converged: bool,
    fanout_bounded: bool,
    gaps: Vec<String>,
    gossip_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn registration_payload(simulation: RegistrationSimulation) -> (serde_json::Value, bool) {
    let RegistrationSimulation {
        registration,
        heartbeat,
        liveness_policy,
        version_rule,
        now_unix_ms,
    } = simulation;
    let identity_valid = validate_worker_identity(&registration.identity).is_ok();
    let alive = worker_alive(&heartbeat, now_unix_ms, &liveness_policy);
    let version_compatible =
        check_worker_version_compatibility(&registration.identity.worker_version, &version_rule);
    let mut gaps = Vec::new();
    if !identity_valid {
        gaps.push("worker registration requires stable non-empty identity fields".to_string());
    }
    if registration.registered_unix_ms == 0 {
        gaps.push("worker registration must record a registration timestamp".to_string());
    }
    if heartbeat.worker_id != registration.identity.worker_id {
        gaps.push("worker heartbeat does not match the registered worker identity".to_string());
    }
    if !alive {
        gaps.push("worker liveness is stale under the declared heartbeat policy".to_string());
    }
    if !version_compatible {
        gaps.push("worker version is below the planner compatibility floor".to_string());
    }
    if registration.capabilities.cpu_capacity == 0 || registration.capabilities.memory_mb == 0 {
        gaps.push("worker registration must declare non-zero cpu and memory capacity".to_string());
    }
    let report = RegistrationReport {
        worker_id: registration.identity.worker_id,
        backend_kind: registration.identity.backend_kind,
        labels: registration.identity.labels,
        inflight_nodes: heartbeat.inflight_nodes,
        alive,
        version_compatible,
        registration_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.registration_ready;
    (serde_json::to_value(report).expect("registration report"), ok)
}

fn capability_payload(simulation: CapabilitySimulation) -> (serde_json::Value, bool) {
    let CapabilitySimulation { worker_id, capabilities, request, pool, placement_hint } =
        simulation;
    let capability_match = worker_pool_satisfies_capability_request(&capabilities, &request);
    let preferred_pool_match = placement_hint.preferred_pool == pool.pool_id;
    let preferred_label_match = placement_hint.preferred_worker_labels.iter().all(|(key, value)| {
        match key.as_str() {
            "backend_kind" => value == "any",
            "supports_container" => {
                value == if capabilities.supports_container { "true" } else { "false" }
            }
            "supports_gpu" => value == if capabilities.supports_gpu { "true" } else { "false" },
            "sandbox_profile" => {
                capabilities.supports_sandbox_profiles.iter().any(|profile| profile == value)
            }
            _ => false,
        }
    });
    let mut gaps = Vec::new();
    if worker_id.trim().is_empty() {
        gaps.push("capability advertisement requires a worker identifier".to_string());
    }
    if pool.pool_id.trim().is_empty() || pool.class.trim().is_empty() {
        gaps.push("worker pool advertisement must declare a stable pool id and class".to_string());
    }
    if pool.workers.is_empty() || !pool.workers.iter().any(|id| id == &worker_id) {
        gaps.push("worker must appear in the declared worker pool membership".to_string());
    }
    if !capability_match {
        gaps.push("declared worker capabilities do not satisfy the requested placement contract".to_string());
    }
    if !preferred_pool_match {
        gaps.push("placement hint points to a different worker pool".to_string());
    }
    if !preferred_label_match {
        gaps.push("worker capabilities do not satisfy the preferred placement labels".to_string());
    }
    let report = CapabilityReport {
        worker_id,
        pool_id: pool.pool_id,
        pool_class: pool.class,
        worker_count: pool.workers.len(),
        capability_match,
        preferred_pool_match,
        preferred_label_match,
        capability_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.capability_ready;
    (serde_json::to_value(report).expect("capability report"), ok)
}

fn drain_payload(simulation: DrainSimulation) -> (serde_json::Value, bool) {
    let DrainSimulation {
        worker_id,
        heartbeat,
        lease_semantics,
        leases,
        draining_started_unix_ms,
        now_unix_ms,
        new_dispatch_blocked,
        replacement_pool_ready,
    } = simulation;
    let semantics_valid = validate_task_lease_semantics(&lease_semantics).is_ok();
    let worker_leases: Vec<_> = leases.into_iter().filter(|lease| lease.worker_id == worker_id).collect();
    let active_leases = worker_leases
        .iter()
        .filter(|lease| lease.expires_unix_ms >= now_unix_ms)
        .count();
    let expired_leases = worker_leases
        .iter()
        .filter(|lease| lease.expires_unix_ms < now_unix_ms)
        .count();
    let recoverable_leases = worker_leases
        .iter()
        .filter(|lease| {
            now_unix_ms.saturating_sub(lease.expires_unix_ms) <= lease_semantics.recovery_grace_ms as u128
        })
        .count();
    let mut gaps = Vec::new();
    if worker_id.trim().is_empty() {
        gaps.push("drain policy requires a worker identifier".to_string());
    }
    if heartbeat.worker_id != worker_id {
        gaps.push("drain heartbeat does not match the targeted worker".to_string());
    }
    if draining_started_unix_ms == 0 {
        gaps.push("drain flow must record when the worker entered drain mode".to_string());
    }
    if !semantics_valid {
        gaps.push("drain flow requires valid task lease semantics".to_string());
    }
    if !new_dispatch_blocked {
        gaps.push("new dispatch must be blocked before a worker is drained".to_string());
    }
    if active_leases > 0 && !replacement_pool_ready {
        gaps.push("replacement capacity must be declared before draining active work".to_string());
    }
    if expired_leases > recoverable_leases {
        gaps.push("some expired worker leases are outside the recovery grace window".to_string());
    }
    let report = DrainReport {
        worker_id,
        inflight_nodes: heartbeat.inflight_nodes.len(),
        active_leases,
        expired_leases,
        recoverable_leases,
        new_dispatch_blocked,
        replacement_pool_ready,
        drain_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.drain_ready;
    (serde_json::to_value(report).expect("drain report"), ok)
}

fn autoscale_payload(simulation: AutoscaleSimulation) -> (serde_json::Value, bool) {
    let AutoscaleSimulation {
        queue_partition,
        scaling_plan,
        queue_depth,
        dispatch_lag_seconds,
        saturation_pct,
        current_replicas,
    } = simulation;
    let hint =
        derive_autoscaling_hint(queue_depth, dispatch_lag_seconds, saturation_pct, current_replicas);
    let mut gaps = Vec::new();
    if queue_partition.queue_name.trim().is_empty() {
        gaps.push("autoscaling hook requires a named queue partition".to_string());
    }
    if queue_partition.max_concurrency == 0 {
        gaps.push("queue partition must declare a non-zero concurrency ceiling".to_string());
    }
    if scaling_plan.worker_count == 0 {
        gaps.push("scheduler scaling plan must declare worker capacity".to_string());
    }
    if scaling_plan.sharding_key.trim().is_empty() {
        gaps.push("scheduler scaling plan must declare a sharding key".to_string());
    }
    if hint.recommended_replicas < current_replicas {
        gaps.push("autoscaling hint should not undercut the currently declared replicas".to_string());
    }
    if saturation_pct > 80 && hint.recommended_replicas == current_replicas {
        gaps.push("high saturation should drive an increased replica recommendation".to_string());
    }
    let report = AutoscaleReport {
        queue_name: queue_partition.queue_name,
        target_component: hint.target_component,
        current_replicas,
        recommended_replicas: hint.recommended_replicas,
        worker_count_declared: scaling_plan.worker_count,
        sharding_key: scaling_plan.sharding_key,
        autoscale_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.autoscale_ready;
    (serde_json::to_value(report).expect("autoscale report"), ok)
}

fn warm_pool_payload(simulation: WarmPoolSimulation) -> (serde_json::Value, bool) {
    let WarmPoolSimulation {
        pool,
        target_runtime_class,
        warm_worker_ids,
        preloaded_profiles,
        cold_start_ms,
        warm_start_ms,
        monthly_cost_estimate,
        policy_id,
    } = simulation;
    let startup_improvement_ms = cold_start_ms.saturating_sub(warm_start_ms);
    let mut gaps = Vec::new();
    if pool.pool_id.trim().is_empty() || pool.class.trim().is_empty() {
        gaps.push("warm pool requires a named worker pool and class".to_string());
    }
    if target_runtime_class.trim().is_empty() {
        gaps.push("warm pool must declare its target runtime class".to_string());
    }
    if pool.class != target_runtime_class {
        gaps.push("warm pool class must match the targeted runtime class".to_string());
    }
    if warm_worker_ids.is_empty() {
        gaps.push("warm pool should keep at least one worker prewarmed".to_string());
    }
    if warm_worker_ids.iter().any(|worker| !pool.workers.iter().any(|id| id == worker)) {
        gaps.push("all warm workers must belong to the declared worker pool".to_string());
    }
    if preloaded_profiles.is_empty() {
        gaps.push("warm pool should declare at least one preloaded profile".to_string());
    }
    if startup_improvement_ms == 0 {
        gaps.push("warm pool must demonstrate a startup improvement over cold capacity".to_string());
    }
    if monthly_cost_estimate <= 0.0 {
        gaps.push("warm pool cost must be visible to operators".to_string());
    }
    if policy_id.trim().is_empty() {
        gaps.push("warm pool requires a governing policy identifier".to_string());
    }
    let report = WarmPoolReport {
        pool_id: pool.pool_id,
        target_runtime_class,
        warm_worker_count: warm_worker_ids.len(),
        preloaded_profiles,
        startup_improvement_ms,
        monthly_cost_estimate,
        policy_id,
        warm_pool_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.warm_pool_ready;
    (serde_json::to_value(report).expect("warm pool report"), ok)
}

fn isolation_payload(simulation: IsolationSimulation) -> (serde_json::Value, bool) {
    let IsolationSimulation {
        tenant_id,
        queue_policy,
        quota,
        scheduler_admission,
        queue_partitions,
        queued_runs,
        pending_dispatches,
        observed_foreign_tenants,
    } = simulation;
    let scheduler_admitted =
        check_scheduler_admission(queued_runs, pending_dispatches, &scheduler_admission);
    let isolated_queues = queue_partitions
        .iter()
        .filter(|partition| partition.tenant_id.as_deref() == Some(queue_policy.tenant_id.0.as_str()))
        .map(|partition| partition.queue_name.clone())
        .collect::<Vec<_>>();
    let mut gaps = Vec::new();
    if tenant_id != queue_policy.tenant_id.0 || tenant_id != quota.tenant_id.0 || tenant_id != scheduler_admission.tenant_id.0 {
        gaps.push("tenant isolation inputs must all target the same tenant".to_string());
    }
    if !queue_policy.hard_isolation {
        gaps.push("noisy-neighbor protection requires hard queue isolation".to_string());
    }
    if isolated_queues.is_empty() {
        gaps.push("tenant must own at least one dedicated queue partition".to_string());
    }
    if quota.max_runs == 0 || quota.max_nodes == 0 {
        gaps.push("tenant quota must declare non-zero run and node ceilings".to_string());
    }
    if !scheduler_admitted {
        gaps.push("scheduler admission limits are already saturated for this tenant".to_string());
    }
    if !observed_foreign_tenants.is_empty() {
        gaps.push("foreign tenants were observed on a supposedly isolated fleet path".to_string());
    }
    let report = IsolationReport {
        tenant_id,
        isolated_queues,
        scheduler_admitted,
        hard_isolation: queue_policy.hard_isolation,
        foreign_tenants_observed: observed_foreign_tenants,
        isolation_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.isolation_ready;
    (serde_json::to_value(report).expect("isolation report"), ok)
}

fn preemption_payload(simulation: PreemptionSimulation) -> (serde_json::Value, bool) {
    let PreemptionSimulation {
        node_id,
        side_effect_class,
        checkpointing_supported,
        preemptible_backend,
        cancellation_issued_unix_ms,
        cancellation_delivered_unix_ms,
        cancellation_deadline_ms,
        lease_semantics,
        reassignment_rule,
    } = simulation;
    let semantics_valid = validate_task_lease_semantics(&lease_semantics).is_ok();
    let delivered_in_time = cancellation_delivered_in_time(
        cancellation_issued_unix_ms,
        cancellation_delivered_unix_ms,
        cancellation_deadline_ms,
    );
    let mut gaps = Vec::new();
    if node_id.trim().is_empty() {
        gaps.push("preemption policy requires a node identifier".to_string());
    }
    if side_effect_class.trim().is_empty() {
        gaps.push("preemption policy requires a declared side-effect class".to_string());
    }
    if !semantics_valid {
        gaps.push("preemption policy requires valid lease semantics".to_string());
    }
    if !preemptible_backend {
        gaps.push("selected backend does not support safe task preemption".to_string());
    }
    if side_effect_class != "read-only" && !checkpointing_supported {
        gaps.push("non-read-only work must be checkpointable before preemption".to_string());
    }
    if !reassignment_rule.preserve_attempt_lineage {
        gaps.push("preemption must preserve attempt lineage during reassignment".to_string());
    }
    if reassignment_rule.max_reassignments == 0 {
        gaps.push("preemption policy must declare a bounded reassignment budget".to_string());
    }
    if !delivered_in_time {
        gaps.push("cancellation did not reach the worker before the deadline".to_string());
    }
    let report = PreemptionReport {
        node_id,
        side_effect_class,
        checkpointing_supported,
        preemptible_backend,
        cancellation_delivered_in_time: delivered_in_time,
        preserve_attempt_lineage: reassignment_rule.preserve_attempt_lineage,
        preemption_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.preemption_ready;
    (serde_json::to_value(report).expect("preemption report"), ok)
}

fn trust_payload(simulation: TrustSimulation) -> (serde_json::Value, bool) {
    let TrustSimulation { identity, bootstrap, trust_domain, mutual_auth, enrollment_approved, attested_image } =
        simulation;
    let identity_valid = validate_worker_identity(&identity).is_ok();
    let trust_domain_name = format!(
        "{}/{}/{}",
        trust_domain.tenant, trust_domain.environment, trust_domain.execution_backend
    );
    let mut gaps = Vec::new();
    if !identity_valid {
        gaps.push("worker trust flow requires a valid stable worker identity".to_string());
    }
    if bootstrap.worker_id != identity.worker_id {
        gaps.push("worker bootstrap flow must bind to the same worker identity".to_string());
    }
    if bootstrap.trust_domain != trust_domain_name {
        gaps.push("worker bootstrap flow is bound to a different trust domain".to_string());
    }
    if identity.backend_kind != trust_domain.execution_backend {
        gaps.push("worker backend kind does not match the declared trust domain backend".to_string());
    }
    if !enrollment_approved {
        gaps.push("ephemeral worker enrollment is not approved".to_string());
    }
    if !attested_image {
        gaps.push("worker image provenance is not attested".to_string());
    }
    let mutual_auth_required =
        mutual_auth.requirement.contains("mutual") || mutual_auth.requirement.contains("mTLS");
    if mutual_auth.worker_identity != identity.worker_id {
        gaps.push("mutual-auth contract references a different worker identity".to_string());
    }
    if !mutual_auth_required {
        gaps.push("worker bootstrap must require mutual authentication".to_string());
    }
    let report = TrustReport {
        worker_id: identity.worker_id,
        trust_domain: trust_domain_name,
        transport: mutual_auth.transport,
        enrollment_approved,
        attested_image,
        mutual_auth_required,
        trust_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.trust_ready;
    (serde_json::to_value(report).expect("trust report"), ok)
}

fn gossip_payload(simulation: GossipSimulation) -> (serde_json::Value, bool) {
    let GossipSimulation {
        heartbeats,
        heartbeat_semantics,
        now_unix_ms,
        max_peer_fanout,
        observed_peer_fanout,
        authoritative_worker_ids,
        gossip_worker_ids,
    } = simulation;
    let mut healthy_workers = 0usize;
    let mut delayed_workers = 0usize;
    let mut lost_workers = 0usize;
    for heartbeat in &heartbeats {
        match classify_heartbeat(heartbeat, now_unix_ms, &heartbeat_semantics) {
            HeartbeatClass::Healthy => healthy_workers += 1,
            HeartbeatClass::Delayed => delayed_workers += 1,
            HeartbeatClass::Lost => lost_workers += 1,
        }
    }
    let authoritative = authoritative_worker_ids.into_iter().collect::<BTreeSet<_>>();
    let gossip = gossip_worker_ids.into_iter().collect::<BTreeSet<_>>();
    let authoritative_converged = authoritative == gossip;
    let fanout_bounded = observed_peer_fanout <= max_peer_fanout;
    let mut gaps = Vec::new();
    if heartbeats.is_empty() {
        gaps.push("gossip audit requires at least one worker heartbeat".to_string());
    }
    if heartbeat_semantics.interval_ms == 0
        || heartbeat_semantics.timeout_ms == 0
        || heartbeat_semantics.delayed_threshold_ms == 0
    {
        gaps.push("gossip audit requires explicit heartbeat timing semantics".to_string());
    }
    if !fanout_bounded {
        gaps.push("worker gossip fan-out exceeds the declared bound".to_string());
    }
    if !authoritative_converged {
        gaps.push("gossip view does not converge to the authoritative worker set".to_string());
    }
    if lost_workers > 0 && delayed_workers == 0 {
        gaps.push("lost workers must surface a delayed state before they become authoritative loss".to_string());
    }
    let report = GossipReport {
        healthy_workers,
        delayed_workers,
        lost_workers,
        authoritative_converged,
        fanout_bounded,
        gossip_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.gossip_ready;
    (serde_json::to_value(report).expect("gossip report"), ok)
}

pub(crate) fn handle_fleet_command(
    cli: &DagCli,
    command: &FleetCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        FleetCommands::Register { simulation } => {
            let simulation: RegistrationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = registration_payload(simulation);
            ("dag.fleet.register", payload, ok)
        }
        FleetCommands::Capabilities { simulation } => {
            let simulation: CapabilitySimulation = parse_json_file(simulation)?;
            let (payload, ok) = capability_payload(simulation);
            ("dag.fleet.capabilities", payload, ok)
        }
        FleetCommands::Drain { simulation } => {
            let simulation: DrainSimulation = parse_json_file(simulation)?;
            let (payload, ok) = drain_payload(simulation);
            ("dag.fleet.drain", payload, ok)
        }
        FleetCommands::Autoscale { simulation } => {
            let simulation: AutoscaleSimulation = parse_json_file(simulation)?;
            let (payload, ok) = autoscale_payload(simulation);
            ("dag.fleet.autoscale", payload, ok)
        }
        FleetCommands::WarmPool { simulation } => {
            let simulation: WarmPoolSimulation = parse_json_file(simulation)?;
            let (payload, ok) = warm_pool_payload(simulation);
            ("dag.fleet.warm-pool", payload, ok)
        }
        FleetCommands::Isolation { simulation } => {
            let simulation: IsolationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = isolation_payload(simulation);
            ("dag.fleet.isolation", payload, ok)
        }
        FleetCommands::Preemption { simulation } => {
            let simulation: PreemptionSimulation = parse_json_file(simulation)?;
            let (payload, ok) = preemption_payload(simulation);
            ("dag.fleet.preemption", payload, ok)
        }
        FleetCommands::Trust { simulation } => {
            let simulation: TrustSimulation = parse_json_file(simulation)?;
            let (payload, ok) = trust_payload(simulation);
            ("dag.fleet.trust", payload, ok)
        }
        FleetCommands::Gossip { simulation } => {
            let simulation: GossipSimulation = parse_json_file(simulation)?;
            let (payload, ok) = gossip_payload(simulation);
            ("dag.fleet.gossip", payload, ok)
        }
    };
    emit_json(
        cli,
        surface,
        ok,
        payload,
        if ok {
            Vec::new()
        } else {
            vec![json!({
                "message":"worker fleet posture is incomplete",
                "remediation":"fix the reported fleet gaps before treating this worker path as production-ready"
            })]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        autoscale_payload, capability_payload, drain_payload, registration_payload,
        warm_pool_payload, isolation_payload, AutoscaleSimulation, CapabilitySimulation,
        DrainSimulation, IsolationSimulation, PreemptionSimulation, RegistrationSimulation,
        TrustSimulation, WarmPoolSimulation, GossipSimulation, gossip_payload,
        preemption_payload, trust_payload,
    };
    use bijux_dag_runtime::simulated_platform::{
        HeartbeatSemantics, LivenessPolicy, MutualAuthDesignNote, PlacementHint, QueuePartition,
        ReassignmentRule, SchedulerScalingPlan, TaskLeaseSemantics, TenantConcurrencyQuota,
        TenantId, TenantQueueIsolationPolicy, TenantSchedulerAdmission, TrustDomain, WorkLease,
        WorkerBootstrapTrustFlow, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity, WorkerPool,
        WorkerPoolCapabilityRequest, WorkerRegistration, WorkerVersionCompatibilityRule,
    };
    use std::collections::BTreeMap;

    #[test]
    fn registration_accepts_live_compatible_worker() {
        let simulation = RegistrationSimulation {
            registration: WorkerRegistration {
                identity: WorkerIdentity {
                    worker_id: "worker-a".to_string(),
                    worker_version: "1.4.0".to_string(),
                    backend_kind: "kubernetes".to_string(),
                    labels: BTreeMap::from([
                        ("region".to_string(), "eu-north".to_string()),
                        ("pool".to_string(), "general".to_string()),
                    ]),
                },
                capabilities: WorkerCapabilities {
                    cpu_capacity: 16,
                    memory_mb: 65_536,
                    supports_gpu: false,
                    supports_container: true,
                    supports_sandbox_profiles: vec!["strict".to_string()],
                },
                registered_unix_ms: 1_700_000_000_000,
            },
            heartbeat: WorkerHeartbeat {
                worker_id: "worker-a".to_string(),
                unix_ms: 1_700_000_000_100,
                inflight_nodes: vec!["load".to_string()],
            },
            liveness_policy: LivenessPolicy { heartbeat_timeout_ms: 5_000, grace_retries: 2 },
            version_rule: WorkerVersionCompatibilityRule {
                planner_version: "1.4.0".to_string(),
                minimum_worker_version: "1.3.0".to_string(),
            },
            now_unix_ms: 1_700_000_001_000,
        };
        let (payload, ok) = registration_payload(simulation);
        assert!(ok);
        assert_eq!(payload["registration_ready"], true);
    }

    #[test]
    fn registration_flags_stale_or_anonymous_worker() {
        let simulation = RegistrationSimulation {
            registration: WorkerRegistration {
                identity: WorkerIdentity {
                    worker_id: String::new(),
                    worker_version: "0.8.0".to_string(),
                    backend_kind: String::new(),
                    labels: BTreeMap::new(),
                },
                capabilities: WorkerCapabilities {
                    cpu_capacity: 0,
                    memory_mb: 0,
                    supports_gpu: false,
                    supports_container: false,
                    supports_sandbox_profiles: Vec::new(),
                },
                registered_unix_ms: 0,
            },
            heartbeat: WorkerHeartbeat {
                worker_id: "other-worker".to_string(),
                unix_ms: 10,
                inflight_nodes: Vec::new(),
            },
            liveness_policy: LivenessPolicy { heartbeat_timeout_ms: 100, grace_retries: 1 },
            version_rule: WorkerVersionCompatibilityRule {
                planner_version: "1.4.0".to_string(),
                minimum_worker_version: "1.3.0".to_string(),
            },
            now_unix_ms: 1_000,
        };
        let (payload, ok) = registration_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn capability_accepts_worker_that_satisfies_requested_pool_contract() {
        let simulation = CapabilitySimulation {
            worker_id: "worker-a".to_string(),
            capabilities: WorkerCapabilities {
                cpu_capacity: 32,
                memory_mb: 131_072,
                supports_gpu: true,
                supports_container: true,
                supports_sandbox_profiles: vec!["strict".to_string(), "gpu".to_string()],
            },
            request: WorkerPoolCapabilityRequest {
                required_min_cpu_capacity: 16,
                required_min_memory_mb: 65_536,
                require_gpu: true,
                require_container_support: true,
                required_sandbox_profile: Some("strict".to_string()),
            },
            pool: WorkerPool {
                pool_id: "gpu-prod".to_string(),
                class: "gpu".to_string(),
                workers: vec!["worker-a".to_string(), "worker-b".to_string()],
            },
            placement_hint: PlacementHint {
                node_id: "train".to_string(),
                preferred_pool: "gpu-prod".to_string(),
                preferred_worker_labels: BTreeMap::from([
                    ("supports_gpu".to_string(), "true".to_string()),
                    ("sandbox_profile".to_string(), "strict".to_string()),
                ]),
            },
        };
        let (payload, ok) = capability_payload(simulation);
        assert!(ok);
        assert_eq!(payload["capability_ready"], true);
    }

    #[test]
    fn capability_flags_pool_or_feature_mismatch() {
        let simulation = CapabilitySimulation {
            worker_id: "worker-a".to_string(),
            capabilities: WorkerCapabilities {
                cpu_capacity: 4,
                memory_mb: 8_192,
                supports_gpu: false,
                supports_container: false,
                supports_sandbox_profiles: Vec::new(),
            },
            request: WorkerPoolCapabilityRequest {
                required_min_cpu_capacity: 16,
                required_min_memory_mb: 65_536,
                require_gpu: true,
                require_container_support: true,
                required_sandbox_profile: Some("strict".to_string()),
            },
            pool: WorkerPool {
                pool_id: "general".to_string(),
                class: String::new(),
                workers: vec!["worker-b".to_string()],
            },
            placement_hint: PlacementHint {
                node_id: "train".to_string(),
                preferred_pool: "gpu-prod".to_string(),
                preferred_worker_labels: BTreeMap::from([
                    ("supports_gpu".to_string(), "true".to_string()),
                    ("sandbox_profile".to_string(), "strict".to_string()),
                ]),
            },
        };
        let (payload, ok) = capability_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn drain_accepts_blocked_dispatch_with_recoverable_inflight_work() {
        let simulation = DrainSimulation {
            worker_id: "worker-a".to_string(),
            heartbeat: WorkerHeartbeat {
                worker_id: "worker-a".to_string(),
                unix_ms: 1_700_000_000_100,
                inflight_nodes: vec!["node-a".to_string(), "node-b".to_string()],
            },
            lease_semantics: TaskLeaseSemantics {
                lease_duration_ms: 5_000,
                renew_before_expiry_ms: 1_000,
                max_renewals: 3,
                recovery_grace_ms: 2_000,
            },
            leases: vec![
                WorkLease {
                    lease_id: "lease-1".to_string(),
                    run_id: "run-1".to_string(),
                    node_id: "node-a".to_string(),
                    worker_id: "worker-a".to_string(),
                    expires_unix_ms: 1_700_000_001_500,
                },
                WorkLease {
                    lease_id: "lease-2".to_string(),
                    run_id: "run-1".to_string(),
                    node_id: "node-b".to_string(),
                    worker_id: "worker-a".to_string(),
                    expires_unix_ms: 1_700_000_000_500,
                },
            ],
            draining_started_unix_ms: 1_700_000_000_000,
            now_unix_ms: 1_700_000_001_000,
            new_dispatch_blocked: true,
            replacement_pool_ready: true,
        };
        let (payload, ok) = drain_payload(simulation);
        assert!(ok);
        assert_eq!(payload["drain_ready"], true);
    }

    #[test]
    fn drain_flags_unbounded_or_unprepared_maintenance() {
        let simulation = DrainSimulation {
            worker_id: "worker-a".to_string(),
            heartbeat: WorkerHeartbeat {
                worker_id: "other-worker".to_string(),
                unix_ms: 1_700_000_000_100,
                inflight_nodes: vec!["node-a".to_string()],
            },
            lease_semantics: TaskLeaseSemantics {
                lease_duration_ms: 0,
                renew_before_expiry_ms: 0,
                max_renewals: 0,
                recovery_grace_ms: 100,
            },
            leases: vec![WorkLease {
                lease_id: "lease-1".to_string(),
                run_id: "run-1".to_string(),
                node_id: "node-a".to_string(),
                worker_id: "worker-a".to_string(),
                expires_unix_ms: 10,
            }],
            draining_started_unix_ms: 0,
            now_unix_ms: 1_000,
            new_dispatch_blocked: false,
            replacement_pool_ready: false,
        };
        let (payload, ok) = drain_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn autoscale_accepts_pressure_backed_replica_growth() {
        let simulation = AutoscaleSimulation {
            queue_partition: QueuePartition {
                queue_name: "scheduler-workers".to_string(),
                tenant_id: Some("atlas".to_string()),
                max_concurrency: 64,
            },
            scaling_plan: SchedulerScalingPlan {
                worker_count: 12,
                sharding_key: "tenant".to_string(),
            },
            queue_depth: 2_000,
            dispatch_lag_seconds: 45,
            saturation_pct: 90,
            current_replicas: 4,
        };
        let (payload, ok) = autoscale_payload(simulation);
        assert!(ok);
        assert_eq!(payload["recommended_replicas"], 6);
    }

    #[test]
    fn autoscale_flags_missing_partition_or_nonresponsive_scaling_plan() {
        let simulation = AutoscaleSimulation {
            queue_partition: QueuePartition {
                queue_name: String::new(),
                tenant_id: None,
                max_concurrency: 0,
            },
            scaling_plan: SchedulerScalingPlan {
                worker_count: 0,
                sharding_key: String::new(),
            },
            queue_depth: 2_000,
            dispatch_lag_seconds: 45,
            saturation_pct: 90,
            current_replicas: 4,
        };
        let (payload, ok) = autoscale_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn warm_pool_accepts_cost_visible_prewarmed_capacity() {
        let simulation = WarmPoolSimulation {
            pool: WorkerPool {
                pool_id: "container-prod".to_string(),
                class: "container".to_string(),
                workers: vec!["worker-a".to_string(), "worker-b".to_string()],
            },
            target_runtime_class: "container".to_string(),
            warm_worker_ids: vec!["worker-a".to_string()],
            preloaded_profiles: vec!["python-biomed".to_string()],
            cold_start_ms: 12_000,
            warm_start_ms: 2_000,
            monthly_cost_estimate: 640.0,
            policy_id: "warm-pool-prod".to_string(),
        };
        let (payload, ok) = warm_pool_payload(simulation);
        assert!(ok);
        assert_eq!(payload["warm_pool_ready"], true);
    }

    #[test]
    fn warm_pool_flags_unscoped_or_cost_blind_capacity() {
        let simulation = WarmPoolSimulation {
            pool: WorkerPool {
                pool_id: String::new(),
                class: "general".to_string(),
                workers: vec!["worker-b".to_string()],
            },
            target_runtime_class: String::new(),
            warm_worker_ids: vec!["worker-a".to_string()],
            preloaded_profiles: Vec::new(),
            cold_start_ms: 2_000,
            warm_start_ms: 2_000,
            monthly_cost_estimate: 0.0,
            policy_id: String::new(),
        };
        let (payload, ok) = warm_pool_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 6);
    }

    #[test]
    fn isolation_accepts_hard_isolated_tenant_capacity() {
        let simulation = IsolationSimulation {
            tenant_id: "atlas".to_string(),
            queue_policy: TenantQueueIsolationPolicy {
                tenant_id: TenantId("atlas".to_string()),
                queue_names: vec!["atlas-high".to_string()],
                hard_isolation: true,
            },
            quota: TenantConcurrencyQuota {
                tenant_id: TenantId("atlas".to_string()),
                max_runs: 50,
                max_nodes: 500,
                max_backfills: 5,
            },
            scheduler_admission: TenantSchedulerAdmission {
                tenant_id: TenantId("atlas".to_string()),
                max_enqueued_runs: 100,
                max_dispatches_per_tick: 20,
            },
            queue_partitions: vec![QueuePartition {
                queue_name: "atlas-high".to_string(),
                tenant_id: Some("atlas".to_string()),
                max_concurrency: 32,
            }],
            queued_runs: 40,
            pending_dispatches: 10,
            observed_foreign_tenants: Vec::new(),
        };
        let (payload, ok) = isolation_payload(simulation);
        assert!(ok);
        assert_eq!(payload["isolation_ready"], true);
    }

    #[test]
    fn isolation_flags_shared_or_saturated_tenant_path() {
        let simulation = IsolationSimulation {
            tenant_id: "atlas".to_string(),
            queue_policy: TenantQueueIsolationPolicy {
                tenant_id: TenantId("other".to_string()),
                queue_names: vec!["shared".to_string()],
                hard_isolation: false,
            },
            quota: TenantConcurrencyQuota {
                tenant_id: TenantId("atlas".to_string()),
                max_runs: 0,
                max_nodes: 0,
                max_backfills: 0,
            },
            scheduler_admission: TenantSchedulerAdmission {
                tenant_id: TenantId("atlas".to_string()),
                max_enqueued_runs: 1,
                max_dispatches_per_tick: 1,
            },
            queue_partitions: vec![QueuePartition {
                queue_name: "shared".to_string(),
                tenant_id: Some("other".to_string()),
                max_concurrency: 8,
            }],
            queued_runs: 10,
            pending_dispatches: 5,
            observed_foreign_tenants: vec!["canon".to_string()],
        };
        let (payload, ok) = isolation_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn preemption_accepts_checkpointed_and_bounded_reassignment() {
        let simulation = PreemptionSimulation {
            node_id: "train".to_string(),
            side_effect_class: "checkpointed-write".to_string(),
            checkpointing_supported: true,
            preemptible_backend: true,
            cancellation_issued_unix_ms: 1_000,
            cancellation_delivered_unix_ms: 1_300,
            cancellation_deadline_ms: 1_000,
            lease_semantics: TaskLeaseSemantics {
                lease_duration_ms: 5_000,
                renew_before_expiry_ms: 1_000,
                max_renewals: 3,
                recovery_grace_ms: 2_000,
            },
            reassignment_rule: ReassignmentRule {
                trigger: "capacity-rebalance".to_string(),
                max_reassignments: 2,
                preserve_attempt_lineage: true,
            },
        };
        let (payload, ok) = preemption_payload(simulation);
        assert!(ok);
        assert_eq!(payload["preemption_ready"], true);
    }

    #[test]
    fn preemption_flags_uncheckpointed_or_late_cancellation() {
        let simulation = PreemptionSimulation {
            node_id: String::new(),
            side_effect_class: "mutating".to_string(),
            checkpointing_supported: false,
            preemptible_backend: false,
            cancellation_issued_unix_ms: 1_000,
            cancellation_delivered_unix_ms: 5_000,
            cancellation_deadline_ms: 500,
            lease_semantics: TaskLeaseSemantics {
                lease_duration_ms: 0,
                renew_before_expiry_ms: 0,
                max_renewals: 0,
                recovery_grace_ms: 0,
            },
            reassignment_rule: ReassignmentRule {
                trigger: "manual".to_string(),
                max_reassignments: 0,
                preserve_attempt_lineage: false,
            },
        };
        let (payload, ok) = preemption_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 6);
    }

    #[test]
    fn trust_accepts_enrolled_and_attested_worker_bootstrap() {
        let simulation = TrustSimulation {
            identity: WorkerIdentity {
                worker_id: "worker-a".to_string(),
                worker_version: "1.5.0".to_string(),
                backend_kind: "kubernetes".to_string(),
                labels: BTreeMap::new(),
            },
            bootstrap: WorkerBootstrapTrustFlow {
                worker_id: "worker-a".to_string(),
                enrollment_token_id: "token-1".to_string(),
                trust_domain: "atlas/prod/kubernetes".to_string(),
            },
            trust_domain: TrustDomain {
                tenant: "atlas".to_string(),
                environment: "prod".to_string(),
                execution_backend: "kubernetes".to_string(),
            },
            mutual_auth: MutualAuthDesignNote {
                control_plane_identity: "scheduler.prod".to_string(),
                worker_identity: "worker-a".to_string(),
                transport: "grpc".to_string(),
                requirement: "mutual tls required".to_string(),
            },
            enrollment_approved: true,
            attested_image: true,
        };
        let (payload, ok) = trust_payload(simulation);
        assert!(ok);
        assert_eq!(payload["trust_ready"], true);
    }

    #[test]
    fn trust_flags_unapproved_or_mismatched_worker_bootstrap() {
        let simulation = TrustSimulation {
            identity: WorkerIdentity {
                worker_id: String::new(),
                worker_version: "1.5.0".to_string(),
                backend_kind: "remote".to_string(),
                labels: BTreeMap::new(),
            },
            bootstrap: WorkerBootstrapTrustFlow {
                worker_id: "worker-b".to_string(),
                enrollment_token_id: "token-1".to_string(),
                trust_domain: "atlas/prod/kubernetes".to_string(),
            },
            trust_domain: TrustDomain {
                tenant: "atlas".to_string(),
                environment: "prod".to_string(),
                execution_backend: "kubernetes".to_string(),
            },
            mutual_auth: MutualAuthDesignNote {
                control_plane_identity: "scheduler.prod".to_string(),
                worker_identity: "worker-c".to_string(),
                transport: "http".to_string(),
                requirement: "optional".to_string(),
            },
            enrollment_approved: false,
            attested_image: false,
        };
        let (payload, ok) = trust_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 6);
    }

    #[test]
    fn gossip_accepts_bounded_converged_worker_view() {
        let simulation = GossipSimulation {
            heartbeats: vec![
                WorkerHeartbeat {
                    worker_id: "worker-a".to_string(),
                    unix_ms: 1_000,
                    inflight_nodes: Vec::new(),
                },
                WorkerHeartbeat {
                    worker_id: "worker-b".to_string(),
                    unix_ms: 950,
                    inflight_nodes: Vec::new(),
                },
            ],
            heartbeat_semantics: HeartbeatSemantics {
                interval_ms: 100,
                timeout_ms: 500,
                delayed_threshold_ms: 200,
            },
            now_unix_ms: 1_100,
            max_peer_fanout: 5,
            observed_peer_fanout: 2,
            authoritative_worker_ids: vec!["worker-a".to_string(), "worker-b".to_string()],
            gossip_worker_ids: vec!["worker-a".to_string(), "worker-b".to_string()],
        };
        let (payload, ok) = gossip_payload(simulation);
        assert!(ok);
        assert_eq!(payload["gossip_ready"], true);
    }

    #[test]
    fn gossip_flags_split_brain_or_unbounded_fanout() {
        let simulation = GossipSimulation {
            heartbeats: vec![WorkerHeartbeat {
                worker_id: "worker-a".to_string(),
                unix_ms: 0,
                inflight_nodes: Vec::new(),
            }],
            heartbeat_semantics: HeartbeatSemantics {
                interval_ms: 0,
                timeout_ms: 100,
                delayed_threshold_ms: 0,
            },
            now_unix_ms: 1_000,
            max_peer_fanout: 2,
            observed_peer_fanout: 5,
            authoritative_worker_ids: vec!["worker-a".to_string()],
            gossip_worker_ids: vec!["worker-a".to_string(), "worker-b".to_string()],
        };
        let (payload, ok) = gossip_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }
}
