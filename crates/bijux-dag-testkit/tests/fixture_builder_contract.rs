use bijux_dag_testkit::{graph_branch_join_fixture, graph_map_reduce_fixture, DagFixture};
use serde_json::json;

#[test]
fn dag_fixture_builder_constructs_shell_and_const_workflows() {
    let graph = DagFixture::new()
        .const_node("seed", json!({"value": 1}))
        .shell_node(
            "transform",
            &["in"],
            &["/bin/sh", "-c", "printf ok > ../outputs/transform.txt"],
            "transform.txt",
        )
        .edge("seed", "out", "transform", "in")
        .build();

    assert_eq!(graph.nodes.len(), 2);
    assert_eq!(graph.edges.len(), 1);
    assert_eq!(graph.nodes[1].id, "transform");
    assert_eq!(graph.nodes[1].outputs[0].path, "transform.txt");
}

#[test]
fn dag_fixture_builder_emits_map_reduce_and_branch_shapes() {
    let map_reduce = graph_map_reduce_fixture();
    assert_eq!(map_reduce.nodes.len(), 5);
    assert_eq!(map_reduce.edges.len(), 6);
    assert!(map_reduce.nodes.iter().any(|node| node.id == "reduce"));

    let branch = graph_branch_join_fixture();
    assert!(branch.nodes.iter().any(|node| node.id == "decide" && node.branch.is_some()));
    assert!(branch.edges.iter().any(|edge| edge.to.node_id == "join"));
}
