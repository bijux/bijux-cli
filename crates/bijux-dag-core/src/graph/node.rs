use crate::{Effect, Graph, Node, OutputKind, OutputSpec, ParamValue, RefSpec};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInterfaceContract {
    pub declared_inputs: Vec<String>,
    pub declared_outputs: Vec<OutputSpec>,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParamBindingSource {
    GraphInput { input_name: String },
    NodeOutput { node_id: String, output_name: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeParamBinding {
    pub key_path: String,
    pub source: ParamBindingSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeEnvBinding {
    pub name: String,
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeOutputContract {
    pub name: String,
    pub path: String,
    pub kind: OutputKind,
    pub required: bool,
    pub media_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NodeIoContract {
    pub inputs: Vec<NodeInputBinding>,
    pub param_bindings: Vec<NodeParamBinding>,
    pub env_bindings: Vec<NodeEnvBinding>,
    pub outputs: Vec<NodeOutputContract>,
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

pub fn node_io_contract(graph: &Graph, node_id: &str) -> Option<NodeIoContract> {
    let node = graph.nodes.iter().find(|node| node.id == node_id)?;
    Some(NodeIoContract {
        inputs: node_input_bindings(graph, node_id),
        param_bindings: collect_param_bindings(&node.params),
        env_bindings: node
            .env_allowlist
            .iter()
            .map(|name| NodeEnvBinding { name: name.clone(), required: true })
            .collect(),
        outputs: node
            .outputs
            .iter()
            .map(|output| NodeOutputContract {
                name: output.name.clone(),
                path: output.path.clone(),
                kind: output.kind.clone(),
                required: output.required,
                media_type: output.effective_media_type(),
            })
            .collect(),
    })
}

fn collect_param_bindings(value: &ParamValue) -> Vec<NodeParamBinding> {
    let mut out = Vec::new();
    collect_param_bindings_inner(value, "$", &mut out);
    out
}

fn collect_param_bindings_inner(
    value: &ParamValue,
    key_path: &str,
    out: &mut Vec<NodeParamBinding>,
) {
    match value {
        ParamValue::Ref(reference) => {
            if let Some(binding) = param_binding_from_ref(key_path, reference) {
                out.push(binding);
            }
        }
        ParamValue::Array(items) => {
            for (index, item) in items.iter().enumerate() {
                collect_param_bindings_inner(item, &format!("{key_path}[{index}]"), out);
            }
        }
        ParamValue::Object(map) => {
            for (key, item) in map {
                collect_param_bindings_inner(item, &format!("{key_path}.{key}"), out);
            }
        }
        ParamValue::Literal(_) => {}
    }
}

fn param_binding_from_ref(key_path: &str, reference: &RefSpec) -> Option<NodeParamBinding> {
    if let Some(input_name) = &reference.graph_input {
        return Some(NodeParamBinding {
            key_path: key_path.to_string(),
            source: ParamBindingSource::GraphInput { input_name: input_name.clone() },
        });
    }
    reference.node_output.as_ref().map(|node_output| NodeParamBinding {
        key_path: key_path.to_string(),
        source: ParamBindingSource::NodeOutput {
            node_id: node_output.node_id.clone(),
            output_name: node_output.output_name.clone(),
        },
    })
}
