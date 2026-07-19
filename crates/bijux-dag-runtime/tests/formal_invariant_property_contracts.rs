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

use bijux_dag_core::{parse_graph_strict, Severity};
use bijux_dag_runtime::{build_plan, RuntimeConfig};
use serde_json::json;

fn chain_graph(node_count: usize) -> String {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for i in 0..node_count {
        nodes.push(json!({
            "id": format!("n{i}"),
            "kind": "const",
            "inputs": if i == 0 { Vec::<String>::new() } else { vec!["in".to_string()] },
            "outputs": [{"name":"out","path":format!("n{i}/out")}],
            "params": {"value": format!("{i}")}
        }));
        if i > 0 {
            edges.push(json!({
                "from":{"node_id": format!("n{}", i - 1), "port":"out"},
                "to":{"node_id": format!("n{i}"), "port":"in"}
            }));
        }
    }
    json!({"spec":"v0.1","nodes":nodes,"edges":edges}).to_string()
}

#[test]
fn generated_chain_graphs_preserve_acyclic_unique_and_deterministic_plan() {
    for size in 1..8 {
        let payload = chain_graph(size);
        let graph = parse_graph_strict(&payload).expect("parse");
        let diagnostics = graph.validate_with_warnings();
        assert!(
            diagnostics.iter().all(|d| d.severity != Severity::Error),
            "graph size {size} has validation errors"
        );

        let options = RuntimeConfig::default();
        let plan_a = build_plan(&graph, &options);
        let plan_b = build_plan(&graph, &options);
        let a = serde_json::to_value(plan_a).expect("serialize a");
        let b = serde_json::to_value(plan_b).expect("serialize b");
        assert_eq!(a, b, "plan should be deterministic for generated graph size {size}");
    }
}

#[test]
fn canonical_order_is_stable_for_diamond_and_fanout_shapes() {
    let diamond = json!({
        "spec":"v0.1",
        "nodes":[
            {"id":"a","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"b","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"c","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"d","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}}
        ],
        "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"in"}},
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"c","port":"in"}},
            {"from":{"node_id":"b","port":"out"},"to":{"node_id":"d","port":"in"}},
            {"from":{"node_id":"c","port":"out"},"to":{"node_id":"d","port":"in"}}
        ]
    })
    .to_string();

    let fanout = json!({
        "spec":"v0.1",
        "nodes":[
            {"id":"root","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"x","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}},
            {"id":"y","kind":"const","outputs":[{"name":"out","path":"out"}],"params":{"value":"1"}}
        ],
        "edges":[
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"x","port":"in"}},
            {"from":{"node_id":"root","port":"out"},"to":{"node_id":"y","port":"in"}}
        ]
    })
    .to_string();

    for payload in [diamond, fanout] {
        let graph = parse_graph_strict(&payload).expect("parse");
        let order_a = graph.canonicalize().nodes.into_iter().map(|n| n.id).collect::<Vec<_>>();
        let order_b = graph.canonicalize().nodes.into_iter().map(|n| n.id).collect::<Vec<_>>();
        assert_eq!(order_a, order_b, "canonical order must be stable");
    }
}
