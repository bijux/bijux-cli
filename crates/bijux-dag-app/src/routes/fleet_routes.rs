use crate::commands::{DagCli, FleetCommands};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::derive_autoscaling_hint;
use bijux_dag_runtime::simulated_platform::{
    check_worker_version_compatibility, validate_worker_identity, worker_alive,
    validate_task_lease_semantics, worker_pool_satisfies_capability_request, LivenessPolicy,
    PlacementHint, QueuePartition, SchedulerScalingPlan, TaskLeaseSemantics, WorkLease,
    WorkerCapabilities, WorkerHeartbeat, WorkerPool, WorkerPoolCapabilityRequest,
    WorkerRegistration, WorkerVersionCompatibilityRule,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
        warm_pool_payload, AutoscaleSimulation, CapabilitySimulation, DrainSimulation,
        RegistrationSimulation, WarmPoolSimulation,
    };
    use bijux_dag_runtime::simulated_platform::{
        LivenessPolicy, PlacementHint, QueuePartition, SchedulerScalingPlan, TaskLeaseSemantics,
        WorkLease, WorkerCapabilities, WorkerHeartbeat, WorkerIdentity, WorkerPool,
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
}
