use bijux_dag_core::{parse_graph_strict, Graph};
use serde_json::json;

mod support;

use support::DagFixture;

fn lcg_next(state: &mut u64) -> u64 {
    *state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
    *state
}

fn synth_graph(state: &mut u64, nodes: usize) -> Graph {
    let mut fixture = DagFixture::new();
    for idx in 0..nodes {
        fixture = fixture.const_node(&format!("n{idx:02}"), json!({"value": idx}));
    }
    for idx in 1..nodes {
        let parent = (lcg_next(state) as usize) % idx;
        fixture = fixture.edge(&format!("n{parent:02}"), "out", &format!("n{idx:02}"), "in");
        if idx > 1 && (lcg_next(state) & 1) == 0 {
            let extra_parent = (lcg_next(state) as usize) % idx;
            if extra_parent != parent {
                fixture = fixture.edge(
                    &format!("n{extra_parent:02}"),
                    "out",
                    &format!("n{idx:02}"),
                    "in",
                );
            }
        }
    }
    fixture.build()
}

fn equivalent_graph_with_reordered_arrays(graph: &Graph) -> Graph {
    let mut value = serde_json::to_value(graph).expect("graph value");
    let nodes = value["nodes"].as_array_mut().expect("nodes array");
    nodes.reverse();
    let edges = value["edges"].as_array_mut().expect("edges array");
    edges.reverse();
    serde_json::from_value(value).expect("graph decode")
}

#[test]
fn canonicalization_fuzz_preserves_identity_for_equivalent_graphs() {
    let mut state = 0xC0FFEE_u64;
    for nodes in [2usize, 3, 5, 8, 13] {
        for _ in 0..64 {
            let base = synth_graph(&mut state, nodes);
            let equivalent = equivalent_graph_with_reordered_arrays(&base);
            assert_eq!(
                base.canonical_json_bytes().expect("canonical base"),
                equivalent.canonical_json_bytes().expect("canonical equivalent"),
            );
            assert_eq!(base.graph_id().expect("base id"), equivalent.graph_id().expect("equiv id"));
        }
    }
}

#[test]
fn canonicalization_fuzz_reparse_roundtrip_stays_stable() {
    let mut state = 0xBAD5EED_u64;
    for _ in 0..128 {
        let graph = synth_graph(&mut state, 6);
        let canonical = graph.canonical_json_bytes().expect("canonical bytes");
        let reparsed =
            parse_graph_strict(&String::from_utf8(canonical.clone()).expect("utf8 canonical"))
                .expect("reparse canonical");
        assert_eq!(canonical, reparsed.canonical_json_bytes().expect("recanon"));
        assert_eq!(graph.graph_id().expect("graph id"), reparsed.graph_id().expect("reparsed id"));
    }
}
