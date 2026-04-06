use crate::{Effect, FileOutput, Node, ParamValue};
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
