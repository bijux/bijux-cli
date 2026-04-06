//! DAG canonicalization entrypoints and helpers.

use crate::{Effect, Graph, GraphError, ParamValue, Severity, ValidationDiagnostic};
use serde_json::Value;
use std::collections::BTreeMap;
use unicode_normalization::UnicodeNormalization;

impl Graph {
    pub fn canonicalize(&self) -> Graph {
        let mut nodes = self.nodes.clone();
        let mut edges = self.edges.clone();

        for node in &mut nodes {
            node.id = normalize_identity_text(&node.id);
            node.inputs = node.inputs.iter().map(|input| normalize_identity_text(input)).collect();
            for output in &mut node.outputs {
                output.name = normalize_identity_text(&output.name);
                output.path = normalize_rel_path(&output.path);
            }
            node.env_allowlist =
                node.env_allowlist.iter().map(|entry| normalize_identity_text(entry)).collect();
            node.tags = node.tags.iter().map(|entry| normalize_identity_text(entry)).collect();
            if let Some(group) = &node.group {
                node.group = Some(normalize_identity_text(group));
            }
        }

        for edge in &mut edges {
            edge.from.node_id = normalize_identity_text(&edge.from.node_id);
            edge.from.port = normalize_identity_text(&edge.from.port);
            edge.to.node_id = normalize_identity_text(&edge.to.node_id);
            edge.to.port = normalize_identity_text(&edge.to.port);
        }

        nodes.sort_by(|left, right| left.id.cmp(&right.id));
        for node in &mut nodes {
            sort_param_value(&mut node.params);
            node.inputs.sort();
            node.outputs.sort_by(|left, right| left.name.cmp(&right.name));
            node.effects.sort_by_key(effect_order);
            node.env_allowlist.sort();
            node.tags.sort();
            if let Some(resources) = &node.resources {
                if resources.cpu == 0 && resources.mem_mb == 0 {
                    node.resources = None;
                }
            }
        }

        edges.sort_by(|left, right| {
            (&left.from.node_id, &left.from.port, &left.to.node_id, &left.to.port).cmp(&(
                &right.from.node_id,
                &right.from.port,
                &right.to.node_id,
                &right.to.port,
            ))
        });

        let mut inputs = self.inputs.clone();
        let mut inputs_value = Value::Object(inputs.clone());
        sort_value_maps(&mut inputs_value);
        if let Value::Object(map) = inputs_value {
            inputs = map;
        }

        Graph {
            spec: self.spec.clone(),
            meta: self.meta.clone(),
            inputs,
            nondeterminism_allowed: self.nondeterminism_allowed,
            nodes,
            edges,
        }
    }

    pub fn to_canonical_json(&self) -> Result<String, GraphError> {
        Ok(serde_json::to_string_pretty(&self.canonicalize())?)
    }

    pub fn canonical_json_bytes(&self) -> Result<Vec<u8>, GraphError> {
        Ok(self.to_canonical_json()?.into_bytes())
    }

    pub fn canonical_graph_pretty(&self) -> Result<String, GraphError> {
        self.to_canonical_json()
    }
}

pub fn canonicalize_graph(graph: &Graph) -> Graph {
    graph.canonicalize()
}

pub fn canonical_json(graph: &Graph) -> Result<String, GraphError> {
    graph.to_canonical_json()
}

pub(crate) fn sort_value_maps(value: &mut Value) {
    match value {
        Value::Object(map) => {
            let mut sorted = BTreeMap::new();
            let entries = std::mem::take(map);
            for (key, mut nested) in entries {
                sort_value_maps(&mut nested);
                sorted.insert(key, nested);
            }

            let mut new_map = serde_json::Map::new();
            for (key, nested) in sorted {
                new_map.insert(key, nested);
            }
            *map = new_map;
        }
        Value::Array(items) => {
            for value in items.iter_mut() {
                sort_value_maps(value);
            }
        }
        _ => {}
    }
}

pub(crate) fn sort_param_value(value: &mut ParamValue) {
    match value {
        ParamValue::Array(items) => {
            for value in items.iter_mut() {
                sort_param_value(value);
            }
        }
        ParamValue::Object(map) => {
            let mut sorted = BTreeMap::new();
            let entries = std::mem::take(map);
            for (key, mut nested) in entries {
                sort_param_value(&mut nested);
                sorted.insert(key, nested);
            }
            *map = sorted;
        }
        ParamValue::Ref(_) | ParamValue::Literal(_) => {}
    }
}

pub(crate) fn normalize_rel_path(path: &str) -> String {
    normalize_identity_text(&path.replace('\\', "/"))
}

pub(crate) fn normalize_identity_text(value: &str) -> String {
    value.nfc().collect()
}

pub(crate) fn is_valid_output_path(path: &str) -> bool {
    if path.contains("..") {
        return false;
    }

    let normalized = normalize_rel_path(path);
    if normalized.starts_with('/') {
        return false;
    }

    if normalized.len() > 2 {
        let bytes = normalized.as_bytes();
        if bytes[1] == b':' && bytes[2] == b'/' {
            return false;
        }
    }

    true
}

pub(crate) fn is_valid_canonical_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

pub(crate) fn error(
    code: &str,
    message: String,
    path: String,
    hint: Option<String>,
) -> ValidationDiagnostic {
    ValidationDiagnostic { code: code.to_string(), message, path, hint, severity: Severity::Error }
}

pub(crate) fn warn(
    code: &str,
    message: String,
    path: String,
    hint: Option<String>,
) -> ValidationDiagnostic {
    ValidationDiagnostic {
        code: code.to_string(),
        message,
        path,
        hint,
        severity: Severity::Warning,
    }
}

pub(crate) fn severity_rank(severity: &Severity) -> u8 {
    match severity {
        Severity::Error => 0,
        Severity::Warning => 1,
    }
}

fn effect_order(effect: &Effect) -> u8 {
    match effect {
        Effect::Filesystem => 0,
        Effect::Network => 1,
        Effect::Env => 2,
        Effect::Clock => 3,
    }
}
