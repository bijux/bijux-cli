use crate::commands::{ControlPlaneCommands, DagCli};
use crate::{emit_json, read_file, ExitCode};
use bijux_dag_runtime::simulated_platform::{
    fence_allows_mutation, is_stale_leader, next_epoch, DurableRunQueueEntry,
    LeaderElectionState, QueueShardLease, SchedulerEpoch, SchedulerFenceToken,
    TypedControlPlaneRequest, TypedControlPlaneResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
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
        PlanningSimulation,
    };
    use bijux_dag_runtime::simulated_platform::{
        DurableRunQueueEntry, LeaderElectionState, QueueShardLease, RunControlOperation,
        SchedulerEpoch, SchedulerFenceToken, TypedControlPlaneRequest, TypedControlPlaneResponse,
    };
    use serde_json::json;

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
}
