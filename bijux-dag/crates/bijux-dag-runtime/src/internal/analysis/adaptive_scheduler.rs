use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveConcurrencyDecision {
    pub queue_pressure: f64,
    pub saturation: f64,
    pub previous_parallelism: u32,
    pub next_parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LearnedDurationProfile {
    pub node_class: String,
    pub sample_count: u32,
    pub p50_seconds: f64,
    pub p95_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveQueueThrottleDecision {
    pub retry_storm_index: f64,
    pub backend_churn_index: f64,
    pub store_pressure_index: f64,
    pub throttle_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveCachePolicyDecision {
    pub reuse_rate: f64,
    pub retention_minutes: u32,
    pub promote_to_hot_cache: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BackendSuitabilitySignal {
    pub backend: String,
    pub node_class: String,
    pub success_rate: f64,
    pub median_duration_seconds: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlaDispatchTuningDecision {
    pub urgency_score: f64,
    pub congestion_score: f64,
    pub priority_boost: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveBackfillPacingDecision {
    pub completion_rate_per_minute: f64,
    pub headroom_score: f64,
    pub next_batch_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactPrefetchHint {
    pub artifact_id: String,
    pub confidence: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveControlLoopGuard {
    pub max_parallelism_step: u32,
    pub rollback_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptiveExplanation {
    pub decision_kind: String,
    pub evidence: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveBoundsPolicy {
    pub min_parallelism: u32,
    pub max_parallelism: u32,
    pub max_priority_boost: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LearningWindowPolicy {
    pub lookback_days: u32,
    pub max_samples_per_node_class: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveDriftReport {
    pub baseline_score: f64,
    pub adaptive_score: f64,
    pub degraded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveComparisonReport {
    pub static_dispatch_latency: f64,
    pub adaptive_dispatch_latency: f64,
    pub static_sla_miss_rate: f64,
    pub adaptive_sla_miss_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptiveFallbackPolicy {
    pub enabled: bool,
    pub fallback_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AdaptiveQualityMetrics {
    pub stability_score: f64,
    pub predictability_score: f64,
    pub sla_benefit_score: f64,
    pub cost_impact_score: f64,
    pub fairness_impact_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdaptiveMaturityGate {
    pub experiments_complete: bool,
    pub acceptance_tests_green: bool,
    pub docs_complete: bool,
}

pub fn decide_adaptive_parallelism(
    queue_pressure: f64,
    saturation: f64,
    current_parallelism: u32,
    bounds: &AdaptiveBoundsPolicy,
) -> AdaptiveConcurrencyDecision {
    let mut next = current_parallelism as i32;
    if queue_pressure > 0.8 && saturation < 0.85 {
        next += 2;
    } else if saturation > 0.9 {
        next -= 2;
    }
    let next = next.clamp(bounds.min_parallelism as i32, bounds.max_parallelism as i32) as u32;
    AdaptiveConcurrencyDecision {
        queue_pressure,
        saturation,
        previous_parallelism: current_parallelism,
        next_parallelism: next,
    }
}

pub fn adaptive_queue_throttle(
    retry_storm_index: f64,
    backend_churn_index: f64,
    store_pressure_index: f64,
) -> AdaptiveQueueThrottleDecision {
    let throttle_ratio =
        ((retry_storm_index + backend_churn_index + store_pressure_index) / 3.0).clamp(0.0, 1.0);
    AdaptiveQueueThrottleDecision {
        retry_storm_index,
        backend_churn_index,
        store_pressure_index,
        throttle_ratio,
    }
}

pub fn adaptive_cache_policy(reuse_rate: f64) -> AdaptiveCachePolicyDecision {
    let promote = reuse_rate >= 0.6;
    let retention = if reuse_rate >= 0.8 {
        180
    } else if reuse_rate >= 0.6 {
        120
    } else {
        60
    };
    AdaptiveCachePolicyDecision {
        reuse_rate,
        retention_minutes: retention,
        promote_to_hot_cache: promote,
    }
}

pub fn detect_adaptive_drift(baseline_score: f64, adaptive_score: f64) -> AdaptiveDriftReport {
    AdaptiveDriftReport {
        baseline_score,
        adaptive_score,
        degraded: adaptive_score < baseline_score,
    }
}

pub fn adaptive_fallback_needed(
    drift: &AdaptiveDriftReport,
    guard: &AdaptiveControlLoopGuard,
) -> bool {
    drift.degraded && (drift.baseline_score - drift.adaptive_score) > guard.rollback_threshold
}

pub fn adaptive_maturity_ready(gate: &AdaptiveMaturityGate) -> bool {
    gate.experiments_complete && gate.acceptance_tests_green && gate.docs_complete
}

pub fn choose_prefetch_hints(
    replay_heavy: bool,
    candidate_artifacts: &[(String, f64)],
) -> Vec<ArtifactPrefetchHint> {
    if !replay_heavy {
        return Vec::new();
    }
    candidate_artifacts
        .iter()
        .filter(|(_, confidence)| *confidence >= 0.6)
        .map(|(artifact_id, confidence)| ArtifactPrefetchHint {
            artifact_id: artifact_id.clone(),
            confidence: *confidence,
        })
        .collect()
}

pub fn compare_static_and_adaptive(
    static_dispatch_latency: f64,
    adaptive_dispatch_latency: f64,
    static_sla_miss_rate: f64,
    adaptive_sla_miss_rate: f64,
) -> AdaptiveComparisonReport {
    AdaptiveComparisonReport {
        static_dispatch_latency,
        adaptive_dispatch_latency,
        static_sla_miss_rate,
        adaptive_sla_miss_rate,
    }
}

pub fn render_adaptive_explanation(
    decision_kind: &str,
    evidence_map: &BTreeMap<String, String>,
) -> AdaptiveExplanation {
    let evidence = evidence_map
        .iter()
        .map(|(key, value)| format!("{key}={value}"))
        .collect();
    AdaptiveExplanation {
        decision_kind: decision_kind.to_string(),
        evidence,
    }
}
