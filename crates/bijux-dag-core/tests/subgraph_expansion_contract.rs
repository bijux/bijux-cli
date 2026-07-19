use bijux_dag_core::{compile_graph, parse_graph_strict, NodeOutputRef, ParamValue, SPEC_VERSION};

fn parse_graph(input: &str) -> bijux_dag_core::Graph {
    parse_graph_strict(input).expect("parse graph")
}

fn reusable_alignment_graph(instances: &str, edges: &str, consumer_ref: &str) -> String {
    format!(
        r#"{{
            "spec":"{spec}",
            "inputs":{{
                "sample":{{"type":"string","default":"tumor"}},
                "replicate":{{"type":"string","default":"r1"}}
            }},
            "subgraphs":{{
                "align_block":{{
                    "graph":{{
                        "spec":"{spec}",
                        "inputs":{{
                            "sample_name":{{"type":"string"}},
                            "replicate_name":{{"type":"string","default":"r1"}}
                        }},
                        "nodes":[
                            {{
                                "id":"extract",
                                "kind":"const",
                                "outputs":[{{"name":"sheet","path":"extract/sheet.txt"}}],
                                "params":{{"sample":{{"graph_input":"sample_name"}}}}
                            }},
                            {{
                                "id":"align",
                                "kind":"const",
                                "inputs":["sheet"],
                                "outputs":[{{"name":"bam","path":"align/result.bam"}}],
                                "params":{{
                                    "sheet":{{"node_output":{{"node_id":"extract","output_name":"sheet"}}}},
                                    "replicate":{{"graph_input":"replicate_name"}}
                                }}
                            }}
                        ],
                        "edges":[
                            {{
                                "from":{{"node_id":"extract","port":"sheet"}},
                                "to":{{"node_id":"align","port":"sheet"}}
                            }}
                        ]
                    }},
                    "outputs":{{
                        "aligned":{{"node_id":"align","output_name":"bam"}}
                    }}
                }}
            }},
            "subgraph_instances":[{instances}],
            "nodes":[
                {{
                    "id":"consume",
                    "kind":"const",
                    "inputs":["bam"],
                    "outputs":[{{"name":"out","path":"consume/out.txt"}}],
                    "params":{{"bam":{consumer_ref}}}
                }}
            ],
            "edges":[{edges}]
        }}"#,
        spec = SPEC_VERSION,
        instances = instances,
        edges = edges,
        consumer_ref = consumer_ref,
    )
}

#[test]
fn compile_expands_reusable_subgraph_instances_into_plain_graphs() {
    let graph = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{
                "sample_name":{"graph_input":"sample"},
                "replicate_name":{"graph_input":"replicate"}
            }
        }"#,
        r#"{
            "from":{"node_id":"tumor_align","port":"aligned"},
            "to":{"node_id":"consume","port":"bam"}
        }"#,
        r#"{"node_output":{"node_id":"tumor_align","output_name":"aligned"}}"#,
    ));

    let compiled = compile_graph(&graph).expect("compile graph");
    let normalized = compiled.normalized_graph;

    assert!(normalized.subgraphs.is_empty());
    assert!(normalized.subgraph_instances.is_empty());
    assert!(normalized.nodes.iter().any(|node| node.id == "tumor_align__extract"));
    assert!(normalized.nodes.iter().any(|node| node.id == "tumor_align__align"));
    assert!(normalized.edges.iter().any(|edge| {
        edge.from.node_id == "tumor_align__extract"
            && edge.to.node_id == "tumor_align__align"
            && edge.to.port == "sheet"
    }));
    assert!(normalized.edges.iter().any(|edge| {
        edge.from.node_id == "tumor_align__align"
            && edge.from.port == "bam"
            && edge.to.node_id == "consume"
            && edge.to.port == "bam"
    }));

    let consume = normalized.nodes.iter().find(|node| node.id == "consume").expect("consumer node");
    assert!(matches!(
        &consume.params,
        ParamValue::Object(fields)
            if matches!(
                fields.get("bam"),
                Some(ParamValue::Ref(reference))
                    if matches!(
                        reference.node_output.as_ref(),
                        Some(NodeOutputRef { node_id, output_name })
                            if node_id == "tumor_align__align" && output_name == "bam"
                    )
            )
    ));

    let align = normalized
        .nodes
        .iter()
        .find(|node| node.id == "tumor_align__align")
        .expect("expanded align node");
    assert_eq!(align.outputs[0].path, "subgraphs/tumor_align/align/result.bam");
}

#[test]
fn reusable_subgraph_identity_is_stable_under_instance_reorder() {
    let a = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        },{
            "id":"normal_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        }"#,
        r#"{
            "from":{"node_id":"tumor_align","port":"aligned"},
            "to":{"node_id":"consume","port":"bam"}
        }"#,
        r#"{"node_output":{"node_id":"normal_align","output_name":"aligned"}}"#,
    ));
    let b = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"normal_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        },{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        }"#,
        r#"{
            "from":{"node_id":"tumor_align","port":"aligned"},
            "to":{"node_id":"consume","port":"bam"}
        }"#,
        r#"{"node_output":{"node_id":"normal_align","output_name":"aligned"}}"#,
    ));

    assert_eq!(a.graph_id().expect("graph id"), b.graph_id().expect("graph id"));
}

#[test]
fn reusable_subgraph_canonical_json_is_stable_under_instance_reorder() {
    let a = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        },{
            "id":"normal_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        }"#,
        r#"{
            "from":{"node_id":"tumor_align","port":"aligned"},
            "to":{"node_id":"consume","port":"bam"}
        }"#,
        r#"{"node_output":{"node_id":"normal_align","output_name":"aligned"}}"#,
    ));
    let b = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"normal_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        },{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        }"#,
        r#"{
            "from":{"node_id":"tumor_align","port":"aligned"},
            "to":{"node_id":"consume","port":"bam"}
        }"#,
        r#"{"node_output":{"node_id":"normal_align","output_name":"aligned"}}"#,
    ));

    assert_eq!(
        a.to_canonical_json().expect("canonical json"),
        b.to_canonical_json().expect("canonical json")
    );
}

#[test]
fn validation_rejects_edges_into_reusable_subgraph_inputs() {
    let graph = parse_graph(&reusable_alignment_graph(
        r#"{
            "id":"tumor_align",
            "subgraph":"align_block",
            "input_bindings":{"sample_name":{"graph_input":"sample"}}
        }"#,
        r#"{
            "from":{"node_id":"consume","port":"out"},
            "to":{"node_id":"tumor_align","port":"sample_name"}
        }"#,
        r#"{"node_output":{"node_id":"tumor_align","output_name":"aligned"}}"#,
    ));

    let diagnostics = graph.validate_with_warnings();
    assert!(diagnostics.iter().any(|diagnostic| diagnostic.code == "E1038"));
}
