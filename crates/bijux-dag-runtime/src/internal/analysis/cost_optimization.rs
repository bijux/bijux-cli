use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExecutionCostModel {
    pub node_execution_cost: f64,
    pub artifact_storage_cost: f64,
    pub artifact_transfer_cost: f64,
    pub backend_usage_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackendPricingModel {
    pub backend_name: String,
    pub cpu_unit_cost: f64,
    pub memory_gb_cost: f64,
    pub gpu_unit_cost: f64,
    pub network_class_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactEgressEstimate {
    pub from_store: String,
    pub to_store: String,
    pub from_region: String,
    pub to_region: String,
    pub bytes: u64,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostAttributionRecord {
    pub tenant: String,
    pub environment: String,
    pub dag_name: String,
    pub run_id: String,
    pub dataset_refs: Vec<String>,
    pub total_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CostAwareRoutingPolicy {
    pub trust_constraints: BTreeSet<String>,
    pub locality_constraints: BTreeSet<String>,
    pub latency_constraints: BTreeSet<String>,
    pub prefer_lower_cost: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CacheReuseCostScore {
    pub recompute_cost: f64,
    pub reuse_cost: f64,
    pub prefer_reuse: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlanCostEstimate {
    pub plan_id: String,
    pub estimated_total_cost: f64,
    pub component_costs: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunBudget {
    pub hard_ceiling: Option<f64>,
    pub soft_ceiling: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TenantBudgetPolicy {
    pub tenant: String,
    pub budget_limit: f64,
    pub throttle_threshold: f64,
    pub reroute_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostAnomaly {
    pub category: String,
    pub expected_cost: f64,
    pub observed_cost: f64,
    pub deviation_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostForecast {
    pub subject: String,
    pub estimated_cost: f64,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CostPlacementExplanation {
    pub run_id: String,
    pub selected_backend: String,
    pub explanation: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostBackfillThrottle {
    pub urgency_score: f64,
    pub cost_pressure_score: f64,
    pub effective_parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CostPerformanceProfile {
    CheapestSafe,
    Balanced,
    FastestSafe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CostObservabilityReport {
    pub by_dag: BTreeMap<String, f64>,
    pub by_tenant: BTreeMap<String, f64>,
    pub by_queue: BTreeMap<String, f64>,
    pub by_backend: BTreeMap<String, f64>,
    pub by_region: BTreeMap<String, f64>,
    pub by_artifact_class: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CostSimulationScenario {
    pub name: String,
    pub cross_region_replay: bool,
    pub hot_cache_miss: bool,
    pub bursty_backfill: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CostSafetyPolicy {
    pub preserve_determinism: bool,
    pub preserve_trust_constraints: bool,
    pub preserve_compliance_constraints: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformCostMaturityScorecard {
    pub unit_economics_ready: bool,
    pub attribution_quality_ready: bool,
    pub optimization_safety_ready: bool,
}

pub fn choose_cost_profile(latency_sensitive: bool, strict_budget: bool) -> CostPerformanceProfile {
    if latency_sensitive && !strict_budget {
        CostPerformanceProfile::FastestSafe
    } else if strict_budget {
        CostPerformanceProfile::CheapestSafe
    } else {
        CostPerformanceProfile::Balanced
    }
}

pub fn cache_reuse_score(recompute_cost: f64, reuse_cost: f64) -> CacheReuseCostScore {
    CacheReuseCostScore { recompute_cost, reuse_cost, prefer_reuse: reuse_cost <= recompute_cost }
}

pub fn run_budget_allows(cost: f64, budget: &RunBudget) -> bool {
    if let Some(hard) = budget.hard_ceiling {
        if cost > hard {
            return false;
        }
    }
    true
}

pub fn budget_policy_action(cost: f64, policy: &TenantBudgetPolicy) -> &'static str {
    if cost >= policy.reroute_threshold {
        "reroute"
    } else if cost >= policy.throttle_threshold {
        "throttle"
    } else {
        "allow"
    }
}

pub fn detect_cost_anomaly(
    expected: f64,
    observed: f64,
    threshold_ratio: f64,
) -> Option<CostAnomaly> {
    if expected <= 0.0 {
        return None;
    }
    let deviation_ratio = observed / expected;
    if deviation_ratio >= threshold_ratio {
        Some(CostAnomaly {
            category: "spike".to_string(),
            expected_cost: expected,
            observed_cost: observed,
            deviation_ratio,
        })
    } else {
        None
    }
}

pub fn cost_optimization_allowed(policy: &CostSafetyPolicy) -> bool {
    policy.preserve_determinism
        && policy.preserve_trust_constraints
        && policy.preserve_compliance_constraints
}

pub fn scorecard_ready(scorecard: &PlatformCostMaturityScorecard) -> bool {
    scorecard.unit_economics_ready
        && scorecard.attribution_quality_ready
        && scorecard.optimization_safety_ready
}
