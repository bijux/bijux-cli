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

use bijux_dag_runtime::{
    build_cost_model, build_performance_maturity_report, compile_environment_profiles,
    derive_autoscaling_hint, detect_performance_regression, forecast_storage_growth,
    synthetic_large_dag_profiles, BenchmarkResult, PerformanceGate,
};

mod support;

#[test]
fn generates_large_dag_profiles() {
    let profiles = synthetic_large_dag_profiles();
    assert_eq!(profiles.len(), 3);
    assert!(profiles.iter().any(|profile| profile.name == "wide-fanout"));
}

#[test]
fn derives_autoscaling_from_backpressure_signals() {
    let hint = derive_autoscaling_hint(2500, 45, 90, 4);
    assert_eq!(hint.current_replicas, 4);
    assert_eq!(hint.recommended_replicas, 6);
}

#[test]
fn forecasts_storage_growth_and_cost() {
    let growth = forecast_storage_growth(85.0);
    assert_eq!(growth.monthly_gb, 2550.0);

    let cost = build_cost_model(500.0, 4000.0, 250.0, 0.2, 0.03, 0.12);
    assert!(cost.object_store_monthly_cost > 0.0);
}

#[test]
fn flags_performance_regression_against_family_gate() {
    let baseline: BenchmarkResult = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/performance/benchmark_baseline.json",
    );
    let candidate: BenchmarkResult = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/performance/benchmark_candidate.json",
    );
    let gate = PerformanceGate {
        family: "planner".to_string(),
        max_latency_regression_pct: 10,
        min_throughput_retention_pct: 95,
    };

    let violations = detect_performance_regression(&baseline, &candidate, &gate);
    assert_eq!(violations.len(), 2);
}

#[test]
fn compiles_environment_scale_profiles_and_maturity_report() {
    let profiles = compile_environment_profiles();
    assert!(profiles.contains_key("dev"));
    assert!(profiles.contains_key("prod"));

    let report = build_performance_maturity_report(1.08, -0.04, 0.12, -0.03);
    assert_eq!(report.throughput_trend, 1.08);
}
