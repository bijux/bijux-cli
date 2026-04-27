use crate::{Effect, FileOutput, Graph, Node, ParamValue};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInterfaceContract {
    pub declared_inputs: Vec<String>,
    pub declared_outputs: Vec<FileOutput>,
    pub declared_params: Vec<String>,
    pub declared_effects: Vec<Effect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeGroupContract {
    pub id: String,
    pub labels: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedNode {
    pub node: Node,
    pub interface: NodeInterfaceContract,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeInputSource {
    UpstreamOutput { node_id: String, output_name: String },
    Unbound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeInputBinding {
    pub name: String,
    pub source: NodeInputSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTypeRegistry {
    pub known_types: Vec<String>,
}

impl NodeTypeRegistry {
    pub fn default_registry() -> Self {
        Self {
            known_types: vec!["const".to_string(), "shell".to_string(), "container".to_string()],
        }
    }

    pub fn validate_node_kinds(&self, nodes: &[Node]) -> Result<(), String> {
        for node in nodes {
            let kind = node.kind.as_str();
            if self.known_types.iter().any(|known| known == kind) {
                continue;
            }
            if kind.is_empty() {
                return Err("node kind must not be empty".to_string());
            }
        }
        Ok(())
    }
}

pub fn derive_interface(node: &Node) -> NodeInterfaceContract {
    let declared_params = match &node.params {
        ParamValue::Object(map) => map.keys().cloned().collect(),
        _ => Vec::new(),
    };
    NodeInterfaceContract {
        declared_inputs: node.inputs.clone(),
        declared_outputs: node.outputs.clone(),
        declared_params,
        declared_effects: node.effects.clone(),
    }
}

pub fn node_input_bindings(graph: &Graph, node_id: &str) -> Vec<NodeInputBinding> {
    let Some(node) = graph.nodes.iter().find(|node| node.id == node_id) else {
        return Vec::new();
    };
    let mut bindings = node
        .inputs
        .iter()
        .map(|name| NodeInputBinding { name: name.clone(), source: NodeInputSource::Unbound })
        .collect::<Vec<_>>();

    for binding in &mut bindings {
        if let Some(edge) = graph
            .edges
            .iter()
            .find(|edge| edge.to.node_id == node_id && edge.to.port == binding.name)
        {
            binding.source = NodeInputSource::UpstreamOutput {
                node_id: edge.from.node_id.clone(),
                output_name: edge.from.port.clone(),
            };
        }
    }

    bindings
}
