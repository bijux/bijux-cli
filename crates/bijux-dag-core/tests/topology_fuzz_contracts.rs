use bijux_dag_core::{Edge, EdgeKind, Graph, NodeKind, PortRef};
use serde_json::json;
use std::collections::BTreeMap;

mod support;

use support::DagFixture;

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(2862933555777941757).wrapping_add(3037000493);
    *state
}

fn dag_with_forward_edges(state: &mut u64, nodes: usize) -> Graph {
    let mut fixture = DagFixture::new();
    for idx in 0..nodes {
        fixture = fixture.const_node(&format!("n{idx:02}"), json!(idx));
    }
    for idx in 1..nodes {
        let edge_count = ((lcg_next(state) % 3) + 1) as usize;
        for _ in 0..edge_count {
            let parent = (lcg_next(state) as usize) % idx;
            fixture = fixture.edge(&format!("n{parent:02}"), "out", &format!("n{idx:02}"), "in");
        }
    }
    fixture.build()
}

fn positions(order: &[String]) -> BTreeMap<String, usize> {
    order
        .iter()
        .enumerate()
        .map(|(idx, node_id)| (node_id.clone(), idx))
        .collect::<BTreeMap<_, _>>()
}

#[test]
fn topology_fuzz_accepts_valid_dags_and_keeps_dependencies_before_dependents() {
    let mut state = 0xFACEB00C_u64;
    for nodes in [2usize, 4, 7, 11, 19] {
        for _ in 0..64 {
            let graph = dag_with_forward_edges(&mut state, nodes);
            let order = graph.topo_order().expect("valid dag");
            assert_eq!(order.len(), graph.nodes.len());
            let index = positions(&order);
            for edge in &graph.edges {
                assert!(
                    index[&edge.from.node_id] < index[&edge.to.node_id],
                    "dependency order violated for {} -> {}",
                    edge.from.node_id,
                    edge.to.node_id
                );
            }
        }
    }
}

#[test]
fn topology_fuzz_rejects_cycles_and_self_edges() {
    let mut state = 0x1234ABCD_u64;
    for _ in 0..128 {
        let mut graph = dag_with_forward_edges(&mut state, 6);
        graph.edges.push(Edge {
            id: Some("cycle-edge".to_string()),
            kind: EdgeKind::Data,
            decision: None,
            from: PortRef { node_id: "n05".to_string(), port: "out".to_string() },
            to: PortRef { node_id: "n00".to_string(), port: "in".to_string() },
        });
        assert!(graph.topo_order().is_err(), "cycle must be rejected");
    }

    let graph = DagFixture::new().const_node("solo", json!(1)).build();
    let mut self_edge = graph.clone();
    self_edge.edges.push(Edge {
        id: Some("self-edge".to_string()),
        kind: EdgeKind::Data,
        decision: None,
        from: PortRef { node_id: "solo".to_string(), port: "out".to_string() },
        to: PortRef { node_id: "solo".to_string(), port: "in".to_string() },
    });
    assert!(self_edge.topo_order().is_err(), "self-edge must be rejected");
    assert_eq!(graph.nodes[0].kind, NodeKind::Const);
}
