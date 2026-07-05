//! DAG resolve entrypoints.

use crate::canonical::sort_value_maps;
use crate::expansion::expand_graph;
use crate::{
    is_known_path_variable, materialize_graph_input_value, Graph, GraphError, Node, ParamValue,
    RefSpec, ResolvedGraph,
};
use serde_json::Value;
use std::collections::BTreeMap;

impl Graph {
    pub fn resolve_graph(&self) -> Result<ResolvedGraph, GraphError> {
        let expanded = expand_graph(self).map_err(|_| GraphError::ValidationFailed)?;
        let mut resolved_params = BTreeMap::new();
        for node in &expanded.nodes {
            let mut value = resolve_param_value(&node.params, &expanded)?;
            value = resolve_command_param_templates(&expanded, node, &value)?;
            sort_value_maps(&mut value);
            resolved_params.insert(node.id.clone(), value);
        }

        Ok(ResolvedGraph { graph: expanded, resolved_params })
    }
}

pub fn resolve_graph(graph: &Graph) -> Result<ResolvedGraph, GraphError> {
    graph.resolve_graph()
}

pub fn resolve_command_argv_templates(
    graph: &Graph,
    node: &Node,
    argv: &[String],
    resolved_params: &Value,
) -> Result<Vec<String>, GraphError> {
    argv.iter()
        .map(|entry| resolve_command_template_string(graph, node, resolved_params, entry))
        .collect()
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
        if let Some(spec) = graph.inputs.get(input_name) {
            if let Some(value) = spec.effective_value() {
                return materialize_graph_input_value(
                    spec,
                    value,
                    &format!("/inputs/{input_name}"),
                )
                .map_err(|_| GraphError::ValidationFailed);
            }
        }
        return Err(GraphError::ValidationFailed);
    }

    if let Some(node_output) = &reference.node_output {
        if let Some(node) = graph.nodes.iter().find(|node| node.id == node_output.node_id) {
            if let Some(output) =
                node.outputs.iter().find(|output| output.name == node_output.output_name)
            {
                return Ok(Value::String(output.path.clone()));
            }
        }
        return Err(GraphError::ValidationFailed);
    }

    if let Some(path_var) = &reference.path_var {
        return Ok(Value::String(path_var.display_path()));
    }

    Err(GraphError::ValidationFailed)
}

fn resolve_command_param_templates(
    graph: &Graph,
    node: &Node,
    resolved_params: &Value,
) -> Result<Value, GraphError> {
    let Value::Object(fields) = resolved_params else {
        return Ok(resolved_params.clone());
    };
    let Some(argv) = fields.get("argv").and_then(Value::as_array) else {
        return Ok(resolved_params.clone());
    };

    let resolved_argv = argv
        .iter()
        .map(|entry| match entry {
            Value::String(text) => {
                resolve_command_template_string(graph, node, resolved_params, text)
                    .map(Value::String)
            }
            Value::Number(number) => Ok(Value::String(number.to_string())),
            Value::Bool(flag) => Ok(Value::String(flag.to_string())),
            other => Ok(other.clone()),
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut updated = fields.clone();
    updated.insert("argv".to_string(), Value::Array(resolved_argv));
    Ok(Value::Object(updated))
}

fn resolve_command_template_string(
    graph: &Graph,
    node: &Node,
    resolved_params: &Value,
    text: &str,
) -> Result<String, GraphError> {
    let mut rendered = String::with_capacity(text.len());
    let mut cursor = 0;

    while let Some(open_offset) = text[cursor..].find('{') {
        let open_index = cursor + open_offset;
        rendered.push_str(&text[cursor..open_index]);
        let Some(close_offset) = text[(open_index + 1)..].find('}') else {
            rendered.push_str(&text[open_index..]);
            return Ok(rendered);
        };
        let close_index = open_index + 1 + close_offset;
        let placeholder = &text[(open_index + 1)..close_index];
        if let Some(value) =
            resolve_command_template_placeholder(graph, node, resolved_params, placeholder)?
        {
            rendered.push_str(&value);
        } else {
            rendered.push_str(&text[open_index..=close_index]);
        }
        cursor = close_index + 1;
    }

    rendered.push_str(&text[cursor..]);
    Ok(rendered)
}

fn resolve_command_template_placeholder(
    graph: &Graph,
    node: &Node,
    resolved_params: &Value,
    placeholder: &str,
) -> Result<Option<String>, GraphError> {
    if is_known_path_variable(placeholder) {
        return Ok(Some(format!("{{{placeholder}}}")));
    }
    if let Some(path) = placeholder.strip_prefix("params.") {
        let value = lookup_value_path(resolved_params, path).ok_or(GraphError::ValidationFailed)?;
        return value_to_command_token(value).map(Some);
    }
    if let Some(input_name) = placeholder.strip_prefix("inputs.") {
        return stable_input_path(graph, node, input_name).map(Some);
    }
    if let Some(output_name) = placeholder.strip_prefix("outputs.") {
        return stable_output_path(node, output_name).map(Some);
    }
    Ok(None)
}

fn stable_input_path(graph: &Graph, node: &Node, input_name: &str) -> Result<String, GraphError> {
    let edge = graph
        .edges
        .iter()
        .find(|edge| edge.to.node_id == node.id && edge.to.port == input_name)
        .ok_or(GraphError::ValidationFailed)?;
    Ok(format!("{{inputs_dir}}/{}/{}", edge.from.node_id, edge.to.port))
}

fn stable_output_path(node: &Node, output_name: &str) -> Result<String, GraphError> {
    let output = node
        .outputs
        .iter()
        .find(|output| output.name == output_name)
        .ok_or(GraphError::ValidationFailed)?;
    Ok(format!("{{outputs_dir}}/{}", output.path))
}

fn lookup_value_path<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let mut current = value;
    let mut token = String::new();
    let mut chars = path.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '.' => {
                if token.is_empty() {
                    return None;
                }
                current = current.get(&token)?;
                token.clear();
            }
            '[' => {
                if !token.is_empty() {
                    current = current.get(&token)?;
                    token.clear();
                }
                let mut index_text = String::new();
                while let Some(next) = chars.peek().copied() {
                    chars.next();
                    if next == ']' {
                        break;
                    }
                    index_text.push(next);
                }
                let index = index_text.parse::<usize>().ok()?;
                current = current.get(index)?;
            }
            _ => token.push(ch),
        }
    }

    if token.is_empty() {
        Some(current)
    } else {
        current.get(&token)
    }
}

fn value_to_command_token(value: &Value) -> Result<String, GraphError> {
    match value {
        Value::String(text) => Ok(text.clone()),
        Value::Number(number) => Ok(number.to_string()),
        Value::Bool(flag) => Ok(flag.to_string()),
        _ => Err(GraphError::ValidationFailed),
    }
}
