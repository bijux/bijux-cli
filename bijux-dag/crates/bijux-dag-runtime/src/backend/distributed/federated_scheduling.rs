use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SchedulerDomainId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederationDomainIdentity {
    pub domain_id: SchedulerDomainId,
    pub trust_tier: String,
    pub issuer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunDelegationRecord {
    pub run_id: String,
    pub parent_domain: SchedulerDomainId,
    pub child_domain: SchedulerDomainId,
    pub reason: String,
    pub deterministic_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossClusterRoutingPolicy {
    pub preferred_regions: BTreeSet<String>,
    pub tenant_overrides: BTreeMap<String, SchedulerDomainId>,
    pub backend_class_routes: BTreeMap<String, SchedulerDomainId>,
    pub locality_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerPeeringRule {
    pub from_domain: SchedulerDomainId,
    pub to_domain: SchedulerDomainId,
    pub overflow_enabled: bool,
    pub burst_share_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederatedBackfillPlan {
    pub batch_id: String,
    pub domains: Vec<SchedulerDomainId>,
    pub partition_count: usize,
    pub deterministic_partitioning: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InterSchedulerFlowControl {
    pub source_domain: SchedulerDomainId,
    pub target_domain: SchedulerDomainId,
    pub max_inflight_delegations: usize,
    pub max_delegations_per_minute: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainCapabilityAdvertisement {
    pub domain_id: SchedulerDomainId,
    pub backend_classes: BTreeSet<String>,
    pub storage_classes: BTreeSet<String>,
    pub policy_strictness: String,
    pub trust_labels: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainHealthSnapshot {
    pub domain_id: SchedulerDomainId,
    pub healthy: bool,
    pub impairment_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederatedScheduleSuppression {
    pub domain_id: SchedulerDomainId,
    pub reason: String,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossDomainReplaySafety {
    pub artifact_compatible: bool,
    pub policy_compatible: bool,
    pub backend_compatible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DomainRoutingExplanation {
    pub run_id: String,
    pub selected_domain: SchedulerDomainId,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DelegationFailureAction {
    RetrySameDomain,
    Reroute,
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DelegationFailurePolicy {
    pub transient_action: DelegationFailureAction,
    pub persistent_action: DelegationFailureAction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederatedSimulationScenario {
    pub name: String,
    pub overflow_burst_factor: u32,
    pub failover_domain: Option<SchedulerDomainId>,
    pub policy_conflict: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeeringObservabilityContract {
    pub exchange_metrics: bool,
    pub exchange_audit_events: bool,
    pub redaction_profile: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederationConcurrencyControl {
    pub global_limit: usize,
    pub local_limits: BTreeMap<SchedulerDomainId, usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TrustTierRoutingRule {
    pub min_trust_tier: String,
    pub allowed_domains: BTreeSet<SchedulerDomainId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederationMaturityMatrix {
    pub single_domain: String,
    pub active_passive: String,
    pub overflow_peering: String,
    pub full_multi_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FederatedConformanceGate {
    pub lineage_auditable: bool,
    pub routing_deterministic: bool,
    pub audit_events_complete: bool,
}

pub fn domain_healthy(domain: &SchedulerDomainId, health: &[DomainHealthSnapshot]) -> bool {
    health
        .iter()
        .find(|snapshot| &snapshot.domain_id == domain)
        .map(|snapshot| snapshot.healthy)
        .unwrap_or(false)
}

pub fn delegation_allowed(
    flow: &InterSchedulerFlowControl,
    inflight: usize,
    per_minute: usize,
) -> bool {
    inflight < flow.max_inflight_delegations && per_minute < flow.max_delegations_per_minute
}

pub fn cross_domain_replay_safe(safety: &CrossDomainReplaySafety) -> bool {
    safety.artifact_compatible && safety.policy_compatible && safety.backend_compatible
}

pub fn select_delegation_failure_action(
    policy: &DelegationFailurePolicy,
    persistent_failure: bool,
) -> DelegationFailureAction {
    if persistent_failure {
        policy.persistent_action.clone()
    } else {
        policy.transient_action.clone()
    }
}

pub fn trust_tier_allows_domain(rule: &TrustTierRoutingRule, domain: &SchedulerDomainId) -> bool {
    rule.allowed_domains.contains(domain)
}

pub fn federation_conformance_passes(gate: &FederatedConformanceGate) -> bool {
    gate.lineage_auditable && gate.routing_deterministic && gate.audit_events_complete
}

pub fn default_federation_maturity_matrix() -> FederationMaturityMatrix {
    FederationMaturityMatrix {
        single_domain: "local scheduling only".to_string(),
        active_passive: "failover ready with one standby domain".to_string(),
        overflow_peering: "burst sharing with deterministic delegation".to_string(),
        full_multi_domain: "coordinated routing with global policy and audit exchange".to_string(),
    }
}
