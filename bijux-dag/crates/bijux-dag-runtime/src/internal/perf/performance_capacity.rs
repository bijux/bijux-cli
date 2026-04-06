use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SyntheticDagProfile {
    pub name: String,
    pub depth: usize,
    pub fan_out: usize,
    pub branch_factor: usize,
    pub partitions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BenchmarkResult {
    pub name: String,
    pub throughput_per_sec: f64,
    pub p95_latency_ms: f64,
    pub memory_mb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SchedulerScalabilityResult {
    pub queue_throughput_per_sec: f64,
    pub fairness_score: f64,
    pub admission_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ArtifactStoreBenchmarkResult {
    pub read_mbps: f64,
    pub write_mbps: f64,
    pub dedup_ratio: f64,
    pub gc_minutes: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CapacityModel {
    pub scheduler_capacity: usize,
    pub worker_capacity: usize,
    pub artifact_iops: usize,
    pub registry_qps: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AutoscalingHint {
    pub target_component: String,
    pub current_replicas: usize,
    pub recommended_replicas: usize,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageGrowthForecast {
    pub daily_gb: f64,
    pub monthly_gb: f64,
    pub annual_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StorageCostModel {
    pub local_store_monthly_cost: f64,
    pub object_store_monthly_cost: f64,
    pub hot_cache_monthly_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PerformanceGate {
    pub family: String,
    pub max_latency_regression_pct: u32,
    pub min_throughput_retention_pct: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentScaleProfile {
    pub environment: String,
    pub max_active_runs: usize,
    pub max_parallel_nodes: usize,
    pub expected_artifact_gb_per_day: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PerformanceMaturityReport {
    pub throughput_trend: f64,
    pub latency_trend: f64,
    pub utilization_trend: f64,
    pub cost_trend: f64,
}

pub fn synthetic_large_dag_profiles() -> Vec<SyntheticDagProfile> {
    vec![
        SyntheticDagProfile {
            name: "deep-chain".to_string(),
            depth: 1000,
            fan_out: 1,
            branch_factor: 1,
            partitions: 1,
        },
        SyntheticDagProfile {
            name: "wide-fanout".to_string(),
            depth: 20,
            fan_out: 200,
            branch_factor: 3,
            partitions: 8,
        },
        SyntheticDagProfile {
            name: "mixed-partition-branch".to_string(),
            depth: 120,
            fan_out: 20,
            branch_factor: 5,
            partitions: 64,
        },
    ]
}

pub fn derive_autoscaling_hint(
    queue_depth: usize,
    dispatch_lag_seconds: u32,
    saturation_pct: u32,
    current_replicas: usize,
) -> AutoscalingHint {
    let mut recommended = current_replicas;
    if queue_depth > 1000 || dispatch_lag_seconds > 30 || saturation_pct > 80 {
        recommended += 2;
    }
    AutoscalingHint {
        target_component: "scheduler-workers".to_string(),
        current_replicas,
        recommended_replicas: recommended,
        reason: format!(
            "queue_depth={queue_depth}, dispatch_lag_seconds={dispatch_lag_seconds}, saturation_pct={saturation_pct}"
        ),
    }
}

pub fn forecast_storage_growth(daily_gb: f64) -> StorageGrowthForecast {
    StorageGrowthForecast {
        daily_gb,
        monthly_gb: daily_gb * 30.0,
        annual_gb: daily_gb * 365.0,
    }
}

pub fn build_cost_model(
    local_gb: f64,
    object_gb: f64,
    cache_gb: f64,
    local_cost_per_gb: f64,
    object_cost_per_gb: f64,
    cache_cost_per_gb: f64,
) -> StorageCostModel {
    StorageCostModel {
        local_store_monthly_cost: local_gb * local_cost_per_gb,
        object_store_monthly_cost: object_gb * object_cost_per_gb,
        hot_cache_monthly_cost: cache_gb * cache_cost_per_gb,
    }
}

pub fn detect_performance_regression(
    baseline: &BenchmarkResult,
    candidate: &BenchmarkResult,
    gate: &PerformanceGate,
) -> Vec<String> {
    let mut violations = Vec::new();
    let latency_regression =
        ((candidate.p95_latency_ms - baseline.p95_latency_ms) / baseline.p95_latency_ms) * 100.0;
    let throughput_retention = (candidate.throughput_per_sec / baseline.throughput_per_sec) * 100.0;

    if latency_regression > gate.max_latency_regression_pct as f64 {
        violations.push(format!(
            "latency regression {:.2}% exceeds {}%",
            latency_regression, gate.max_latency_regression_pct
        ));
    }
    if throughput_retention < gate.min_throughput_retention_pct as f64 {
        violations.push(format!(
            "throughput retention {:.2}% below {}%",
            throughput_retention, gate.min_throughput_retention_pct
        ));
    }
    violations
}

pub fn compile_environment_profiles() -> BTreeMap<String, EnvironmentScaleProfile> {
    BTreeMap::from([
        (
            "dev".to_string(),
            EnvironmentScaleProfile {
                environment: "dev".to_string(),
                max_active_runs: 20,
                max_parallel_nodes: 100,
                expected_artifact_gb_per_day: 5,
            },
        ),
        (
            "ci".to_string(),
            EnvironmentScaleProfile {
                environment: "ci".to_string(),
                max_active_runs: 100,
                max_parallel_nodes: 500,
                expected_artifact_gb_per_day: 30,
            },
        ),
        (
            "staging".to_string(),
            EnvironmentScaleProfile {
                environment: "staging".to_string(),
                max_active_runs: 500,
                max_parallel_nodes: 2_000,
                expected_artifact_gb_per_day: 120,
            },
        ),
        (
            "prod".to_string(),
            EnvironmentScaleProfile {
                environment: "prod".to_string(),
                max_active_runs: 5_000,
                max_parallel_nodes: 20_000,
                expected_artifact_gb_per_day: 2_000,
            },
        ),
    ])
}

pub fn build_performance_maturity_report(
    throughput_trend: f64,
    latency_trend: f64,
    utilization_trend: f64,
    cost_trend: f64,
) -> PerformanceMaturityReport {
    PerformanceMaturityReport {
        throughput_trend,
        latency_trend,
        utilization_trend,
        cost_trend,
    }
}
