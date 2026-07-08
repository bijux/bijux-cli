use bijux_dag_core::{parse_graph_strict, EdgeKind, Effect};

mod support;

use support::branch_semantics_graph_json;

fn base_graph() -> bijux_dag_core::Graph {
    parse_graph_strict(branch_semantics_graph_json()).expect("parse branch contract graph")
}

#[test]
fn semantic_mutations_change_graph_identity() {
    let base = base_graph();
    let base_id = base.graph_id().expect("base graph id");

    let mut changed_params = base.clone();
    changed_params.nodes[1].params =
        serde_json::from_value(serde_json::json!({"argv":["echo","right"]})).expect("params");
    assert_ne!(base_id, changed_params.graph_id().expect("changed params id"));

    let mut changed_output = base.clone();
    changed_output.nodes[0].outputs[0].path = "seed/alternate.txt".to_string();
    assert_ne!(base_id, changed_output.graph_id().expect("changed output id"));

    let mut changed_effects = base.clone();
    changed_effects.nodes[1].effects.push(Effect::Network);
    assert_ne!(base_id, changed_effects.graph_id().expect("changed effects id"));

    let mut changed_branch = base.clone();
    changed_branch.nodes[1].branch.as_mut().expect("branch spec").default_decision =
        Some("right".to_string());
    assert_ne!(base_id, changed_branch.graph_id().expect("changed branch id"));

    let mut changed_edge_kind = base.clone();
    changed_edge_kind.edges[1].kind = EdgeKind::Data;
    assert_ne!(base_id, changed_edge_kind.graph_id().expect("changed edge kind id"));

    let mut changed_edge_decision = base.clone();
    changed_edge_decision.edges[1].decision = Some("alternate".to_string());
    assert_ne!(base_id, changed_edge_decision.graph_id().expect("changed decision id"));
}

#[test]
fn non_semantic_reordering_does_not_change_graph_identity() {
    let mut reordered = base_graph();
    reordered.nodes.reverse();
    reordered.edges.reverse();

    let base = base_graph();
    assert_eq!(base.graph_id().expect("base id"), reordered.graph_id().expect("reordered id"));
    assert_eq!(
        base.canonical_json_bytes().expect("base canonical"),
        reordered.canonical_json_bytes().expect("reordered canonical"),
    );
}
