use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::simulated_platform::{
    approval_gate_ready, critical_workflow_ready, evolution_plan_valid, portfolio_observability,
    product_positioning_note, rollout_is_progressive, wait_state_resumable,
    workflow_blueprint_valid, workflow_quality_gate_passed, workflow_template_catalog,
    world_class_score, ApprovalGateNode, CriticalWorkflowDesignation, EvolutionPlan,
    HumanWaitState, PolicyComposedBlueprint, RolloutWorkflow, WorkflowQualityGate,
    WorldClassPlatformScorecard,
};
use std::collections::BTreeMap;

fn load_health() -> BTreeMap<String, bool> {
    let raw = std::fs::read_to_string("tests/fixtures/workflow_product/workflow_health.json")
        .expect("workflow health fixture");
    serde_json::from_str(&raw).expect("valid workflow health fixture")
}

#[test]
fn workflow_templates_and_positioning_note_are_present() {
    let catalog = workflow_template_catalog();
    assert!(!catalog.is_empty());

    let note = product_positioning_note();
    assert!(note.statement.contains("workflow operating system"));
}

#[test]
fn approval_wait_and_quality_gates_enforce_contract_requirements() {
    let approval = ApprovalGateNode {
        node_id: "approval-release".to_string(),
        policy_ref: "policy-release-gate".to_string(),
        timeout_seconds: 3600,
    };
    assert!(approval_gate_ready(&approval));

    let wait = HumanWaitState {
        wait_id: "wait-human-review".to_string(),
        resume_token: "resume-123".to_string(),
        deterministic_state_snapshot: "snapshot-abc".to_string(),
    };
    assert!(wait_state_resumable(&wait));

    let gate = WorkflowQualityGate {
        gate_id: "quality-release".to_string(),
        requires_dataset_checks: true,
        requires_artifact_verification: true,
        requires_approval: true,
    };
    assert!(workflow_quality_gate_passed(&gate, true, true, true));
    assert!(!workflow_quality_gate_passed(&gate, true, false, true));
}

#[test]
fn critical_designation_rollout_and_blueprint_validations_work() {
    let critical = CriticalWorkflowDesignation {
        workflow_id: "workflow-risk-report".to_string(),
        stronger_scheduling_guarantees: true,
        stronger_audit_guarantees: true,
        stronger_verification_guarantees: true,
    };
    assert!(critical_workflow_ready(&critical));

    let rollout = RolloutWorkflow {
        rollout_id: "rollout-risk".to_string(),
        source_environment: "staging".to_string(),
        target_environments: vec!["prod-eu".to_string(), "prod-us".to_string()],
        progressive_steps: vec![
            "canary-10".to_string(),
            "canary-50".to_string(),
            "full".to_string(),
        ],
    };
    assert!(rollout_is_progressive(&rollout));

    let blueprint = PolicyComposedBlueprint {
        blueprint_id: "regulated-release".to_string(),
        required_policy_bundle: "policy-regulated-v4".to_string(),
        guarded_workflow_template: "validation-promote".to_string(),
    };
    assert!(workflow_blueprint_valid(&blueprint));
}

#[test]
fn portfolio_observability_and_world_class_score_are_computable() {
    let health = load_health();
    let summary = portfolio_observability("portfolio-analytics", &health);
    assert_eq!(summary.healthy_workflows, 2);
    assert_eq!(summary.unhealthy_workflows, 1);

    let scorecard = WorldClassPlatformScorecard {
        determinism: 95,
        artifacts: 92,
        replay: 90,
        observability: 88,
        federation: 86,
        trust: 94,
        operability: 89,
    };
    assert_eq!(world_class_score(&scorecard), 90.57142857142857);
}

#[test]
fn evolution_plan_requires_stable_and_research_tracks() {
    let plan = EvolutionPlan {
        stable_commitments: vec!["deterministic-core".to_string()],
        research_directions: vec!["autonomous-optimization-safety".to_string()],
    };
    assert!(evolution_plan_valid(&plan));
}
