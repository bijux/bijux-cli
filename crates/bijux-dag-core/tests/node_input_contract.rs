use bijux_dag_core::{
    node_input_bindings, node_io_contract, parse_graph_strict, NodeInputSource, OutputKind,
    ParamBindingSource,
};

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

#[test]
fn node_io_contract_exposes_param_env_and_output_bindings() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "inputs":{"threads":8},
          "nodes":[
            {"id":"seed","kind":"const","inputs":[],"outputs":[{"name":"out","path":"seed/out"}],"params":{"value":1}},
            {
              "id":"run",
              "kind":"shell",
              "inputs":["reads"],
              "outputs":[{"name":"bam","path":"align/out.bam","kind":"binary","required":false,"media_type":"application/bam"}],
              "params":{
                "argv":["aligner","--threads",{"graph_input":"threads"}],
                "seed":{"node_output":{"node_id":"seed","path":"out"}}
              },
              "effects":["filesystem","env"],
              "env_allowlist":["REFGENOME"]
            }
          ],
          "edges":[
            {"from":{"node_id":"seed","port":"out"},"to":{"node_id":"run","port":"reads"}}
          ]
        }"#,
    )
    .expect("parse graph");

    let contract = node_io_contract(&graph, "run").expect("io contract");
    assert_eq!(contract.inputs.len(), 1);
    assert_eq!(contract.env_bindings[0].name, "REFGENOME");
    assert!(!contract.outputs[0].required);
    assert_eq!(contract.outputs[0].kind, OutputKind::Binary);
    assert_eq!(contract.outputs[0].media_type, "application/bam");
    assert!(contract.param_bindings.iter().any(|binding| {
        matches!(
            binding.source,
            ParamBindingSource::GraphInput { ref input_name } if input_name == "threads"
        )
    }));
    assert!(contract.param_bindings.iter().any(|binding| {
        matches!(
            binding.source,
            ParamBindingSource::NodeOutput { ref node_id, ref output_name }
                if node_id == "seed" && output_name == "out"
        )
    }));
}

#[test]
fn node_io_contract_marks_wildcard_env_patterns_as_optional_matches() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"run",
              "kind":"shell",
              "inputs":[],
              "outputs":[{"name":"out","path":"run/out"}],
              "params":{"argv":["/bin/sh","-c","env > ../outputs/run/out"]},
              "effects":["filesystem","env"],
              "env_allowlist":["EXACT_ENV","PREFIX_*"]
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph");

    let contract = node_io_contract(&graph, "run").expect("io contract");
    assert_eq!(contract.env_bindings.len(), 2);
    assert_eq!(contract.env_bindings[0].name, "EXACT_ENV");
    assert!(contract.env_bindings[0].required);
    assert_eq!(contract.env_bindings[1].name, "PREFIX_*");
    assert!(!contract.env_bindings[1].required);
}

#[test]
fn graph_resources_preserve_named_resource_requests() {
    let graph = parse_graph_strict(
        r#"{
          "spec":"bijux-dag/v0.1",
          "nodes":[
            {
              "id":"licensed",
              "kind":"shell",
              "inputs":[],
              "outputs":[{"name":"out","path":"licensed/out"}],
              "params":{"argv":["echo","licensed"]},
              "resources":{
                "cpu":1,
                "mem_mb":256,
                "named_resources":{
                  "database_slot":2,
                  "license.render":1
                }
              }
            }
          ],
          "edges":[]
        }"#,
    )
    .expect("parse graph");

    let resources = graph.nodes[0].resources.as_ref().expect("resources");
    assert_eq!(resources.named_resources.get("database_slot"), Some(&2));
    assert_eq!(resources.named_resources.get("license.render"), Some(&1));
}
