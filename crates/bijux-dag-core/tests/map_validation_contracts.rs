use bijux_dag_core::{
    validate_graph, Edge, EdgeKind, FileOutput, Graph, Node, NodeKind, OutputKind, ParamValue,
    PortRef, SemanticNodeKind, SPEC_VERSION,
};
use serde_json::json;

fn map_node(inputs: Vec<&str>, output_kind: OutputKind, params: ParamValue) -> Node {
    Node {
        id: "map".to_string(),
        kind: NodeKind::Shell,
        semantic_kind: SemanticNodeKind::Map,
        inputs: inputs.into_iter().map(str::to_string).collect(),
        outputs: vec![FileOutput {
            name: "out".to_string(),
            path: "mapped".to_string(),
            kind: output_kind,
            required: true,
            media_type: None,
            promotable: false,
        }],
        params,
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: Default::default(),
        cache: Default::default(),
        effects: vec![bijux_dag_core::Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: Default::default(),
        branch: None,
        dynamic: None,
    }
}

#[test]
fn semantic_map_requires_directory_outputs_and_explicit_multi_input_binding() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![
            Node {
                id: "seed".to_string(),
                kind: NodeKind::Const,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out", "seed/out.json")],
                params: ParamValue::Literal(json!(["a", "b"])),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: Default::default(),
                cache: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
                trigger_rule: Default::default(),
                branch: None,
                dynamic: None,
            },
            map_node(
                vec!["left", "right"],
                OutputKind::File,
                ParamValue::Object(
                    [(
                        "argv".to_string(),
                        ParamValue::Array(vec![
                            ParamValue::Literal(json!("/bin/sh")),
                            ParamValue::Literal(json!("-c")),
                            ParamValue::Literal(json!("printf ok")),
                        ]),
                    )]
                    .into_iter()
                    .collect(),
                ),
            ),
        ],
        edges: vec![Edge {
            id: None,
            kind: EdgeKind::Data,
            decision: None,
            from: PortRef { node_id: "seed".to_string(), port: "out".to_string() },
            to: PortRef { node_id: "map".to_string(), port: "left".to_string() },
        }],
    };

    let diagnostics = validate_graph(&graph);
    assert!(diagnostics.iter().any(|diag| diag.code == "E1057"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E1058"));
}

#[test]
fn semantic_map_accepts_directory_outputs_with_declared_array_input_binding() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![
            Node {
                id: "seed".to_string(),
                kind: NodeKind::Const,
                semantic_kind: SemanticNodeKind::Task,
                inputs: vec![],
                outputs: vec![FileOutput::new("out", "seed/out.json")],
                params: ParamValue::Literal(json!(["a", "b"])),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: Default::default(),
                cache: Default::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
                trigger_rule: Default::default(),
                branch: None,
                dynamic: None,
            },
            Node {
                params: ParamValue::Object(
                    [
                        (
                            "argv".to_string(),
                            ParamValue::Array(vec![
                                ParamValue::Literal(json!("/bin/sh")),
                                ParamValue::Literal(json!("-c")),
                                ParamValue::Literal(json!("printf ok")),
                            ]),
                        ),
                        (
                            "map".to_string(),
                            ParamValue::Object(
                                [("input".to_string(), ParamValue::Literal(json!("left")))]
                                    .into_iter()
                                    .collect(),
                            ),
                        ),
                    ]
                    .into_iter()
                    .collect(),
                ),
                ..map_node(vec!["left", "right"], OutputKind::Directory, ParamValue::default())
            },
        ],
        edges: vec![
            Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "seed".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "map".to_string(), port: "left".to_string() },
            },
            Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: "seed".to_string(), port: "out".to_string() },
                to: PortRef { node_id: "map".to_string(), port: "right".to_string() },
            },
        ],
    };

    let diagnostics = validate_graph(&graph);
    assert!(!diagnostics
        .iter()
        .any(|diag| matches!(diag.code.as_str(), "E1056" | "E1057" | "E1058")));
}
