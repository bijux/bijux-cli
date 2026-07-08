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
    budget_policy_action, cache_reuse_score, choose_cost_profile, cost_optimization_allowed,
    detect_cost_anomaly, run_budget_allows, scorecard_ready, CostPerformanceProfile,
    CostSafetyPolicy, PlatformCostMaturityScorecard, RunBudget, TenantBudgetPolicy,
};

#[test]
fn profile_selection_balances_latency_and_budget() {
    assert_eq!(choose_cost_profile(true, false), CostPerformanceProfile::FastestSafe);
    assert_eq!(choose_cost_profile(false, true), CostPerformanceProfile::CheapestSafe);
    assert_eq!(choose_cost_profile(false, false), CostPerformanceProfile::Balanced);
}

#[test]
fn cache_reuse_scoring_prefers_lower_cost_path() {
    let score = cache_reuse_score(12.0, 3.0);
    assert!(score.prefer_reuse);

    let expensive_reuse = cache_reuse_score(4.0, 7.0);
    assert!(!expensive_reuse.prefer_reuse);
}

#[test]
fn run_budget_and_tenant_policy_controls_apply() {
    let budget = RunBudget { hard_ceiling: Some(100.0), soft_ceiling: Some(80.0) };
    assert!(run_budget_allows(95.0, &budget));
    assert!(!run_budget_allows(120.0, &budget));

    let policy = TenantBudgetPolicy {
        tenant: "tenant-a".to_string(),
        budget_limit: 10_000.0,
        throttle_threshold: 300.0,
        reroute_threshold: 700.0,
    };
    assert_eq!(budget_policy_action(250.0, &policy), "allow");
    assert_eq!(budget_policy_action(350.0, &policy), "throttle");
    assert_eq!(budget_policy_action(750.0, &policy), "reroute");
}

#[test]
fn anomaly_detection_flags_spikes_only() {
    let anomaly = detect_cost_anomaly(100.0, 260.0, 2.0);
    assert!(anomaly.is_some());

    let normal = detect_cost_anomaly(100.0, 130.0, 2.0);
    assert!(normal.is_none());
}

#[test]
fn cost_optimization_never_bypasses_safety_constraints() {
    let safe = CostSafetyPolicy {
        preserve_determinism: true,
        preserve_trust_constraints: true,
        preserve_compliance_constraints: true,
    };
    assert!(cost_optimization_allowed(&safe));

    let unsafe_policy = CostSafetyPolicy { preserve_trust_constraints: false, ..safe };
    assert!(!cost_optimization_allowed(&unsafe_policy));
}

#[test]
fn cost_maturity_scorecard_requires_all_dimensions() {
    let ready = PlatformCostMaturityScorecard {
        unit_economics_ready: true,
        attribution_quality_ready: true,
        optimization_safety_ready: true,
    };
    assert!(scorecard_ready(&ready));

    let not_ready = PlatformCostMaturityScorecard { optimization_safety_ready: false, ..ready };
    assert!(!scorecard_ready(&not_ready));
}
