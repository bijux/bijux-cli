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
    adaptive_cache_policy, adaptive_fallback_needed, adaptive_maturity_ready,
    adaptive_queue_throttle, choose_prefetch_hints, compare_static_and_adaptive,
    decide_adaptive_parallelism, detect_adaptive_drift, render_adaptive_explanation,
    AdaptiveBoundsPolicy, AdaptiveControlLoopGuard, AdaptiveMaturityGate,
};
use serde::Deserialize;
use std::collections::BTreeMap;

#[derive(Debug, Deserialize)]
struct FixtureMetrics {
    queue_pressure: f64,
    saturation: f64,
    current_parallelism: u32,
    baseline_score: f64,
    adaptive_score: f64,
}

fn load_metrics() -> FixtureMetrics {
    let raw = std::fs::read_to_string("tests/fixtures/adaptive/metrics.json")
        .expect("adaptive metrics fixture");
    serde_json::from_str(&raw).expect("valid adaptive metrics fixture")
}

#[test]
fn adaptive_concurrency_respects_bounds() {
    let m = load_metrics();
    let bounds =
        AdaptiveBoundsPolicy { min_parallelism: 2, max_parallelism: 12, max_priority_boost: 4 };

    let decision =
        decide_adaptive_parallelism(m.queue_pressure, m.saturation, m.current_parallelism, &bounds);
    assert!(decision.next_parallelism <= 12);
    assert!(decision.next_parallelism >= 2);
}

#[test]
fn adaptive_throttle_and_cache_policies_follow_pressure_and_reuse() {
    let throttle = adaptive_queue_throttle(0.8, 0.7, 0.6);
    assert!(throttle.throttle_ratio > 0.6);

    let cache = adaptive_cache_policy(0.82);
    assert!(cache.promote_to_hot_cache);
    assert_eq!(cache.retention_minutes, 180);
}

#[test]
fn drift_detection_and_fallback_trigger_when_regression_is_large() {
    let m = load_metrics();
    let drift = detect_adaptive_drift(m.baseline_score, m.adaptive_score);
    assert!(drift.degraded);

    let guard = AdaptiveControlLoopGuard { max_parallelism_step: 2, rollback_threshold: 0.05 };
    assert!(adaptive_fallback_needed(&drift, &guard));
}

#[test]
fn prefetch_hints_require_replay_heavy_signal_and_confidence() {
    let hints = choose_prefetch_hints(
        true,
        &[("artifact-a".to_string(), 0.9), ("artifact-b".to_string(), 0.4)],
    );
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0].artifact_id, "artifact-a");
}

#[test]
fn static_vs_adaptive_comparison_and_explanations_are_machine_readable() {
    let report = compare_static_and_adaptive(300.0, 220.0, 0.08, 0.04);
    assert!(report.adaptive_dispatch_latency < report.static_dispatch_latency);

    let evidence = BTreeMap::from([
        ("queue_pressure".to_string(), "0.92".to_string()),
        ("saturation".to_string(), "0.74".to_string()),
    ]);
    let explanation = render_adaptive_explanation("parallelism-adjustment", &evidence);
    assert_eq!(explanation.decision_kind, "parallelism-adjustment");
    assert_eq!(explanation.evidence.len(), 2);
}

#[test]
fn maturity_gate_requires_tests_experiments_and_docs() {
    let gate = AdaptiveMaturityGate {
        experiments_complete: true,
        acceptance_tests_green: true,
        docs_complete: true,
    };
    assert!(adaptive_maturity_ready(&gate));
}
