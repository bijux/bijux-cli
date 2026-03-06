use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkflowTemplateKind {
    Etl,
    MlTraining,
    ValidationPromote,
    ReportGeneration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowTemplate {
    pub template_id: String,
    pub kind: WorkflowTemplateKind,
    pub required_nodes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SubworkflowInvocation {
    pub parent_workflow: String,
    pub child_workflow: String,
    pub deterministic_binding: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowContractInheritance {
    pub base_contract: String,
    pub inherited_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ApprovalGateNode {
    pub node_id: String,
    pub policy_ref: String,
    pub timeout_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HumanWaitState {
    pub wait_id: String,
    pub resume_token: String,
    pub deterministic_state_snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowEvent {
    pub event_type: String,
    pub payload_schema: String,
    pub downstream_trigger: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MultiDagTransaction {
    pub transaction_id: String,
    pub dag_refs: Vec<String>,
    pub publication_atomic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowPortfolio {
    pub portfolio_id: String,
    pub dag_refs: Vec<String>,
    pub schedule_refs: Vec<String>,
    pub dataset_refs: Vec<String>,
    pub policy_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RolloutWorkflow {
    pub rollout_id: String,
    pub source_environment: String,
    pub target_environments: Vec<String>,
    pub progressive_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowVerificationPlan {
    pub workflow_id: String,
    pub reproducibility_checks: Vec<String>,
    pub policy_baseline_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyComposedBlueprint {
    pub blueprint_id: String,
    pub required_policy_bundle: String,
    pub guarded_workflow_template: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowQualityGate {
    pub gate_id: String,
    pub requires_dataset_checks: bool,
    pub requires_artifact_verification: bool,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowScenarioTest {
    pub scenario_id: String,
    pub workflow_family: String,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowProductMetadata {
    pub workflow_id: String,
    pub owner: String,
    pub sla: String,
    pub lineage_surface: String,
    pub cost_profile: String,
    pub compliance_posture: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortfolioObservabilitySummary {
    pub portfolio_id: String,
    pub healthy_workflows: usize,
    pub unhealthy_workflows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkflowFamilyImpactAnalysis {
    pub family_id: String,
    pub impacted_workflows: Vec<String>,
    pub impact_reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CriticalWorkflowDesignation {
    pub workflow_id: String,
    pub stronger_scheduling_guarantees: bool,
    pub stronger_audit_guarantees: bool,
    pub stronger_verification_guarantees: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductPositioningNote {
    pub statement: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InnovationRoadmap {
    pub stable_commitments: Vec<String>,
    pub research_directions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorldClassPlatformScorecard {
    pub determinism: u8,
    pub artifacts: u8,
    pub replay: u8,
    pub observability: u8,
    pub federation: u8,
    pub trust: u8,
    pub operability: u8,
}

pub fn workflow_template_catalog() -> Vec<WorkflowTemplate> {
    vec![
        WorkflowTemplate {
            template_id: "etl-standard".to_string(),
            kind: WorkflowTemplateKind::Etl,
            required_nodes: vec!["extract".to_string(), "transform".to_string(), "load".to_string()],
        },
        WorkflowTemplate {
            template_id: "ml-train-eval".to_string(),
            kind: WorkflowTemplateKind::MlTraining,
            required_nodes: vec![
                "prepare-data".to_string(),
                "train-model".to_string(),
                "evaluate-model".to_string(),
            ],
        },
    ]
}

pub fn approval_gate_ready(gate: &ApprovalGateNode) -> bool {
    !gate.policy_ref.is_empty() && gate.timeout_seconds > 0
}

pub fn wait_state_resumable(state: &HumanWaitState) -> bool {
    !state.resume_token.is_empty() && !state.deterministic_state_snapshot.is_empty()
}

pub fn workflow_quality_gate_passed(
    gate: &WorkflowQualityGate,
    dataset_checks_ok: bool,
    artifact_verification_ok: bool,
    approval_ok: bool,
) -> bool {
    (!gate.requires_dataset_checks || dataset_checks_ok)
        && (!gate.requires_artifact_verification || artifact_verification_ok)
        && (!gate.requires_approval || approval_ok)
}

pub fn critical_workflow_ready(designation: &CriticalWorkflowDesignation) -> bool {
    designation.stronger_scheduling_guarantees
        && designation.stronger_audit_guarantees
        && designation.stronger_verification_guarantees
}

pub fn world_class_score(scorecard: &WorldClassPlatformScorecard) -> f64 {
    (scorecard.determinism as f64
        + scorecard.artifacts as f64
        + scorecard.replay as f64
        + scorecard.observability as f64
        + scorecard.federation as f64
        + scorecard.trust as f64
        + scorecard.operability as f64)
        / 7.0
}

pub fn product_positioning_note() -> ProductPositioningNote {
    ProductPositioningNote {
        statement:
            "bijux-dag is a governed workflow operating system with deterministic execution guarantees."
                .to_string(),
    }
}

pub fn portfolio_observability(
    portfolio_id: &str,
    workflow_health: &BTreeMap<String, bool>,
) -> PortfolioObservabilitySummary {
    let healthy = workflow_health.values().filter(|status| **status).count();
    let unhealthy = workflow_health.len().saturating_sub(healthy);
    PortfolioObservabilitySummary {
        portfolio_id: portfolio_id.to_string(),
        healthy_workflows: healthy,
        unhealthy_workflows: unhealthy,
    }
}

pub fn rollout_is_progressive(rollout: &RolloutWorkflow) -> bool {
    !rollout.progressive_steps.is_empty() && !rollout.target_environments.is_empty()
}

pub fn workflow_blueprint_valid(blueprint: &PolicyComposedBlueprint) -> bool {
    !blueprint.required_policy_bundle.is_empty() && !blueprint.guarded_workflow_template.is_empty()
}

pub fn innovation_roadmap_valid(roadmap: &InnovationRoadmap) -> bool {
    !roadmap.stable_commitments.is_empty() && !roadmap.research_directions.is_empty()
}
