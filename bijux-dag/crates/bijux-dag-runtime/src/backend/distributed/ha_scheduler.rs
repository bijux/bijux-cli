use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerEpoch {
    pub replica_id: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LeaderElectionState {
    pub leader_replica_id: String,
    pub lease_expires_unix_ms: u128,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableSchedulerTick {
    pub tick_id: String,
    pub evaluated_unix_ms: u128,
    pub schedule_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableRunQueueEntry {
    pub queue_key: String,
    pub tenant_id: Option<String>,
    pub schedule_id: String,
    pub run_key: String,
    pub created_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurableInFlightDispatch {
    pub run_key: String,
    pub node_id: String,
    pub worker_id: Option<String>,
    pub dispatched_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueShardLease {
    pub shard_id: String,
    pub owner_replica_id: String,
    pub lease_expires_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerFenceToken {
    pub replica_id: String,
    pub epoch: u64,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QueueOwnershipTransfer {
    pub shard_id: String,
    pub from_replica_id: String,
    pub to_replica_id: String,
    pub transfer_unix_ms: u128,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleDedupRecord {
    pub dedup_key: String,
    pub run_key: String,
    pub epoch: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerClockAssumption {
    pub max_clock_skew_ms: u64,
    pub tick_grace_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulerAuditEventKind {
    LeaderElection,
    Failover,
    QueueRebalance,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerAuditEvent {
    pub kind: SchedulerAuditEventKind,
    pub replica_id: String,
    pub unix_ms: u128,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerRecoveryObjectives {
    pub cold_restart_rto_ms: u64,
    pub failover_rto_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaSimulationScenario {
    pub scenario_id: String,
    pub replicas: Vec<String>,
    pub shards: Vec<String>,
    pub trigger_storm_size: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaMilestoneDefinition {
    pub milestone_name: String,
    pub supports_durable_queue: bool,
    pub supports_fencing: bool,
    pub supports_deduplication: bool,
    pub excludes_multi_zone: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HaConformanceReport {
    pub no_duplicate_runs: bool,
    pub stale_leader_fenced: bool,
    pub sequencing_preserved: bool,
    pub failures: Vec<String>,
}

pub trait DurableSchedulerStateStore {
    fn load_leader_state(&self) -> Result<Option<LeaderElectionState>, String>;
    fn save_leader_state(&self, state: &LeaderElectionState) -> Result<(), String>;
    fn append_queue_entry(&self, entry: &DurableRunQueueEntry) -> Result<(), String>;
    fn list_queue_entries(&self) -> Result<Vec<DurableRunQueueEntry>, String>;
}

pub fn is_stale_leader(leader: &LeaderElectionState, now_unix_ms: u128) -> bool {
    now_unix_ms > leader.lease_expires_unix_ms
}

pub fn next_epoch(current: &SchedulerEpoch) -> SchedulerEpoch {
    SchedulerEpoch {
        replica_id: current.replica_id.clone(),
        epoch: current.epoch + 1,
    }
}

pub fn idempotent_run_creation(
    dedup: &mut BTreeMap<String, String>,
    dedup_key: &str,
    run_key: &str,
) -> String {
    if let Some(existing) = dedup.get(dedup_key) {
        return existing.clone();
    }
    dedup.insert(dedup_key.to_string(), run_key.to_string());
    run_key.to_string()
}

pub fn deduplicate_across_replicas(
    existing: &[ScheduleDedupRecord],
    proposed: &ScheduleDedupRecord,
) -> bool {
    !existing.iter().any(|r| r.dedup_key == proposed.dedup_key)
}

pub fn ordering_during_failover(
    mut entries: Vec<DurableRunQueueEntry>,
) -> Vec<DurableRunQueueEntry> {
    entries.sort_by(|a, b| {
        a.created_unix_ms
            .cmp(&b.created_unix_ms)
            .then_with(|| a.schedule_id.cmp(&b.schedule_id))
            .then_with(|| a.run_key.cmp(&b.run_key))
    });
    entries
}

pub fn clock_within_assumption(
    local_unix_ms: u128,
    reference_unix_ms: u128,
    assumption: &SchedulerClockAssumption,
) -> bool {
    let skew = local_unix_ms.abs_diff(reference_unix_ms);
    skew <= assumption.max_clock_skew_ms as u128
}

pub fn fence_allows_mutation(token: &SchedulerFenceToken, epoch: &SchedulerEpoch) -> bool {
    token.replica_id == epoch.replica_id && token.epoch == epoch.epoch
}

pub fn failover_recovery_passes(
    measured_cold_restart_ms: u64,
    measured_failover_ms: u64,
    objectives: &SchedulerRecoveryObjectives,
) -> bool {
    measured_cold_restart_ms <= objectives.cold_restart_rto_ms
        && measured_failover_ms <= objectives.failover_rto_ms
}

pub fn conformance_no_duplicate_runs(run_keys: &[String]) -> bool {
    let unique = run_keys.iter().cloned().collect::<BTreeSet<_>>();
    unique.len() == run_keys.len()
}

pub fn evaluate_ha_conformance(
    run_keys: &[String],
    stale_leader_fenced: bool,
    sequencing_preserved: bool,
) -> HaConformanceReport {
    let no_duplicate_runs = conformance_no_duplicate_runs(run_keys);
    let mut failures = Vec::new();
    if !no_duplicate_runs {
        failures.push("duplicate scheduled runs detected".to_string());
    }
    if !stale_leader_fenced {
        failures.push("stale leader performed a mutation".to_string());
    }
    if !sequencing_preserved {
        failures.push("run submission sequencing violated".to_string());
    }
    HaConformanceReport {
        no_duplicate_runs,
        stale_leader_fenced,
        sequencing_preserved,
        failures,
    }
}
