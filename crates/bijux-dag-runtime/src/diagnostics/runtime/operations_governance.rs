use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformOperatingModel {
    pub dag_author: Vec<String>,
    pub operator: Vec<String>,
    pub releaser: Vec<String>,
    pub tenant_admin: Vec<String>,
    pub platform_admin: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceLevelObjective {
    pub run_creation_latency_ms: f64,
    pub dispatch_latency_ms: f64,
    pub completion_reliability_ratio: f64,
    pub artifact_availability_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceLevelIndicators {
    pub measured_run_creation_latency_ms: f64,
    pub measured_dispatch_latency_ms: f64,
    pub measured_completion_reliability_ratio: f64,
    pub measured_artifact_availability_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ErrorBudgetPolicy {
    pub scheduler_outage_minutes_per_quarter: f64,
    pub backend_degradation_minutes_per_quarter: f64,
    pub artifact_corruption_incident_budget: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum IncidentSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IncidentClassification {
    pub incident_type: String,
    pub severity: IncidentSeverity,
    pub routing: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunbookEntry {
    pub name: String,
    pub trigger: String,
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GamedayScenario {
    pub name: String,
    pub failure_class: String,
    pub expected_outcome: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PostmortemTemplate {
    pub required_sections: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseGovernancePolicy {
    pub requires_evidence_bundle: bool,
    pub requires_compatibility_results: bool,
    pub requires_rollback_plan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OperatorTrainingCatalog {
    pub interventions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuditReadinessChecklist {
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlatformHealthDashboard {
    pub engine_health: f64,
    pub scheduler_health: f64,
    pub artifact_store_health: f64,
    pub auth_health: f64,
    pub policy_health: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupportabilityModel {
    pub official_plugins: BTreeSet<String>,
    pub supported_backends: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductBoundary {
    pub platform_guarantees: Vec<String>,
    pub operator_responsibilities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoadmapGovernance {
    pub requires_contract_coverage: bool,
    pub requires_docs_coverage: bool,
    pub requires_operational_readiness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleGovernanceRule {
    pub feature_name: String,
    pub state: String,
    pub decision_due_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformAcceptanceBoard {
    pub members: Vec<String>,
    pub preview_to_stable_criteria: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SustainabilityOwnership {
    pub subsystem_owners: BTreeMap<String, String>,
    pub review_routing: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformInvariantCatalog {
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegratedVerificationLane {
    pub name: String,
    pub required_domains: Vec<String>,
    pub required_evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SloEvaluation {
    pub passed: bool,
    pub violations: Vec<String>,
}

pub fn evaluate_slo(objective: &ServiceLevelObjective, indicators: &ServiceLevelIndicators) -> SloEvaluation {
    let mut violations = Vec::new();
    if indicators.measured_run_creation_latency_ms > objective.run_creation_latency_ms {
        violations.push("run creation latency SLO missed".to_string());
    }
    if indicators.measured_dispatch_latency_ms > objective.dispatch_latency_ms {
        violations.push("dispatch latency SLO missed".to_string());
    }
    if indicators.measured_completion_reliability_ratio < objective.completion_reliability_ratio {
        violations.push("completion reliability SLO missed".to_string());
    }
    if indicators.measured_artifact_availability_ratio < objective.artifact_availability_ratio {
        violations.push("artifact availability SLO missed".to_string());
    }
    SloEvaluation {
        passed: violations.is_empty(),
        violations,
    }
}

pub fn release_policy_allows(release: &ReleaseGovernancePolicy) -> bool {
    release.requires_evidence_bundle
        && release.requires_compatibility_results
        && release.requires_rollback_plan
}

pub fn health_dashboard_score(dashboard: &PlatformHealthDashboard) -> f64 {
    (dashboard.engine_health
        + dashboard.scheduler_health
        + dashboard.artifact_store_health
        + dashboard.auth_health
        + dashboard.policy_health)
        / 5.0
}

pub fn invariant_catalog_default() -> PlatformInvariantCatalog {
    PlatformInvariantCatalog {
        invariants: vec![
            "deterministic planning for identical graph and policy input".to_string(),
            "artifact content identity must remain immutable".to_string(),
            "tenant isolation boundaries cannot be bypassed".to_string(),
            "authorization deny rules are final".to_string(),
            "replay evidence must preserve provenance visibility".to_string(),
        ],
    }
}

pub fn integrated_verification_lane_default() -> IntegratedVerificationLane {
    IntegratedVerificationLane {
        name: "platform-integrated-verification".to_string(),
        required_domains: vec![
            "multi-tenant".to_string(),
            "ha-scheduler".to_string(),
            "policy-enforcement".to_string(),
            "backend-execution".to_string(),
            "artifact-lineage".to_string(),
            "compatibility-governance".to_string(),
        ],
        required_evidence: vec![
            "conformance-suite-report".to_string(),
            "compatibility-dashboard".to_string(),
            "attestation-verification-report".to_string(),
            "scheduler-failover-simulation-report".to_string(),
        ],
    }
}
