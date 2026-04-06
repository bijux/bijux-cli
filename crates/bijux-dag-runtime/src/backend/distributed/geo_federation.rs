use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct RegionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsistencyClass {
    StronglyConsistent,
    RegionallyConsistent,
    EventuallyReplicated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionAwareDagActivation {
    pub dag_name: String,
    pub version: String,
    pub global_visibility: bool,
    pub active_regions: BTreeSet<RegionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionScheduleRule {
    pub region: RegionId,
    pub timezone: String,
    pub failover_regions: Vec<RegionId>,
    pub utc_anchor_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionQueuePartition {
    pub region: RegionId,
    pub queue_name: String,
    pub shared_with_regions: BTreeSet<RegionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionAffinityPolicy {
    pub dag_regions: BTreeSet<RegionId>,
    pub run_regions: BTreeSet<RegionId>,
    pub artifact_regions: BTreeSet<RegionId>,
    pub tenant_regions: BTreeSet<RegionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossRegionFailoverRule {
    pub service: String,
    pub primary_region: RegionId,
    pub secondary_regions: Vec<RegionId>,
    pub max_failover_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionalReplicaOwnership {
    pub region: RegionId,
    pub owns_registry_writes: bool,
    pub owns_scheduler_evaluation: bool,
    pub lease_ttl_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WriteRoutingRule {
    pub resource: String,
    pub global_visible: bool,
    pub write_regions: BTreeSet<RegionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsistencyBoundaryNote {
    pub resource: String,
    pub class: ConsistencyClass,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionLineageRecord {
    pub artifact_id: String,
    pub producer_region: RegionId,
    pub consumer_regions: BTreeSet<RegionId>,
    pub lineage_queryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionObservabilityPartition {
    pub region: RegionId,
    pub local_retention_days: u32,
    pub aggregate_to_global: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionPolicyOverlay {
    pub region: RegionId,
    pub regulatory_profile: String,
    pub cost_profile: String,
    pub infrastructure_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionBackendRegistry {
    pub region: RegionId,
    pub backend_classes: BTreeSet<String>,
    pub routing_hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterRegionReplicationPolicy {
    pub artifact_classes: BTreeSet<String>,
    pub run_metadata_replicated: bool,
    pub audit_logs_replicated: bool,
    pub replication_rpo_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DisasterRecoveryPlaybook {
    pub region: RegionId,
    pub control_plane_outage_steps: Vec<String>,
    pub artifact_store_outage_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegionMigrationWorkflow {
    pub entity_kind: String,
    pub source_region: RegionId,
    pub target_region: RegionId,
    pub deterministic_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SplitBrainMitigationPlan {
    pub detection_signals: Vec<String>,
    pub mitigation_actions: Vec<String>,
    pub fencing_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeoSimulationScenario {
    pub name: String,
    pub replication_lag_seconds: u32,
    pub region_loss: Option<RegionId>,
    pub delayed_failover_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GeoReadyAcceptanceGate {
    pub registry_ready: bool,
    pub scheduler_ready: bool,
    pub lineage_ready: bool,
    pub observability_ready: bool,
}

pub fn region_write_allowed(rule: &WriteRoutingRule, region: &RegionId) -> bool {
    rule.write_regions.contains(region)
}

pub fn classify_resource_consistency(
    resource: &str,
    overrides: &[ConsistencyBoundaryNote],
) -> ConsistencyClass {
    overrides
        .iter()
        .find(|note| note.resource == resource)
        .map(|note| note.class.clone())
        .unwrap_or(ConsistencyClass::EventuallyReplicated)
}

pub fn geo_ready(gate: &GeoReadyAcceptanceGate) -> bool {
    gate.registry_ready && gate.scheduler_ready && gate.lineage_ready && gate.observability_ready
}

pub fn default_split_brain_mitigation() -> SplitBrainMitigationPlan {
    SplitBrainMitigationPlan {
        detection_signals: vec![
            "dual-leader-epoch-detected".to_string(),
            "conflicting-queue-ownership".to_string(),
            "replica-write-divergence".to_string(),
        ],
        mitigation_actions: vec![
            "issue-fencing-token-rotation".to_string(),
            "freeze-secondary-writers".to_string(),
            "reconcile-authoritative-log".to_string(),
        ],
        fencing_required: true,
    }
}

pub fn build_consistency_catalog(
    entries: &[ConsistencyBoundaryNote],
) -> BTreeMap<String, ConsistencyClass> {
    let mut catalog = BTreeMap::new();
    for entry in entries {
        catalog.insert(entry.resource.clone(), entry.class.clone());
    }
    catalog
}
