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

use bijux_dag_runtime::invariant_catalog_default;
use bijux_dag_runtime::simulated_platform::{
    evaluate_slo, health_dashboard_score, integrated_verification_lane_default,
    release_policy_allows, PlatformHealthDashboard, ReleaseGovernancePolicy,
    ServiceLevelIndicators, ServiceLevelObjective,
};

fn load_objective() -> ServiceLevelObjective {
    let raw = std::fs::read_to_string("tests/fixtures/operations/slo_objective.json")
        .expect("objective fixture");
    serde_json::from_str(&raw).expect("valid objective")
}

fn load_indicators() -> ServiceLevelIndicators {
    let raw = std::fs::read_to_string("tests/fixtures/operations/slo_indicators.json")
        .expect("indicators fixture");
    serde_json::from_str(&raw).expect("valid indicators")
}

#[test]
fn evaluates_slo_success_when_indicators_meet_objective() {
    let objective = load_objective();
    let indicators = load_indicators();
    let evaluation = evaluate_slo(&objective, &indicators);
    assert!(evaluation.passed);
    assert!(evaluation.violations.is_empty());
}

#[test]
fn release_policy_requires_all_release_controls() {
    let policy = ReleaseGovernancePolicy {
        requires_evidence_bundle: true,
        requires_compatibility_results: true,
        requires_rollback_plan: true,
    };
    assert!(release_policy_allows(&policy));

    let blocked = ReleaseGovernancePolicy {
        requires_evidence_bundle: true,
        requires_compatibility_results: false,
        requires_rollback_plan: true,
    };
    assert!(!release_policy_allows(&blocked));
}

#[test]
fn computes_platform_health_score() {
    let dashboard = PlatformHealthDashboard {
        engine_health: 0.98,
        scheduler_health: 0.95,
        artifact_store_health: 0.99,
        auth_health: 0.97,
        policy_health: 0.96,
    };
    let score = health_dashboard_score(&dashboard);
    assert!(score > 0.96 && score < 0.98);
}

#[test]
fn includes_non_negotiable_invariants() {
    let catalog = invariant_catalog_default();
    assert!(!catalog.is_empty());
}

#[test]
fn integrated_lane_covers_required_domains() {
    let lane = integrated_verification_lane_default();
    assert!(lane.required_domains.iter().any(|item| item == "multi-tenant"));
    assert!(lane.required_domains.iter().any(|item| item == "compatibility-governance"));
    assert!(lane.required_evidence.iter().any(|item| item == "attestation-verification-report"));
}
