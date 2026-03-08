use crate::{NodeStatus, RunMetrics, RuntimeConfig, SchedulerMetrics};
use bijux_dag_artifacts::NodeCounts;
use serde_json::Value;

pub fn build_run_metrics(
    node_counts: &NodeCounts,
    graph_node_count: usize,
    options: &RuntimeConfig,
    finished_unix_ms: u128,
    started_unix_ms: u128,
    cache_hits: u64,
    output_count: usize,
) -> RunMetrics {
    let total_nodes = graph_node_count.max(1) as f64;
    RunMetrics {
        makespan_ms: finished_unix_ms.saturating_sub(started_unix_ms),
        success_ratio: node_counts.success as f64 / total_nodes,
        parallelism_utilization: (node_counts.success + node_counts.cached) as f64
            / (options.jobs.max(1) as f64 * total_nodes).max(1.0),
        cache_reuse_ratio: cache_hits as f64 / total_nodes,
        artifact_volume_bytes: output_count as u64,
        planning_ms: 0,
        scheduling_wait_ms: 0,
        execution_ms: finished_unix_ms.saturating_sub(started_unix_ms),
        trace_write_ms: 0,
        manifest_finalize_ms: 0,
        replay_compare_ms: 0,
    }
}

pub fn build_scheduler_metrics(
    node_counts: &NodeCounts,
    run_log_index: &[Value],
    options: &RuntimeConfig,
    failure_propagation_records: &[Value],
) -> SchedulerMetrics {
    SchedulerMetrics {
        queue_depth: 0,
        ready_count: 0,
        running_count: 0,
        completed_count: (node_counts.success
            + node_counts.failed
            + node_counts.skipped
            + node_counts.cached) as usize,
        retry_count: count_runtime_events(run_log_index, "node_attempt_started"),
        cache_hit_count: count_runtime_events(run_log_index, "cache_hit"),
        cache_miss_count: count_runtime_events(run_log_index, "cache_miss"),
        failure_count: count_runtime_events(run_log_index, "node_failed"),
        starvation_count: failure_propagation_records
            .iter()
            .filter(|v| v.get("cause").and_then(|x| x.as_str()) == Some("budget"))
            .count() as u64,
        dispatch_latency_ms: 0,
        concurrency_pressure: (options.jobs.max(1) as f64)
            / (options.scheduler_policy.max_parallelism.max(1) as f64),
    }
}

fn count_runtime_events(run_log_index: &[Value], name: &str) -> u64 {
    run_log_index
        .iter()
        .filter(|row| row.get("event").and_then(|v| v.as_str()) == Some(name))
        .count() as u64
}

pub fn count_cache_hits(status_map: &std::collections::HashMap<String, NodeStatus>) -> u64 {
    status_map
        .values()
        .filter(|status| matches!(status, NodeStatus::Cached))
        .count() as u64
}
