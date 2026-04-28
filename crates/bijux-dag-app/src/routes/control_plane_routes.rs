use crate::commands::{ControlPlaneCommands, DagCli};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::{merge_timeout_and_exit_events, thread_safety_audit};
use bijux_dag_runtime::simulated_platform::{
    deduplicate_across_replicas, fence_allows_mutation, idempotent_run_creation,
    invalidate_decision_cache, is_stale_leader, next_epoch, ordering_during_failover,
    resolve_environment_values, select_dag_version, validate_task_lease_semantics,
    ApiCompatibilityRule, ApiVersion, CompatibilityDecision, DagRegistry,
    DagVersionSelectionPolicy, DurableRunQueueEntry, EnvironmentConfiguration,
    LeaderElectionState, PolicyDecisionCache, QueueOwnershipTransfer, QueuePartition,
    QueueShardLease, RegionId, RegionQueuePartition, ScheduleDedupRecord, SchedulerEpoch,
    SchedulerFenceToken, TaskLeaseSemantics, TypedControlPlaneRequest,
    TypedControlPlaneResponse, WorkLease, check_api_compatibility,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

#[derive(Debug, Deserialize)]
struct ApiSimulation {
    request: TypedControlPlaneRequest,
    #[serde(default)]
    replica_ids: Vec<String>,
    #[serde(default)]
    responses: Vec<TypedControlPlaneResponse>,
    load_balanced: bool,
    hidden_in_memory_authority: bool,
}

#[derive(Debug, Serialize)]
struct ApiReport {
    operation: String,
    replica_count: usize,
    load_balanced: bool,
    hidden_in_memory_authority: bool,
    response_consistent: bool,
    gaps: Vec<String>,
    api_ready: bool,
}

#[derive(Debug, Deserialize)]
struct LeadershipSimulation {
    leader: LeaderElectionState,
    shard_lease: QueueShardLease,
    current_epoch: SchedulerEpoch,
    fence_token: SchedulerFenceToken,
    now_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct LeadershipReport {
    leader_replica_id: String,
    shard_id: String,
    leader_stale: bool,
    fence_allows_mutation: bool,
    shard_owner_consistent: bool,
    next_epoch: u64,
    gaps: Vec<String>,
    leadership_ready: bool,
}

#[derive(Debug, Deserialize)]
struct PlanningSimulation {
    dag_name: String,
    plan_digest: String,
    plan_persisted_unix_ms: u128,
    dispatch_started_unix_ms: u128,
    queue_entry: DurableRunQueueEntry,
    review_recorded: bool,
}

#[derive(Debug, Serialize)]
struct PlanningReport {
    dag_name: String,
    schedule_id: String,
    run_key: String,
    plan_persisted_before_dispatch: bool,
    review_recorded: bool,
    queue_key: String,
    gaps: Vec<String>,
    planning_ready: bool,
}

#[derive(Debug, Deserialize)]
struct ShardingSimulation {
    #[serde(default)]
    queue_partitions: Vec<QueuePartition>,
    #[serde(default)]
    region_partitions: Vec<RegionQueuePartition>,
    #[serde(default)]
    active_tenants: Vec<String>,
    #[serde(default)]
    active_regions: Vec<RegionId>,
}

#[derive(Debug, Serialize)]
struct ShardingReport {
    tenant_partition_count: usize,
    region_partition_count: usize,
    all_tenants_assigned: bool,
    all_regions_assigned: bool,
    shared_region_edges: usize,
    gaps: Vec<String>,
    sharding_ready: bool,
}

#[derive(Debug, Deserialize)]
struct LeasesSimulation {
    shard_lease: QueueShardLease,
    ownership_transfer: QueueOwnershipTransfer,
    work_lease: WorkLease,
    semantics: TaskLeaseSemantics,
    now_unix_ms: u128,
}

#[derive(Debug, Serialize)]
struct LeasesReport {
    shard_id: String,
    worker_id: String,
    shard_lease_live: bool,
    work_lease_live: bool,
    transfer_recorded: bool,
    semantics_valid: bool,
    gaps: Vec<String>,
    leases_ready: bool,
}

#[derive(Debug, Deserialize)]
struct IdempotencySimulation {
    #[serde(default)]
    existing_dedup: BTreeMap<String, String>,
    repeated_dedup_key: String,
    first_run_key: String,
    replayed_run_key: String,
    #[serde(default)]
    replica_records: Vec<ScheduleDedupRecord>,
    proposed_record: ScheduleDedupRecord,
    #[serde(default)]
    queue_entries: Vec<DurableRunQueueEntry>,
}

#[derive(Debug, Serialize)]
struct IdempotencyReport {
    dedup_key: String,
    canonical_run_key: String,
    stable_run_key: bool,
    replica_unique: bool,
    queue_order_stable: bool,
    gaps: Vec<String>,
    idempotency_ready: bool,
}

#[derive(Debug, Deserialize)]
struct BackpressureSimulation {
    #[serde(default)]
    queue_entries: Vec<DurableRunQueueEntry>,
    #[serde(default)]
    queue_partitions: Vec<QueuePartition>,
    dispatch_lag_ms: u64,
    ready_worker_slots: usize,
    high_watermark: usize,
    max_dispatch_lag_ms: u64,
    #[serde(default)]
    hot_tenants: Vec<String>,
}

#[derive(Debug, Serialize)]
struct BackpressureReport {
    queue_depth: usize,
    partitioned_capacity: usize,
    backlog_within_limit: bool,
    lag_within_limit: bool,
    enough_worker_slots: bool,
    hot_tenants_isolated: bool,
    gaps: Vec<String>,
    backpressure_ready: bool,
}

#[derive(Debug, Deserialize)]
struct CacheSimulation {
    registry: DagRegistry,
    dag_name: String,
    selection_policy: DagVersionSelectionPolicy,
    cache: PolicyDecisionCache,
    current_policy_bundle_version: String,
    current_env: EnvironmentConfiguration,
    parent_env: Option<EnvironmentConfiguration>,
}

#[derive(Debug, Serialize)]
struct CacheReport {
    selected_version: Option<String>,
    cache_entries_before: usize,
    cache_entries_after: usize,
    stale_entries_removed: usize,
    environment_keys: usize,
    selection_resolved: bool,
    cache_disciplined: bool,
    environment_resolved: bool,
    gaps: Vec<String>,
    cache_ready: bool,
}

#[derive(Debug, Deserialize)]
struct MigrationSimulation {
    api_version: ApiVersion,
    compatibility_rule: ApiCompatibilityRule,
    source_registry: DagRegistry,
    target_registry: DagRegistry,
    dry_run_completed: bool,
    rollback_plan_recorded: bool,
    mixed_writer_blocked: bool,
}

#[derive(Debug, Serialize)]
struct MigrationReport {
    compatibility_ok: bool,
    source_dag_count: usize,
    target_covers_all_dags: bool,
    dry_run_completed: bool,
    rollback_plan_recorded: bool,
    mixed_writer_blocked: bool,
    gaps: Vec<String>,
    migration_ready: bool,
}

#[derive(Debug, Deserialize)]
struct FanInSimulation {
    #[serde(default)]
    timed_out_nodes: Vec<String>,
    #[serde(default)]
    exited_nodes: Vec<String>,
    fan_in_limit: usize,
    aggregator_threads: usize,
    #[serde(default)]
    required_audit_surfaces: Vec<String>,
}

#[derive(Debug, Serialize)]
struct FanInReport {
    merged_event_count: usize,
    fan_in_within_limit: bool,
    aggregator_threads_bounded: bool,
    audit_surfaces_present: bool,
    gaps: Vec<String>,
    fan_in_ready: bool,
}

fn parse_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T, ExitCode> {
    let raw = read_file(path)?;
    serde_json::from_str(&raw).map_err(|_| ExitCode::from(3))
}

fn api_payload(simulation: ApiSimulation) -> (serde_json::Value, bool) {
    let ApiSimulation { request, replica_ids, responses, load_balanced, hidden_in_memory_authority } =
        simulation;
    let first = responses.first();
    let response_consistent = first.is_some()
        && responses.iter().all(|response| {
            response.accepted == first.expect("first response").accepted
                && response.message == first.expect("first response").message
                && response.details == first.expect("first response").details
        });
    let mut gaps = Vec::new();
    if replica_ids.len() < 2 {
        gaps.push("stateless api evaluation should compare at least two replicas".to_string());
    }
    if responses.len() != replica_ids.len() {
        gaps.push("api simulation must provide one response per replica".to_string());
    }
    if !load_balanced {
        gaps.push("api tier should be exercised behind a load-balanced path".to_string());
    }
    if hidden_in_memory_authority {
        gaps.push("api replica depends on hidden in-memory authority".to_string());
    }
    if !response_consistent {
        gaps.push("replicas returned inconsistent control-plane responses".to_string());
    }
    let report = ApiReport {
        operation: format!("{:?}", request.operation).to_lowercase(),
        replica_count: replica_ids.len(),
        load_balanced,
        hidden_in_memory_authority,
        response_consistent,
        api_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.api_ready;
    (serde_json::to_value(report).expect("api report"), ok)
}

fn leadership_payload(simulation: LeadershipSimulation) -> (serde_json::Value, bool) {
    let LeadershipSimulation { leader, shard_lease, current_epoch, fence_token, now_unix_ms } =
        simulation;
    let leader_stale = is_stale_leader(&leader, now_unix_ms);
    let next_epoch_value = next_epoch(&current_epoch).epoch;
    let fence_allows = fence_allows_mutation(&fence_token, &current_epoch);
    let shard_owner_consistent = shard_lease.owner_replica_id == leader.leader_replica_id;
    let mut gaps = Vec::new();
    if leader.leader_replica_id.trim().is_empty() {
        gaps.push("leadership protocol requires a stable replica identity".to_string());
    }
    if shard_lease.shard_id.trim().is_empty() {
        gaps.push("leadership protocol requires a shard identifier".to_string());
    }
    if leader_stale {
        gaps.push("current leader lease is already stale".to_string());
    }
    if !fence_allows {
        gaps.push("fence token does not authorize the current epoch to mutate".to_string());
    }
    if !shard_owner_consistent {
        gaps.push("shard lease owner does not match the elected leader".to_string());
    }
    if current_epoch.replica_id != leader.leader_replica_id {
        gaps.push("scheduler epoch replica id does not match the elected leader".to_string());
    }
    let report = LeadershipReport {
        leader_replica_id: leader.leader_replica_id,
        shard_id: shard_lease.shard_id,
        leader_stale,
        fence_allows_mutation: fence_allows,
        shard_owner_consistent,
        next_epoch: next_epoch_value,
        leadership_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.leadership_ready;
    (serde_json::to_value(report).expect("leadership report"), ok)
}

fn planning_payload(simulation: PlanningSimulation) -> (serde_json::Value, bool) {
    let PlanningSimulation {
        dag_name,
        plan_digest,
        plan_persisted_unix_ms,
        dispatch_started_unix_ms,
        queue_entry,
        review_recorded,
    } = simulation;
    let plan_persisted_before_dispatch = plan_persisted_unix_ms > 0
        && plan_persisted_unix_ms <= dispatch_started_unix_ms
        && queue_entry.created_unix_ms >= plan_persisted_unix_ms;
    let mut gaps = Vec::new();
    if dag_name.trim().is_empty() {
        gaps.push("planning audit requires a dag name".to_string());
    }
    if plan_digest.trim().is_empty() {
        gaps.push("planning audit requires a durable plan digest".to_string());
    }
    if !plan_persisted_before_dispatch {
        gaps.push("plan artifact was not durably persisted before dispatch began".to_string());
    }
    if !review_recorded {
        gaps.push("plan review was not recorded before scheduling".to_string());
    }
    if queue_entry.schedule_id.trim().is_empty() || queue_entry.run_key.trim().is_empty() {
        gaps.push("queued dispatch must reference schedule and run identity".to_string());
    }
    let report = PlanningReport {
        dag_name,
        schedule_id: queue_entry.schedule_id,
        run_key: queue_entry.run_key,
        plan_persisted_before_dispatch,
        review_recorded,
        queue_key: queue_entry.queue_key,
        planning_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.planning_ready;
    (serde_json::to_value(report).expect("planning report"), ok)
}

fn sharding_payload(simulation: ShardingSimulation) -> (serde_json::Value, bool) {
    let ShardingSimulation { queue_partitions, region_partitions, active_tenants, active_regions } =
        simulation;
    let assigned_tenants = queue_partitions
        .iter()
        .filter_map(|partition| partition.tenant_id.clone())
        .collect::<BTreeSet<_>>();
    let assigned_regions = region_partitions
        .iter()
        .map(|partition| partition.region.clone())
        .collect::<BTreeSet<_>>();
    let all_tenants_assigned = active_tenants.iter().all(|tenant| assigned_tenants.contains(tenant));
    let all_regions_assigned = active_regions.iter().all(|region| assigned_regions.contains(region));
    let shared_region_edges =
        region_partitions.iter().map(|partition| partition.shared_with_regions.len()).sum::<usize>();
    let mut gaps = Vec::new();
    if queue_partitions.is_empty() {
        gaps.push("control-plane sharding requires at least one queue partition".to_string());
    }
    if region_partitions.is_empty() {
        gaps.push("control-plane sharding requires at least one region partition".to_string());
    }
    if !all_tenants_assigned {
        gaps.push("not every active tenant is mapped to an explicit queue partition".to_string());
    }
    if !all_regions_assigned {
        gaps.push("not every active region is mapped to an explicit region partition".to_string());
    }
    if shared_region_edges > region_partitions.len() {
        gaps.push("regional partitions are overly shared and weaken shard isolation".to_string());
    }
    let report = ShardingReport {
        tenant_partition_count: queue_partitions.len(),
        region_partition_count: region_partitions.len(),
        all_tenants_assigned,
        all_regions_assigned,
        shared_region_edges,
        sharding_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.sharding_ready;
    (serde_json::to_value(report).expect("sharding report"), ok)
}

fn leases_payload(simulation: LeasesSimulation) -> (serde_json::Value, bool) {
    let LeasesSimulation { shard_lease, ownership_transfer, work_lease, semantics, now_unix_ms } =
        simulation;
    let shard_lease_live = shard_lease.lease_expires_unix_ms >= now_unix_ms;
    let work_lease_live = work_lease.expires_unix_ms >= now_unix_ms;
    let transfer_recorded = ownership_transfer.shard_id == shard_lease.shard_id
        && ownership_transfer.to_replica_id == shard_lease.owner_replica_id;
    let semantics_valid = validate_task_lease_semantics(&semantics).is_ok();
    let mut gaps = Vec::new();
    if shard_lease.shard_id.trim().is_empty() {
        gaps.push("durable lease audit requires a shard identifier".to_string());
    }
    if !shard_lease_live {
        gaps.push("shard lease has already expired".to_string());
    }
    if !work_lease_live {
        gaps.push("work lease has already expired".to_string());
    }
    if !transfer_recorded {
        gaps.push("ownership transfer is not recorded against the active shard lease".to_string());
    }
    if !semantics_valid {
        gaps.push("task lease semantics are invalid".to_string());
    }
    if work_lease.worker_id.trim().is_empty() {
        gaps.push("work lease must bind to a worker".to_string());
    }
    let report = LeasesReport {
        shard_id: shard_lease.shard_id,
        worker_id: work_lease.worker_id,
        shard_lease_live,
        work_lease_live,
        transfer_recorded,
        semantics_valid,
        leases_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.leases_ready;
    (serde_json::to_value(report).expect("leases report"), ok)
}

fn idempotency_payload(simulation: IdempotencySimulation) -> (serde_json::Value, bool) {
    let IdempotencySimulation {
        mut existing_dedup,
        repeated_dedup_key,
        first_run_key,
        replayed_run_key,
        replica_records,
        proposed_record,
        queue_entries,
    } = simulation;
    let canonical_run_key = idempotent_run_creation(
        &mut existing_dedup,
        &repeated_dedup_key,
        &first_run_key,
    );
    let replay_run_key = idempotent_run_creation(
        &mut existing_dedup,
        &repeated_dedup_key,
        &replayed_run_key,
    );
    let stable_run_key = canonical_run_key == replay_run_key;
    let replica_unique = deduplicate_across_replicas(&replica_records, &proposed_record);
    let canonical_order = ordering_during_failover(queue_entries.clone());
    let queue_order_stable = !queue_entries.is_empty() && queue_entries == canonical_order;
    let mut gaps = Vec::new();
    if repeated_dedup_key.trim().is_empty() {
        gaps.push("idempotent command audit requires a non-empty dedup key".to_string());
    }
    if !stable_run_key {
        gaps.push("replayed command resolved to a different canonical run key".to_string());
    }
    if !replica_unique {
        gaps.push("replica dedup ledger already contains the proposed schedule key".to_string());
    }
    if queue_entries.is_empty() {
        gaps.push("idempotency audit requires durable queue entries".to_string());
    }
    if !queue_order_stable {
        gaps.push("failover queue ordering is not deterministic".to_string());
    }
    if proposed_record.run_key != canonical_run_key {
        gaps.push("proposed dedup record does not point at the canonical run key".to_string());
    }
    let report = IdempotencyReport {
        dedup_key: repeated_dedup_key,
        canonical_run_key,
        stable_run_key,
        replica_unique,
        queue_order_stable,
        idempotency_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.idempotency_ready;
    (serde_json::to_value(report).expect("idempotency report"), ok)
}

fn backpressure_payload(simulation: BackpressureSimulation) -> (serde_json::Value, bool) {
    let BackpressureSimulation {
        queue_entries,
        queue_partitions,
        dispatch_lag_ms,
        ready_worker_slots,
        high_watermark,
        max_dispatch_lag_ms,
        hot_tenants,
    } = simulation;
    let queue_depth = queue_entries.len();
    let partitioned_capacity =
        queue_partitions.iter().map(|partition| partition.max_concurrency as usize).sum::<usize>();
    let backlog_within_limit =
        queue_depth <= high_watermark && queue_depth <= partitioned_capacity.max(high_watermark);
    let lag_within_limit = dispatch_lag_ms <= max_dispatch_lag_ms;
    let enough_worker_slots = ready_worker_slots >= queue_depth.min(partitioned_capacity.max(1));
    let assigned_tenants = queue_partitions
        .iter()
        .filter_map(|partition| partition.tenant_id.clone())
        .collect::<BTreeSet<_>>();
    let hot_tenants_isolated = hot_tenants.iter().all(|tenant| assigned_tenants.contains(tenant));
    let mut gaps = Vec::new();
    if queue_entries.is_empty() {
        gaps.push("backpressure audit requires queued work to evaluate".to_string());
    }
    if queue_partitions.is_empty() {
        gaps.push("backpressure audit requires explicit queue partitions".to_string());
    }
    if !backlog_within_limit {
        gaps.push("queued backlog exceeds the declared control-plane capacity".to_string());
    }
    if !lag_within_limit {
        gaps.push("dispatch lag exceeds the allowed control-plane threshold".to_string());
    }
    if !enough_worker_slots {
        gaps.push("ready worker slots cannot absorb the current queue pressure".to_string());
    }
    if !hot_tenants_isolated {
        gaps.push("hot tenants are not isolated behind explicit queue partitions".to_string());
    }
    let report = BackpressureReport {
        queue_depth,
        partitioned_capacity,
        backlog_within_limit,
        lag_within_limit,
        enough_worker_slots,
        hot_tenants_isolated,
        backpressure_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.backpressure_ready;
    (serde_json::to_value(report).expect("backpressure report"), ok)
}

fn cache_payload(simulation: CacheSimulation) -> (serde_json::Value, bool) {
    let CacheSimulation {
        registry,
        dag_name,
        selection_policy,
        mut cache,
        current_policy_bundle_version,
        current_env,
        parent_env,
    } = simulation;
    let decision = select_dag_version(&registry, &dag_name, &selection_policy);
    let selected_version = match &decision {
        CompatibilityDecision::Selected { version_id, .. } => Some(version_id.clone()),
        CompatibilityDecision::Rejected { .. } => None,
    };
    let selection_resolved = matches!(decision, CompatibilityDecision::Selected { .. });
    let cache_entries_before = cache.entries.len();
    let had_stale_entries = cache
        .entries
        .iter()
        .any(|entry| entry.policy_bundle_version != current_policy_bundle_version);
    invalidate_decision_cache(&mut cache, &current_policy_bundle_version);
    let cache_entries_after = cache.entries.len();
    let stale_entries_removed = cache_entries_before.saturating_sub(cache_entries_after);
    let cache_disciplined = cache
        .entries
        .iter()
        .all(|entry| entry.policy_bundle_version == current_policy_bundle_version);
    let resolved_environment = resolve_environment_values(&current_env, parent_env.as_ref());
    let environment_resolved = !resolved_environment.is_empty();
    let mut gaps = Vec::new();
    if dag_name.trim().is_empty() {
        gaps.push("cache discipline audit requires a dag name".to_string());
    }
    if !selection_resolved {
        gaps.push("registry selection could not resolve an active dag version".to_string());
    }
    if had_stale_entries && stale_entries_removed == 0 {
        gaps.push("stale policy cache entries were not invalidated".to_string());
    }
    if !cache_disciplined {
        gaps.push("policy decision cache still mixes bundle versions".to_string());
    }
    if current_env.parent.is_some() && parent_env.is_none() {
        gaps.push("environment inheritance declares a parent but no parent configuration".to_string());
    }
    if !environment_resolved {
        gaps.push("effective environment values did not resolve".to_string());
    }
    let report = CacheReport {
        selected_version,
        cache_entries_before,
        cache_entries_after,
        stale_entries_removed,
        environment_keys: resolved_environment.len(),
        selection_resolved,
        cache_disciplined,
        environment_resolved,
        cache_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.cache_ready;
    (serde_json::to_value(report).expect("cache report"), ok)
}

fn migration_payload(simulation: MigrationSimulation) -> (serde_json::Value, bool) {
    let MigrationSimulation {
        api_version,
        compatibility_rule,
        source_registry,
        target_registry,
        dry_run_completed,
        rollback_plan_recorded,
        mixed_writer_blocked,
    } = simulation;
    let compatibility_ok = check_api_compatibility(&api_version, &compatibility_rule);
    let source_dag_count = source_registry.entries.len();
    let target_covers_all_dags = source_registry
        .entries
        .keys()
        .all(|dag_name| target_registry.entries.contains_key(dag_name));
    let mut gaps = Vec::new();
    if !compatibility_ok {
        gaps.push("target api version is outside the supported migration window".to_string());
    }
    if source_dag_count == 0 {
        gaps.push("migration audit requires at least one source registry entry".to_string());
    }
    if !target_covers_all_dags {
        gaps.push("target registry does not cover every source dag".to_string());
    }
    if !dry_run_completed {
        gaps.push("schema migration dry run was not completed".to_string());
    }
    if !rollback_plan_recorded {
        gaps.push("schema migration rollback plan is missing".to_string());
    }
    if !mixed_writer_blocked {
        gaps.push("mixed-version control-plane writes are not blocked".to_string());
    }
    let report = MigrationReport {
        compatibility_ok,
        source_dag_count,
        target_covers_all_dags,
        dry_run_completed,
        rollback_plan_recorded,
        mixed_writer_blocked,
        migration_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.migration_ready;
    (serde_json::to_value(report).expect("migration report"), ok)
}

fn fan_in_payload(simulation: FanInSimulation) -> (serde_json::Value, bool) {
    let FanInSimulation {
        timed_out_nodes,
        exited_nodes,
        fan_in_limit,
        aggregator_threads,
        required_audit_surfaces,
    } = simulation;
    let merged = merge_timeout_and_exit_events(&timed_out_nodes, &exited_nodes);
    let audited_surfaces =
        thread_safety_audit().into_iter().map(|record| record.surface).collect::<BTreeSet<_>>();
    let audit_surfaces_present =
        required_audit_surfaces.iter().all(|surface| audited_surfaces.contains(surface));
    let fan_in_within_limit = merged.len() <= fan_in_limit;
    let aggregator_threads_bounded = aggregator_threads <= fan_in_limit.max(1);
    let mut gaps = Vec::new();
    if merged.is_empty() {
        gaps.push("fan-in audit requires timeout or exit events".to_string());
    }
    if !fan_in_within_limit {
        gaps.push("merged timeout and exit events exceed the declared fan-in bound".to_string());
    }
    if !aggregator_threads_bounded {
        gaps.push("aggregator thread count exceeds the declared fan-in bound".to_string());
    }
    if !audit_surfaces_present {
        gaps.push("required thread-safety audit surfaces are missing".to_string());
    }
    let report = FanInReport {
        merged_event_count: merged.len(),
        fan_in_within_limit,
        aggregator_threads_bounded,
        audit_surfaces_present,
        fan_in_ready: gaps.is_empty(),
        gaps,
    };
    let ok = report.fan_in_ready;
    (serde_json::to_value(report).expect("fan-in report"), ok)
}

pub(crate) fn handle_control_plane_command(
    cli: &DagCli,
    command: &ControlPlaneCommands,
) -> Result<ExitCode, ExitCode> {
    let (surface, payload, ok) = match command {
        ControlPlaneCommands::Api { simulation } => {
            let simulation: ApiSimulation = parse_json_file(simulation)?;
            let (payload, ok) = api_payload(simulation);
            ("dag.control-plane.api", payload, ok)
        }
        ControlPlaneCommands::Leadership { simulation } => {
            let simulation: LeadershipSimulation = parse_json_file(simulation)?;
            let (payload, ok) = leadership_payload(simulation);
            ("dag.control-plane.leadership", payload, ok)
        }
        ControlPlaneCommands::Planning { simulation } => {
            let simulation: PlanningSimulation = parse_json_file(simulation)?;
            let (payload, ok) = planning_payload(simulation);
            ("dag.control-plane.planning", payload, ok)
        }
        ControlPlaneCommands::Sharding { simulation } => {
            let simulation: ShardingSimulation = parse_json_file(simulation)?;
            let (payload, ok) = sharding_payload(simulation);
            ("dag.control-plane.sharding", payload, ok)
        }
        ControlPlaneCommands::Leases { simulation } => {
            let simulation: LeasesSimulation = parse_json_file(simulation)?;
            let (payload, ok) = leases_payload(simulation);
            ("dag.control-plane.leases", payload, ok)
        }
        ControlPlaneCommands::Idempotency { simulation } => {
            let simulation: IdempotencySimulation = parse_json_file(simulation)?;
            let (payload, ok) = idempotency_payload(simulation);
            ("dag.control-plane.idempotency", payload, ok)
        }
        ControlPlaneCommands::Backpressure { simulation } => {
            let simulation: BackpressureSimulation = parse_json_file(simulation)?;
            let (payload, ok) = backpressure_payload(simulation);
            ("dag.control-plane.backpressure", payload, ok)
        }
        ControlPlaneCommands::Cache { simulation } => {
            let simulation: CacheSimulation = parse_json_file(simulation)?;
            let (payload, ok) = cache_payload(simulation);
            ("dag.control-plane.cache", payload, ok)
        }
        ControlPlaneCommands::Migration { simulation } => {
            let simulation: MigrationSimulation = parse_json_file(simulation)?;
            let (payload, ok) = migration_payload(simulation);
            ("dag.control-plane.migration", payload, ok)
        }
        ControlPlaneCommands::FanIn { simulation } => {
            let simulation: FanInSimulation = parse_json_file(simulation)?;
            let (payload, ok) = fan_in_payload(simulation);
            ("dag.control-plane.fan-in", payload, ok)
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
                "message":"control-plane architecture posture is incomplete",
                "remediation":"fix the reported control-plane gaps before treating this surface as production-ready"
            })]
        },
        if ok { ExitCode::SUCCESS } else { ExitCode::from(2) },
    )
}

#[cfg(test)]
mod tests {
    use super::{
        api_payload, backpressure_payload, cache_payload, leadership_payload, planning_payload,
        fan_in_payload, migration_payload, ApiSimulation, BackpressureSimulation,
        CacheSimulation, FanInSimulation, LeadershipSimulation, IdempotencySimulation,
        LeasesSimulation, MigrationSimulation, PlanningSimulation, ShardingSimulation,
        idempotency_payload, leases_payload, sharding_payload,
    };
    use bijux_dag_runtime::simulated_platform::{
        ApiCompatibilityRule, ApiVersion, DagRegistry, DagVersionRecord,
        DagVersionSelectionPolicy, DagVersionStatus, DecisionType, DurableRunQueueEntry,
        EnvironmentConfiguration, EnvironmentMode, LeaderElectionState, PolicyDecisionCache,
        PolicyDecisionCacheEntry, QueueOwnershipTransfer, QueuePartition, QueueShardLease,
        RegionId, RegionQueuePartition, RunControlOperation, ScheduleDedupRecord,
        SchedulerEpoch, SchedulerFenceToken, TaskLeaseSemantics, TypedControlPlaneRequest,
        TypedControlPlaneResponse, WorkLease,
    };
    use serde_json::json;
    use std::collections::{BTreeMap, BTreeSet};

    #[test]
    fn api_accepts_load_balanced_consistent_replicas() {
        let request = TypedControlPlaneRequest {
            operation: RunControlOperation::Submit,
            dag_name: "atlas.load".to_string(),
            run_id: Some("run-1".to_string()),
            payload: json!({"graph_fingerprint":"g1"}),
        };
        let response = TypedControlPlaneResponse {
            accepted: true,
            message: "accepted".to_string(),
            details: json!({"run_id":"run-1"}),
        };
        let simulation = ApiSimulation {
            request,
            replica_ids: vec!["api-a".to_string(), "api-b".to_string()],
            responses: vec![response.clone(), response],
            load_balanced: true,
            hidden_in_memory_authority: false,
        };
        let (payload, ok) = api_payload(simulation);
        assert!(ok);
        assert_eq!(payload["api_ready"], true);
    }

    #[test]
    fn api_flags_sticky_or_inconsistent_replicas() {
        let simulation = ApiSimulation {
            request: TypedControlPlaneRequest {
                operation: RunControlOperation::Submit,
                dag_name: "atlas.load".to_string(),
                run_id: Some("run-1".to_string()),
                payload: json!({"graph_fingerprint":"g1"}),
            },
            replica_ids: vec!["api-a".to_string()],
            responses: vec![
                TypedControlPlaneResponse {
                    accepted: true,
                    message: "accepted".to_string(),
                    details: json!({"run_id":"run-1"}),
                },
                TypedControlPlaneResponse {
                    accepted: false,
                    message: "rejected".to_string(),
                    details: json!({"run_id":"run-1"}),
                },
            ],
            load_balanced: false,
            hidden_in_memory_authority: true,
        };
        let (payload, ok) = api_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn leadership_accepts_consistent_fenced_owner() {
        let simulation = LeadershipSimulation {
            leader: LeaderElectionState {
                leader_replica_id: "scheduler-a".to_string(),
                lease_expires_unix_ms: 2_000,
                epoch: 7,
            },
            shard_lease: QueueShardLease {
                shard_id: "tenant-atlas".to_string(),
                owner_replica_id: "scheduler-a".to_string(),
                lease_expires_unix_ms: 2_000,
            },
            current_epoch: SchedulerEpoch { replica_id: "scheduler-a".to_string(), epoch: 7 },
            fence_token: SchedulerFenceToken {
                replica_id: "scheduler-a".to_string(),
                epoch: 7,
                token: "fence-7".to_string(),
            },
            now_unix_ms: 1_500,
        };
        let (payload, ok) = leadership_payload(simulation);
        assert!(ok);
        assert_eq!(payload["leadership_ready"], true);
    }

    #[test]
    fn leadership_flags_stale_or_mismatched_owner() {
        let simulation = LeadershipSimulation {
            leader: LeaderElectionState {
                leader_replica_id: "scheduler-a".to_string(),
                lease_expires_unix_ms: 100,
                epoch: 7,
            },
            shard_lease: QueueShardLease {
                shard_id: String::new(),
                owner_replica_id: "scheduler-b".to_string(),
                lease_expires_unix_ms: 2_000,
            },
            current_epoch: SchedulerEpoch { replica_id: "scheduler-b".to_string(), epoch: 8 },
            fence_token: SchedulerFenceToken {
                replica_id: "scheduler-a".to_string(),
                epoch: 7,
                token: "fence-7".to_string(),
            },
            now_unix_ms: 1_500,
        };
        let (payload, ok) = leadership_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn planning_accepts_persisted_reviewed_plan_before_dispatch() {
        let simulation = PlanningSimulation {
            dag_name: "atlas.load".to_string(),
            plan_digest: "plan-1".to_string(),
            plan_persisted_unix_ms: 1_000,
            dispatch_started_unix_ms: 1_500,
            queue_entry: DurableRunQueueEntry {
                queue_key: "atlas/default".to_string(),
                tenant_id: Some("atlas".to_string()),
                schedule_id: "sched-1".to_string(),
                run_key: "run-1".to_string(),
                created_unix_ms: 1_200,
            },
            review_recorded: true,
        };
        let (payload, ok) = planning_payload(simulation);
        assert!(ok);
        assert_eq!(payload["planning_ready"], true);
    }

    #[test]
    fn planning_flags_queued_work_without_durable_plan() {
        let simulation = PlanningSimulation {
            dag_name: String::new(),
            plan_digest: String::new(),
            plan_persisted_unix_ms: 2_000,
            dispatch_started_unix_ms: 1_500,
            queue_entry: DurableRunQueueEntry {
                queue_key: String::new(),
                tenant_id: None,
                schedule_id: String::new(),
                run_key: String::new(),
                created_unix_ms: 1_000,
            },
            review_recorded: false,
        };
        let (payload, ok) = planning_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn sharding_accepts_explicit_tenant_and_region_partitions() {
        let simulation = ShardingSimulation {
            queue_partitions: vec![
                QueuePartition {
                    queue_name: "atlas".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    max_concurrency: 32,
                },
                QueuePartition {
                    queue_name: "canon".to_string(),
                    tenant_id: Some("canon".to_string()),
                    max_concurrency: 16,
                },
            ],
            region_partitions: vec![
                RegionQueuePartition {
                    region: RegionId("eu-north".to_string()),
                    queue_name: "atlas-eu".to_string(),
                    shared_with_regions: BTreeSet::new(),
                },
                RegionQueuePartition {
                    region: RegionId("us-east".to_string()),
                    queue_name: "atlas-us".to_string(),
                    shared_with_regions: BTreeSet::new(),
                },
            ],
            active_tenants: vec!["atlas".to_string(), "canon".to_string()],
            active_regions: vec![RegionId("eu-north".to_string()), RegionId("us-east".to_string())],
        };
        let (payload, ok) = sharding_payload(simulation);
        assert!(ok);
        assert_eq!(payload["sharding_ready"], true);
    }

    #[test]
    fn sharding_flags_unassigned_or_overly_shared_partitions() {
        let simulation = ShardingSimulation {
            queue_partitions: vec![QueuePartition {
                queue_name: "atlas".to_string(),
                tenant_id: Some("atlas".to_string()),
                max_concurrency: 32,
            }],
            region_partitions: vec![RegionQueuePartition {
                region: RegionId("eu-north".to_string()),
                queue_name: "atlas-eu".to_string(),
                shared_with_regions: BTreeSet::from([
                    RegionId("us-east".to_string()),
                    RegionId("us-west".to_string()),
                ]),
            }],
            active_tenants: vec!["atlas".to_string(), "canon".to_string()],
            active_regions: vec![RegionId("eu-north".to_string()), RegionId("us-east".to_string())],
        };
        let (payload, ok) = sharding_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 2);
    }

    #[test]
    fn leases_accept_live_scheduler_and_worker_ownership() {
        let simulation = LeasesSimulation {
            shard_lease: QueueShardLease {
                shard_id: "atlas".to_string(),
                owner_replica_id: "scheduler-a".to_string(),
                lease_expires_unix_ms: 2_000,
            },
            ownership_transfer: QueueOwnershipTransfer {
                shard_id: "atlas".to_string(),
                from_replica_id: "scheduler-b".to_string(),
                to_replica_id: "scheduler-a".to_string(),
                transfer_unix_ms: 1_000,
            },
            work_lease: WorkLease {
                lease_id: "lease-1".to_string(),
                run_id: "run-1".to_string(),
                node_id: "node-a".to_string(),
                worker_id: "worker-a".to_string(),
                expires_unix_ms: 2_000,
            },
            semantics: TaskLeaseSemantics {
                lease_duration_ms: 5_000,
                renew_before_expiry_ms: 1_000,
                max_renewals: 3,
                recovery_grace_ms: 2_000,
            },
            now_unix_ms: 1_500,
        };
        let (payload, ok) = leases_payload(simulation);
        assert!(ok);
        assert_eq!(payload["leases_ready"], true);
    }

    #[test]
    fn leases_flag_expired_or_untracked_ownership() {
        let simulation = LeasesSimulation {
            shard_lease: QueueShardLease {
                shard_id: String::new(),
                owner_replica_id: "scheduler-a".to_string(),
                lease_expires_unix_ms: 100,
            },
            ownership_transfer: QueueOwnershipTransfer {
                shard_id: "other".to_string(),
                from_replica_id: "scheduler-b".to_string(),
                to_replica_id: "scheduler-c".to_string(),
                transfer_unix_ms: 1_000,
            },
            work_lease: WorkLease {
                lease_id: "lease-1".to_string(),
                run_id: "run-1".to_string(),
                node_id: "node-a".to_string(),
                worker_id: String::new(),
                expires_unix_ms: 100,
            },
            semantics: TaskLeaseSemantics {
                lease_duration_ms: 0,
                renew_before_expiry_ms: 0,
                max_renewals: 0,
                recovery_grace_ms: 0,
            },
            now_unix_ms: 1_500,
        };
        let (payload, ok) = leases_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 5);
    }

    #[test]
    fn idempotency_accepts_replayed_commands_with_one_canonical_run() {
        let simulation = IdempotencySimulation {
            existing_dedup: BTreeMap::new(),
            repeated_dedup_key: "atlas|2026-04-28T10".to_string(),
            first_run_key: "run-1".to_string(),
            replayed_run_key: "run-2".to_string(),
            replica_records: Vec::new(),
            proposed_record: ScheduleDedupRecord {
                dedup_key: "atlas|2026-04-28T10".to_string(),
                run_key: "run-1".to_string(),
                epoch: 4,
            },
            queue_entries: vec![
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-a".to_string(),
                    run_key: "run-1".to_string(),
                    created_unix_ms: 100,
                },
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-b".to_string(),
                    run_key: "run-2".to_string(),
                    created_unix_ms: 200,
                },
            ],
        };
        let (payload, ok) = idempotency_payload(simulation);
        assert!(ok);
        assert_eq!(payload["idempotency_ready"], true);
        assert_eq!(payload["canonical_run_key"], "run-1");
    }

    #[test]
    fn idempotency_flags_duplicate_dedup_or_unstable_ordering() {
        let simulation = IdempotencySimulation {
            existing_dedup: BTreeMap::new(),
            repeated_dedup_key: String::new(),
            first_run_key: "run-1".to_string(),
            replayed_run_key: "run-2".to_string(),
            replica_records: vec![ScheduleDedupRecord {
                dedup_key: "atlas|2026-04-28T10".to_string(),
                run_key: "run-0".to_string(),
                epoch: 3,
            }],
            proposed_record: ScheduleDedupRecord {
                dedup_key: "atlas|2026-04-28T10".to_string(),
                run_key: "run-2".to_string(),
                epoch: 4,
            },
            queue_entries: vec![
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-b".to_string(),
                    run_key: "run-2".to_string(),
                    created_unix_ms: 200,
                },
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-a".to_string(),
                    run_key: "run-1".to_string(),
                    created_unix_ms: 100,
                },
            ],
        };
        let (payload, ok) = idempotency_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn backpressure_accepts_partitioned_capacity_with_low_lag() {
        let simulation = BackpressureSimulation {
            queue_entries: vec![
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-a".to_string(),
                    run_key: "run-1".to_string(),
                    created_unix_ms: 100,
                },
                DurableRunQueueEntry {
                    queue_key: "canon/default".to_string(),
                    tenant_id: Some("canon".to_string()),
                    schedule_id: "sched-b".to_string(),
                    run_key: "run-2".to_string(),
                    created_unix_ms: 200,
                },
            ],
            queue_partitions: vec![
                QueuePartition {
                    queue_name: "atlas".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    max_concurrency: 4,
                },
                QueuePartition {
                    queue_name: "canon".to_string(),
                    tenant_id: Some("canon".to_string()),
                    max_concurrency: 4,
                },
            ],
            dispatch_lag_ms: 500,
            ready_worker_slots: 4,
            high_watermark: 8,
            max_dispatch_lag_ms: 1_000,
            hot_tenants: vec!["atlas".to_string(), "canon".to_string()],
        };
        let (payload, ok) = backpressure_payload(simulation);
        assert!(ok);
        assert_eq!(payload["backpressure_ready"], true);
    }

    #[test]
    fn backpressure_flags_shared_hotspots_and_queue_saturation() {
        let simulation = BackpressureSimulation {
            queue_entries: vec![
                DurableRunQueueEntry {
                    queue_key: "atlas/default".to_string(),
                    tenant_id: Some("atlas".to_string()),
                    schedule_id: "sched-a".to_string(),
                    run_key: "run-1".to_string(),
                    created_unix_ms: 100,
                },
                DurableRunQueueEntry {
                    queue_key: "canon/default".to_string(),
                    tenant_id: Some("canon".to_string()),
                    schedule_id: "sched-b".to_string(),
                    run_key: "run-2".to_string(),
                    created_unix_ms: 200,
                },
                DurableRunQueueEntry {
                    queue_key: "canon/default".to_string(),
                    tenant_id: Some("canon".to_string()),
                    schedule_id: "sched-c".to_string(),
                    run_key: "run-3".to_string(),
                    created_unix_ms: 300,
                },
            ],
            queue_partitions: vec![QueuePartition {
                queue_name: "atlas".to_string(),
                tenant_id: Some("atlas".to_string()),
                max_concurrency: 1,
            }],
            dispatch_lag_ms: 5_000,
            ready_worker_slots: 1,
            high_watermark: 2,
            max_dispatch_lag_ms: 1_000,
            hot_tenants: vec!["atlas".to_string(), "canon".to_string()],
        };
        let (payload, ok) = backpressure_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn cache_accepts_one_registry_line_and_one_policy_bundle_version() {
        let mut registry = DagRegistry::default();
        registry.entries.insert(
            "atlas.load".to_string(),
            bijux_dag_runtime::simulated_platform::DagRegistryEntry {
                dag_name: "atlas.load".to_string(),
                owner: "atlas".to_string(),
                tags: vec!["critical".to_string()],
                versions: vec![DagVersionRecord {
                    version_id: "2026.04.28".to_string(),
                    compatibility_line: "v1".to_string(),
                    status: DagVersionStatus::Active,
                    created_unix_ms: 100,
                }],
            },
        );
        let simulation = CacheSimulation {
            registry,
            dag_name: "atlas.load".to_string(),
            selection_policy: DagVersionSelectionPolicy::RunLatest,
            cache: PolicyDecisionCache {
                entries: vec![PolicyDecisionCacheEntry {
                    cache_key: "k1".to_string(),
                    decision: DecisionType::Allow,
                    policy_bundle_version: "2026.04".to_string(),
                }],
            },
            current_policy_bundle_version: "2026.04".to_string(),
            current_env: EnvironmentConfiguration {
                mode: EnvironmentMode::Production,
                parent: Some("shared".to_string()),
                values: BTreeMap::from([("queue".to_string(), "atlas".to_string())]),
                overrides: BTreeMap::from([("max_parallelism".to_string(), "8".to_string())]),
            },
            parent_env: Some(EnvironmentConfiguration {
                mode: EnvironmentMode::Staging,
                parent: None,
                values: BTreeMap::from([("region".to_string(), "eu-north".to_string())]),
                overrides: BTreeMap::new(),
            }),
        };
        let (payload, ok) = cache_payload(simulation);
        assert!(ok);
        assert_eq!(payload["cache_ready"], true);
        assert_eq!(payload["selected_version"], "2026.04.28");
    }

    #[test]
    fn cache_flags_unresolved_selection_or_missing_parent_overlay() {
        let simulation = CacheSimulation {
            registry: DagRegistry::default(),
            dag_name: String::new(),
            selection_policy: DagVersionSelectionPolicy::RunLatest,
            cache: PolicyDecisionCache {
                entries: vec![
                    PolicyDecisionCacheEntry {
                        cache_key: "k1".to_string(),
                        decision: DecisionType::Allow,
                        policy_bundle_version: "2026.03".to_string(),
                    },
                    PolicyDecisionCacheEntry {
                        cache_key: "k2".to_string(),
                        decision: DecisionType::Deny,
                        policy_bundle_version: "2026.02".to_string(),
                    },
                ],
            },
            current_policy_bundle_version: "2026.04".to_string(),
            current_env: EnvironmentConfiguration {
                mode: EnvironmentMode::Production,
                parent: Some("shared".to_string()),
                values: BTreeMap::new(),
                overrides: BTreeMap::new(),
            },
            parent_env: None,
        };
        let (payload, ok) = cache_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }

    #[test]
    fn migration_accepts_compatible_api_and_guarded_rollout() {
        let source_registry = DagRegistry {
            entries: BTreeMap::from([(
                "atlas.load".to_string(),
                bijux_dag_runtime::simulated_platform::DagRegistryEntry {
                    dag_name: "atlas.load".to_string(),
                    owner: "atlas".to_string(),
                    tags: vec!["critical".to_string()],
                    versions: vec![DagVersionRecord {
                        version_id: "2026.04.28".to_string(),
                        compatibility_line: "v1".to_string(),
                        status: DagVersionStatus::Active,
                        created_unix_ms: 100,
                    }],
                },
            )]),
        };
        let target_registry = source_registry.clone();
        let simulation = MigrationSimulation {
            api_version: ApiVersion { major: 1, minor: 2 },
            compatibility_rule: ApiCompatibilityRule {
                min_supported_major: 1,
                max_supported_major: 2,
                supports_minor_additive_fields: true,
            },
            source_registry,
            target_registry,
            dry_run_completed: true,
            rollback_plan_recorded: true,
            mixed_writer_blocked: true,
        };
        let (payload, ok) = migration_payload(simulation);
        assert!(ok);
        assert_eq!(payload["migration_ready"], true);
    }

    #[test]
    fn migration_flags_incompatible_api_or_uncovered_registry() {
        let source_registry = DagRegistry {
            entries: BTreeMap::from([(
                "atlas.load".to_string(),
                bijux_dag_runtime::simulated_platform::DagRegistryEntry {
                    dag_name: "atlas.load".to_string(),
                    owner: "atlas".to_string(),
                    tags: vec!["critical".to_string()],
                    versions: vec![DagVersionRecord {
                        version_id: "2026.04.28".to_string(),
                        compatibility_line: "v1".to_string(),
                        status: DagVersionStatus::Active,
                        created_unix_ms: 100,
                    }],
                },
            )]),
        };
        let simulation = MigrationSimulation {
            api_version: ApiVersion { major: 3, minor: 0 },
            compatibility_rule: ApiCompatibilityRule {
                min_supported_major: 1,
                max_supported_major: 2,
                supports_minor_additive_fields: false,
            },
            source_registry,
            target_registry: DagRegistry::default(),
            dry_run_completed: false,
            rollback_plan_recorded: false,
            mixed_writer_blocked: false,
        };
        let (payload, ok) = migration_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 4);
    }

    #[test]
    fn fan_in_accepts_bounded_event_merge_with_audited_surfaces() {
        let simulation = FanInSimulation {
            timed_out_nodes: vec!["node-a".to_string()],
            exited_nodes: vec!["node-b".to_string()],
            fan_in_limit: 4,
            aggregator_threads: 2,
            required_audit_surfaces: vec![
                "scheduler_state".to_string(),
                "trace_write_ledger".to_string(),
            ],
        };
        let (payload, ok) = fan_in_payload(simulation);
        assert!(ok);
        assert_eq!(payload["fan_in_ready"], true);
    }

    #[test]
    fn fan_in_flags_unbounded_merge_or_missing_audit_coverage() {
        let simulation = FanInSimulation {
            timed_out_nodes: vec![
                "node-a".to_string(),
                "node-b".to_string(),
                "node-c".to_string(),
            ],
            exited_nodes: vec!["node-d".to_string()],
            fan_in_limit: 2,
            aggregator_threads: 3,
            required_audit_surfaces: vec!["missing_surface".to_string()],
        };
        let (payload, ok) = fan_in_payload(simulation);
        assert!(!ok);
        assert!(payload["gaps"].as_array().expect("gaps").len() >= 3);
    }
}
