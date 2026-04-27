use bijux_dag_core::{node_input_bindings, parse_graph_strict, NodeInputSource};

#[test]
fn node_input_bindings_resolve_upstream_outputs_by_port() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}},
            {"id":"b","kind":"shell","inputs":["reads","index"],"outputs":[{"name":"out","path":"b/out"}],"params":{"argv":["echo","b"]}}
          ],
          "edges":[
            {"from":{"node_id":"a","port":"out"},"to":{"node_id":"b","port":"reads"}}
          ]
        }"#,
    )
    .expect("parse graph");

    let bindings = node_input_bindings(&graph, "b");
    assert_eq!(bindings.len(), 2);
    assert_eq!(
        bindings[0].source,
        NodeInputSource::UpstreamOutput {
            node_id: "a".to_string(),
            output_name: "out".to_string(),
        }
    );
    assert_eq!(bindings[1].source, NodeInputSource::Unbound);
}

#[test]
fn node_input_bindings_for_unknown_node_are_empty() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[{"id":"a","kind":"const","inputs":[],"outputs":[{"name":"out","path":"a/out"}],"params":{"value":1}}],
          "edges":[]
        }"#,
    )
    .expect("parse graph");
    assert!(node_input_bindings(&graph, "missing").is_empty());
}
