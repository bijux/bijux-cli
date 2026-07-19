use bijux_dag_core::{
    validate_graph, FileOutput, Graph, Node, NodeKind, ParamValue, SemanticNodeKind, TriggerRule,
    SPEC_VERSION,
};
use serde_json::json;

fn reduce_node(outputs: Vec<FileOutput>, params: ParamValue, trigger_rule: TriggerRule) -> Node {
    Node {
        id: "reduce".to_string(),
        kind: NodeKind::Shell,
        semantic_kind: SemanticNodeKind::Reduce,
        inputs: vec!["mapped".to_string()],
        outputs,
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
        trigger_rule,
        branch: None,
        dynamic: None,
    }
}

fn reduce_params(items: Vec<(&str, ParamValue)>) -> ParamValue {
    ParamValue::Object(items.into_iter().map(|(key, value)| (key.to_string(), value)).collect())
}

fn reduce_runtime_params(entries: Vec<(&str, ParamValue)>) -> ParamValue {
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
        .chain([("reduce".to_string(), reduce_params(entries))])
        .collect(),
    )
}

#[test]
fn semantic_reduce_requires_single_output_and_supported_contract_values() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![reduce_node(
            vec![
                FileOutput::new("out", "reduce.txt"),
                FileOutput::new("extra", "reduce.extra.txt"),
            ],
            reduce_runtime_params(vec![
                ("mode", ParamValue::Literal(json!("best_effort"))),
                ("empty", ParamValue::Literal(json!("maybe"))),
            ]),
            TriggerRule::AnySuccess,
        )],
        edges: vec![],
    };

    let diagnostics = validate_graph(&graph);
    assert!(diagnostics.iter().any(|diag| diag.code == "E1059"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E1060"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E1061"));
    assert!(diagnostics.iter().any(|diag| diag.code == "E1062"));
}

#[test]
fn semantic_reduce_rejects_legacy_allow_empty_collection_flag() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![reduce_node(
            vec![FileOutput::new("out", "reduce.txt")],
            reduce_runtime_params(vec![(
                "allow_empty_collection",
                ParamValue::Literal(json!(true)),
            )]),
            TriggerRule::AllSuccess,
        )],
        edges: vec![],
    };

    let diagnostics = validate_graph(&graph);
    assert!(diagnostics.iter().any(|diag| diag.code == "E1061"));
}

#[test]
fn semantic_reduce_accepts_partial_mode_and_explicit_empty_policy() {
    let graph = Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: Default::default(),
        nondeterminism_allowed: false,
        subgraphs: Default::default(),
        subgraph_instances: Vec::new(),
        nodes: vec![reduce_node(
            vec![FileOutput::new("out", "reduce.txt")],
            reduce_runtime_params(vec![
                ("mode", ParamValue::Literal(json!("partial"))),
                ("empty", ParamValue::Literal(json!("allow"))),
            ]),
            TriggerRule::AllSuccess,
        )],
        edges: vec![],
    };

    let diagnostics = validate_graph(&graph);
    assert!(!diagnostics
        .iter()
        .any(|diag| { matches!(diag.code.as_str(), "E1059" | "E1060" | "E1061" | "E1062") }));
}
