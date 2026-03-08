//! DAG resolve entrypoints.

use crate::canonical::sort_value_maps;
use crate::{Graph, GraphError, ParamValue, RefSpec, ResolvedGraph};
use serde_json::Value;
use std::collections::BTreeMap;

impl Graph {
    pub fn resolve_graph(&self) -> Result<ResolvedGraph, GraphError> {
        let mut resolved_params = BTreeMap::new();
        for node in &self.nodes {
            let mut value = resolve_param_value(&node.params, self)?;
            sort_value_maps(&mut value);
            resolved_params.insert(node.id.clone(), value);
        }

        Ok(ResolvedGraph {
            graph: self.clone(),
            resolved_params,
        })
    }
}

pub fn resolve_graph(graph: &Graph) -> Result<ResolvedGraph, GraphError> {
    graph.resolve_graph()
}

pub(crate) fn resolve_param_value(value: &ParamValue, graph: &Graph) -> Result<Value, GraphError> {
    match value {
        ParamValue::Literal(value) => Ok(value.clone()),
        ParamValue::Array(items) => {
            let mut out = Vec::with_capacity(items.len());
            for value in items {
                out.push(resolve_param_value(value, graph)?);
            }
            Ok(Value::Array(out))
        }
        ParamValue::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map {
                out.insert(key.clone(), resolve_param_value(value, graph)?);
            }
            Ok(Value::Object(out))
        }
        ParamValue::Ref(reference) => resolve_ref(reference, graph),
    }
}

fn resolve_ref(reference: &RefSpec, graph: &Graph) -> Result<Value, GraphError> {
    if let Some(input_name) = &reference.graph_input {
        if let Some(value) = graph.inputs.get(input_name) {
            return Ok(value.clone());
        }
        return Err(GraphError::ValidationFailed);
    }

    if let Some(node_output) = &reference.node_output {
        if let Some(node) = graph.nodes.iter().find(|node| node.id == node_output.node_id) {
            if let Some(output) = node.outputs.iter().find(|output| output.name == node_output.path) {
                return Ok(Value::String(output.path.clone()));
            }
        }
        return Err(GraphError::ValidationFailed);
    }

    Err(GraphError::ValidationFailed)
}
