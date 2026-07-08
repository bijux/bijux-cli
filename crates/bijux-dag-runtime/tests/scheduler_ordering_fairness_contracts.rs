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

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{
    build_plan, build_scheduler, deterministic_schedule_order, fairness_is_satisfied,
    weighted_priority_tie_break_order, DependencyCounter, PriorityClass, ReadyNode, ReadyQueue,
    RuntimeConfig, ScheduledSubmission, SchedulerPolicy, WeightedPriorityPolicy,
};
use std::collections::BTreeMap;
use std::time::Instant;

#[test]
fn scheduler_ordering_is_deterministic_for_equal_priority_ready_nodes() {
    let ready = vec![ready("b", 10, 0, 100), ready("a", 10, 0, 100), ready("c", 10, 0, 100)];

    let first = deterministic_schedule_order(ready.clone(), &BTreeMap::new());
    let second = deterministic_schedule_order(ready, &BTreeMap::new());
    assert_eq!(first, second);
    assert_eq!(ids(&first), vec!["a", "b", "c"]);
}

#[test]
fn scheduler_fairness_promotes_starved_nodes() {
    let ready = vec![ready("critical", 100, 0, 100), ready("starved", 1, 0, 100)];
    let starvation = BTreeMap::from([("starved".to_string(), 20)]);

    let ordered = deterministic_schedule_order(ready, &starvation);
    assert_eq!(ordered[0].node_id, "starved");
    assert!(fairness_is_satisfied(&ordered, 10, &starvation));
}

#[test]
fn scheduler_concurrency_limit_is_enforced_in_batch_decision() {
    let graph = independent_graph(3);
    let mut cfg = RuntimeConfig::default();
    cfg.scheduler_policy = SchedulerPolicy { max_parallelism: 2, ..SchedulerPolicy::default() };

    let plan = build_plan(&graph, &cfg);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&cfg.scheduler_policy);

    let decision = scheduler.next_batch(&graph, &mut ready, &cfg, Instant::now(), false);
    assert!(decision.batch.len() <= 2);
}

#[test]
fn scheduler_resource_budget_blocks_excess_cpu_work() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"a/out"}],"resources":{"cpu":2,"mem_mb":64}},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"b/out"}],"resources":{"cpu":2,"mem_mb":64}}
          ],
          "edges":[]
        }"#,
    )
    .expect("graph");

    let mut cfg = RuntimeConfig::default();
    cfg.scheduler_policy =
        SchedulerPolicy { max_parallelism: 2, cpu_budget: Some(2), ..SchedulerPolicy::default() };

    let plan = build_plan(&graph, &cfg);
    let dep_counter = DependencyCounter::from_plan(&plan);
    let mut ready = ReadyQueue::from_indegree(dep_counter.indegree_map());
    let mut scheduler = build_scheduler(&cfg.scheduler_policy);
    let decision = scheduler.next_batch(&graph, &mut ready, &cfg, Instant::now(), false);

    assert_eq!(decision.batch.len(), 1);
    assert_eq!(decision.blocked_by_budget.len(), 1);
}

#[test]
fn scheduler_priority_ordering_uses_weighted_policy() {
    let submissions =
        vec![sub("standard-1", "r1", 3), sub("critical-1", "r2", 4), sub("high-1", "r3", 2)];
    let priorities = BTreeMap::from([
        ("standard-1".to_string(), PriorityClass::Standard),
        ("critical-1".to_string(), PriorityClass::Critical),
        ("high-1".to_string(), PriorityClass::High),
    ]);
    let policy = WeightedPriorityPolicy {
        critical_weight: 100,
        high_weight: 75,
        standard_weight: 50,
        low_weight: 25,
    };

    let ordered = weighted_priority_tie_break_order(submissions, &priorities, &policy);
    assert_eq!(ordered[0].schedule_id, "critical-1");
    assert_eq!(ordered[1].schedule_id, "high-1");
}

#[test]
fn scheduler_starvation_prevention_prefers_oldest_starved_first() {
    let ready = vec![ready("n1", 5, 0, 100), ready("n2", 5, 0, 100)];
    let starvation = BTreeMap::from([("n1".to_string(), 2), ("n2".to_string(), 8)]);

    let ordered = deterministic_schedule_order(ready, &starvation);
    assert_eq!(ordered[0].node_id, "n2");
}

#[test]
fn scheduler_tie_break_rules_are_stable_for_identical_priority() {
    let ready = vec![ready("b", 10, 1, 200), ready("a", 10, 1, 200), ready("c", 10, 1, 100)];
    let ordered = deterministic_schedule_order(ready, &BTreeMap::new());

    assert_eq!(ids(&ordered), vec!["c", "a", "b"]);
}

#[test]
fn scheduler_mixed_duration_signals_are_captured_in_sla_metrics() {
    let metrics = bijux_dag_runtime::evaluate_sla_metrics(
        &[(120, 100), (100, 100), (140, 120)],
        &[(400, 350), (200, 250)],
        2,
        1,
    );
    assert_eq!(metrics.missed_expected_start, 2);
    assert_eq!(metrics.missed_expected_finish, 1);
}

#[test]
fn scheduler_handles_large_ready_sets() {
    let mut ready_nodes = Vec::new();
    for i in 0..5_000u32 {
        ready_nodes.push(ready(&format!("n{i}"), 5, 0, i as u128));
    }

    let ordered = deterministic_schedule_order(ready_nodes, &BTreeMap::new());
    assert_eq!(ordered.len(), 5_000);
    assert_eq!(ordered[0].node_id, "n0");
}

#[test]
fn scheduler_fuzz_ordering_is_stable_under_randomized_inputs() {
    let mut seed = 0x51a7_9eed_u64;
    for _ in 0..200 {
        let n = 3 + (lcg(&mut seed) % 32) as usize;
        let mut ready_nodes = Vec::new();
        for i in 0..n {
            ready_nodes.push(ready(
                &format!("n{i}"),
                (lcg(&mut seed) % 10) as u8,
                (lcg(&mut seed) % 3) as u32,
                lcg(&mut seed) as u128 % 10_000,
            ));
        }

        let first = deterministic_schedule_order(ready_nodes.clone(), &BTreeMap::new());
        shuffle(&mut ready_nodes, &mut seed);
        let second = deterministic_schedule_order(ready_nodes, &BTreeMap::new());
        assert_eq!(ids(&first), ids(&second));
    }
}

fn independent_graph(nodes: usize) -> bijux_dag_core::Graph {
    let raw_nodes = (0..nodes)
        .map(|i| {
            serde_json::json!({
                "id": format!("n{i}"),
                "kind": "const",
                "inputs": [],
                "outputs": [{"name":"out","path":format!("n{i}/out.txt")}],
                "effects": ["filesystem"],
                "params": {"value": i as i64}
            })
        })
        .collect::<Vec<_>>();
    let raw = serde_json::json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"independent","owners":[],"tags":[]},
      "nodes": raw_nodes,
      "edges": []
    })
    .to_string();
    parse_graph_strict(&raw).expect("parse graph")
}

fn ready(node_id: &str, priority: u8, attempt: u32, ready_unix_ms: u128) -> ReadyNode {
    ReadyNode { node_id: node_id.to_string(), priority, attempt, ready_unix_ms }
}

fn sub(schedule_id: &str, run_id: &str, created_unix_ms: u128) -> ScheduledSubmission {
    ScheduledSubmission {
        schedule_id: schedule_id.to_string(),
        run_id: run_id.to_string(),
        created_unix_ms,
        status: bijux_dag_runtime::ScheduleSubmissionStatus::Pending,
    }
}

fn ids(nodes: &[ReadyNode]) -> Vec<&str> {
    nodes.iter().map(|n| n.node_id.as_str()).collect()
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
    *seed
}

fn shuffle<T>(items: &mut [T], seed: &mut u64) {
    if items.len() < 2 {
        return;
    }
    for i in (1..items.len()).rev() {
        let j = (lcg(seed) % ((i + 1) as u64)) as usize;
        items.swap(i, j);
    }
}
