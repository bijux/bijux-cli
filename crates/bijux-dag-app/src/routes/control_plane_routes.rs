use crate::commands::{ControlPlaneCommands, DagCli};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    fence_allows_mutation, is_stale_leader, next_epoch, validate_task_lease_semantics,
    DurableRunQueueEntry, LeaderElectionState, QueueOwnershipTransfer, QueuePartition,
    QueueShardLease, RegionId, RegionQueuePartition, SchedulerEpoch, SchedulerFenceToken,
    TaskLeaseSemantics, TypedControlPlaneRequest, TypedControlPlaneResponse, WorkLease,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeSet;
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
        api_payload, leadership_payload, planning_payload, ApiSimulation, LeadershipSimulation,
        LeasesSimulation, PlanningSimulation, ShardingSimulation, leases_payload,
        sharding_payload,
    };
    use bijux_dag_runtime::simulated_platform::{
        DurableRunQueueEntry, LeaderElectionState, QueueOwnershipTransfer, QueuePartition,
        QueueShardLease, RegionId, RegionQueuePartition, RunControlOperation, SchedulerEpoch,
        SchedulerFenceToken, TaskLeaseSemantics, TypedControlPlaneRequest,
        TypedControlPlaneResponse, WorkLease,
    };
    use serde_json::json;
    use std::collections::BTreeSet;

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
}
