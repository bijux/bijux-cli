use criterion as _;
use hex as _;
use serde as _;
use serde_json::{self, json};
use serde_yaml as _;
use sha2::{Digest, Sha256};
use tempfile as _;
use thiserror as _;
use unicode_normalization as _;

use bijux_dag_core::{lower_graph_to_execution_plan, parse_graph_strict, PlanOptions};
use std::time::Instant;

#[test]
fn planner_stress_handles_thousands_of_nodes() {
    let graph = build_chain_graph(2_500);
    let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");

    assert_eq!(plan.nodes.len(), 2_500);
    assert_eq!(plan.ordering.len(), 2_500);
    assert_eq!(plan.edges.len(), 2_499);
}

#[test]
fn planner_reproducibility_hash_is_stable_across_runs() {
    let graph = build_fan_graph(400);

    let mut digests = Vec::new();
    for _ in 0..5 {
        let plan = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");
        let payload = serde_json::to_vec(&plan).expect("serialize plan");
        let mut h = Sha256::new();
        h.update(payload);
        digests.push(format!("{:x}", h.finalize()));
    }

    assert!(
        digests.windows(2).all(|w| w[0] == w[1]),
        "planner output must stay byte-stable across repeated runs"
    );
}

#[test]
fn planner_runtime_budget_large_graph_regression_guard() {
    let graph = build_chain_graph(2_000);
    let started = Instant::now();

    let _ = lower_graph_to_execution_plan(&graph, PlanOptions::default()).expect("plan");

    let elapsed = started.elapsed();
    assert!(
        elapsed.as_millis() < 8_000,
        "planner runtime budget exceeded for 2k-node graph: {:?}",
        elapsed
    );
}

fn build_chain_graph(nodes: usize) -> bijux_dag_core::Graph {
    let mut raw_nodes = Vec::with_capacity(nodes);
    let mut edges = Vec::with_capacity(nodes.saturating_sub(1));

    for i in 0..nodes {
        raw_nodes.push(json!({
            "id": format!("n{i}"),
            "kind": "const",
            "inputs": if i == 0 { vec![] } else { vec!["in"] },
            "outputs": [{"name":"out","path":format!("n{i}/out.txt")}],
            "effects": ["filesystem"],
            "params": {"value": i as i64}
        }));

        if i > 0 {
            edges.push(json!({
                "from": {"node_id": format!("n{}", i - 1), "port": "out"},
                "to": {"node_id": format!("n{i}"), "port": "in"}
            }));
        }
    }

    let raw = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"planner-chain","owners":[],"tags":[]},
      "nodes": raw_nodes,
      "edges": edges
    })
    .to_string();

    parse_graph_strict(&raw).expect("parse graph")
}

fn build_fan_graph(leaves: usize) -> bijux_dag_core::Graph {
    let mut raw_nodes = vec![json!({
        "id":"root",
        "kind":"const",
        "inputs":[],
        "outputs":[{"name":"out","path":"root/out.txt"}],
        "effects":["filesystem"],
        "params":{"value":1}
    })];
    let mut edges = Vec::with_capacity(leaves);

    for i in 0..leaves {
        raw_nodes.push(json!({
            "id": format!("leaf_{i}"),
            "kind":"const",
            "inputs":["in"],
            "outputs":[{"name":"out","path":format!("leaf_{i}/out.txt")}],
            "effects":["filesystem"],
            "params":{"value":i as i64}
        }));
        edges.push(json!({
            "from": {"node_id":"root","port":"out"},
            "to": {"node_id": format!("leaf_{i}"), "port":"in"}
        }));
    }

    let raw = json!({
      "spec":"bijux-dag/v0.1",
      "meta":{"name":"planner-fan","owners":[],"tags":[]},
      "nodes": raw_nodes,
      "edges": edges
    })
    .to_string();

    parse_graph_strict(&raw).expect("parse graph")
}
