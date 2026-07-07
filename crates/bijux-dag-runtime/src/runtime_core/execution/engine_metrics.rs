use crate::{NodeStatus, RunMetrics, RuntimeConfig, SchedulerMetrics};
use bijux_dag_artifacts::{FailurePropagationRecord, NodeCounts};
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
    failure_propagation_records: &[FailurePropagationRecord],
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
            .filter(|record| record.reason == "budget")
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
    status_map.values().filter(|status| matches!(status, NodeStatus::Cached)).count() as u64
}

#[cfg(test)]
mod tests {
    use super::{build_run_metrics, build_scheduler_metrics, count_cache_hits};
    use crate::{NodeStatus, RuntimeConfig};
    use bijux_dag_artifacts::{FailurePropagationRecord, NodeCounts};
    use serde_json::json;
    use std::collections::HashMap;

    #[test]
    fn run_metrics_shape_is_stable_for_finished_run() {
        let counts = NodeCounts { success: 3, failed: 1, skipped: 0, cached: 2, cancelled: 0 };
        let metrics = build_run_metrics(&counts, 6, &RuntimeConfig::default(), 1_500, 1_000, 2, 4);
        assert_eq!(metrics.makespan_ms, 500);
        assert_eq!(metrics.execution_ms, 500);
        assert!(metrics.success_ratio > 0.0);
        assert!(metrics.cache_reuse_ratio > 0.0);
    }

    #[test]
    fn scheduler_metrics_counts_events_and_budget_starvation() {
        let counts = NodeCounts { success: 2, failed: 1, skipped: 1, cached: 1, cancelled: 0 };
        let log = vec![
            json!({"event":"node_attempt_started"}),
            json!({"event":"cache_hit"}),
            json!({"event":"cache_miss"}),
            json!({"event":"node_failed"}),
        ];
        let failures = vec![
            FailurePropagationRecord {
                node_id: "blocked".to_string(),
                status: "skipped".to_string(),
                reason: "budget".to_string(),
                propagation_mode: None,
                blocking_nodes: Vec::new(),
            },
            FailurePropagationRecord {
                node_id: "other".to_string(),
                status: "failed".to_string(),
                reason: "other".to_string(),
                propagation_mode: None,
                blocking_nodes: Vec::new(),
            },
        ];
        let metrics = build_scheduler_metrics(&counts, &log, &RuntimeConfig::default(), &failures);
        assert_eq!(metrics.retry_count, 1);
        assert_eq!(metrics.cache_hit_count, 1);
        assert_eq!(metrics.cache_miss_count, 1);
        assert_eq!(metrics.failure_count, 1);
        assert_eq!(metrics.starvation_count, 1);
    }

    #[test]
    fn cache_hit_counter_tracks_only_cached_nodes() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), NodeStatus::Cached);
        map.insert("b".to_string(), NodeStatus::Success);
        map.insert("c".to_string(), NodeStatus::Cached);
        assert_eq!(count_cache_hits(&map), 2);
    }
}
