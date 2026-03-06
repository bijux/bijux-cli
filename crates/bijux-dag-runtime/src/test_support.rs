#![cfg(test)]

use crate::{Graph, Node, NodeKind};
use bijux_dag_core::{Edge, Effect, FileOutput, ParamValue, PortRef, SPEC_VERSION};
use serde_json::Value;
use std::collections::BTreeMap;
use std::process::Command;

pub(crate) fn param_object(items: Vec<(&str, Value)>) -> ParamValue {
    let mut map = BTreeMap::new();
    for (k, v) in items {
        map.insert(k.to_string(), ParamValue::Literal(v));
    }
    ParamValue::Object(map)
}

pub(crate) fn docker_available() -> bool {
    Command::new("docker")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub(crate) fn sample_graph() -> Graph {
    Graph {
        spec: SPEC_VERSION.to_string(),
        meta: None,
        inputs: serde_json::Map::new(),
        nondeterminism_allowed: false,
        nodes: vec![
            Node {
                id: "a".to_string(),
                kind: NodeKind::Const,
                inputs: vec![],
                outputs: vec![FileOutput {
                    name: "out_a".to_string(),
                    path: "out_a".to_string(),
                }],
                params: param_object(vec![("value", Value::from(1))]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![],
                env_allowlist: vec![],
                group: None,
            },
            Node {
                id: "b".to_string(),
                kind: NodeKind::Shell,
                inputs: vec!["in".to_string()],
                outputs: vec![FileOutput {
                    name: "out_b".to_string(),
                    path: "out_b".to_string(),
                }],
                params: param_object(vec![(
                    "argv",
                    Value::Array(vec![
                        Value::from("/bin/sh"),
                        Value::from("-c"),
                        Value::from("echo ok > ../outputs/out_b"),
                    ]),
                )]),
                container: None,
                timeout_ms: None,
                resources: None,
                tags: vec![],
                retry: bijux_dag_core::RetryPolicy::default(),
                effects: vec![Effect::Filesystem],
                env_allowlist: vec![],
                group: None,
            },
        ],
        edges: vec![Edge {
            from: PortRef {
                node_id: "a".to_string(),
                port: "out_a".to_string(),
            },
            to: PortRef {
                node_id: "b".to_string(),
                port: "in".to_string(),
            },
        }],
    }
}
