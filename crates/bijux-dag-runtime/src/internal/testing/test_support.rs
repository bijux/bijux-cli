use crate::Graph;
use bijux_dag_core::{
    Edge, EdgeKind, Effect, FileOutput, Node, NodeKind, ParamValue, PortRef, RetryPolicy,
    SemanticNodeKind, TriggerRule, SPEC_VERSION,
};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub(crate) fn docker_available() -> bool {
    Command::new("docker").arg("--version").output().map(|o| o.status.success()).unwrap_or(false)
}

pub(crate) fn sample_graph() -> Graph {
    let mut join = shell_node("d");
    join.inputs = vec!["in_left".to_string(), "in_right".to_string()];
    graph_from_nodes(
        vec![const_node("a"), shell_node("b"), shell_node("c"), join],
        vec![
            ("a", "out", "b", "in"),
            ("a", "out", "c", "in"),
            ("b", "out", "d", "in_left"),
            ("c", "out", "d", "in_right"),
        ],
    )
}

pub(crate) fn param_object(items: Vec<(&str, Value)>) -> ParamValue {
    let mut map = BTreeMap::new();
    for (k, v) in items {
        map.insert(k.to_string(), ParamValue::Literal(v));
    }
    ParamValue::Object(map)
}

fn graph_from_nodes(nodes: Vec<Node>, edges: Vec<(&str, &str, &str, &str)>) -> Graph {
    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: BTreeMap::new(),
        nondeterminism_allowed: false,
        subgraphs: BTreeMap::new(),
        subgraph_instances: Vec::new(),
        nodes,
        edges: edges
            .into_iter()
            .map(|(from_node, from_port, to_node, to_port)| Edge {
                id: None,
                kind: EdgeKind::Data,
                decision: None,
                from: PortRef { node_id: from_node.to_string(), port: from_port.to_string() },
                to: PortRef { node_id: to_node.to_string(), port: to_port.to_string() },
            })
            .collect(),
    }
}

fn const_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Const,
        semantic_kind: SemanticNodeKind::Task,
        inputs: vec![],
        outputs: vec![FileOutput::new("out".to_string(), format!("out_{id}"))],
        params: param_object(vec![("value", Value::from("ok"))]),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![],
        env_allowlist: vec![],
        group: None,
        trigger_rule: TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    }
}

fn shell_node(id: &str) -> Node {
    Node {
        id: id.to_string(),
        kind: NodeKind::Shell,
        semantic_kind: SemanticNodeKind::Task,
        inputs: vec!["in".to_string()],
        outputs: vec![FileOutput::new("out".to_string(), format!("out_{id}"))],
        params: param_object(vec![(
            "argv",
            Value::Array(vec![
                Value::from("/bin/sh"),
                Value::from("-c"),
                Value::from(format!("echo ok > ../outputs/out_{id}")),
            ]),
        )]),
        container: None,
        timeout_ms: None,
        resources: None,
        tags: vec![],
        retry: RetryPolicy::default(),
        cache: Default::default(),
        effects: vec![Effect::Filesystem],
        env_allowlist: vec![],
        group: None,
        trigger_rule: TriggerRule::AllSuccess,
        branch: None,
        dynamic: None,
    }
}
