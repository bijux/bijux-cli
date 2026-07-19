//! DAG canonicalization entrypoints and helpers.

use crate::{
    BranchSpec, EdgeKind, Effect, Graph, GraphError, NodeOutputRef, ParamValue, Severity,
    ValidationDiagnostic,
};
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
            if let Some(resources) = &mut node.resources {
                resources.named_resources = resources
                    .named_resources
                    .iter()
                    .map(|(name, amount)| (normalize_identity_text(name), *amount))
                    .collect();
            }
            node.env_allowlist =
                node.env_allowlist.iter().map(|entry| normalize_identity_text(entry)).collect();
            node.tags = node.tags.iter().map(|entry| normalize_identity_text(entry)).collect();
            if let Some(group) = &node.group {
                node.group = Some(normalize_identity_text(group));
            }
            if let Some(branch) = &node.branch {
                node.branch = Some(BranchSpec {
                    decisions: branch
                        .decisions
                        .iter()
                        .map(|decision| normalize_identity_text(decision))
                        .collect(),
                    default_decision: branch
                        .default_decision
                        .as_ref()
                        .map(|decision| normalize_identity_text(decision)),
                    decision_output: normalize_identity_text(&branch.decision_output),
                });
            }
        }

        for edge in &mut edges {
            if let Some(id) = &edge.id {
                edge.id = Some(normalize_identity_text(id));
            }
            if let Some(decision) = &edge.decision {
                edge.decision = Some(normalize_identity_text(decision));
            }
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
            if let Some(branch) = &mut node.branch {
                branch.decisions.sort();
            }
            if let Some(resources) = &node.resources {
                if resources.cpu == 0
                    && resources.mem_mb == 0
                    && resources.gpu_devices == 0
                    && resources.named_resources.is_empty()
                {
                    node.resources = None;
                }
            }
        }

        edges.sort_by(|left, right| {
            (
                edge_kind_order(&left.kind),
                &left.from.node_id,
                &left.from.port,
                &left.to.node_id,
                &left.to.port,
                &left.id,
                &left.decision,
            )
                .cmp(&(
                    edge_kind_order(&right.kind),
                    &right.from.node_id,
                    &right.from.port,
                    &right.to.node_id,
                    &right.to.port,
                    &right.id,
                    &right.decision,
                ))
        });

        let mut inputs = self.inputs.clone();
        let mut inputs_value = serde_json::to_value(&inputs)
            .expect("graph inputs should serialize for canonicalization");
        sort_value_maps(&mut inputs_value);
        inputs = serde_json::from_value(inputs_value)
            .expect("graph inputs should deserialize canonically");

        let subgraphs = self
            .subgraphs
            .iter()
            .map(|(name, definition)| {
                let mut outputs = definition
                    .outputs
                    .iter()
                    .map(|(export_name, reference)| {
                        (
                            normalize_identity_text(export_name),
                            NodeOutputRef {
                                node_id: normalize_identity_text(&reference.node_id),
                                output_name: normalize_identity_text(&reference.output_name),
                            },
                        )
                    })
                    .collect::<Vec<_>>();
                outputs.sort_by(|left, right| left.0.cmp(&right.0));
                (
                    normalize_identity_text(name),
                    crate::SubgraphDefinition {
                        graph: definition.graph.canonicalize(),
                        outputs: outputs.into_iter().collect(),
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        let mut subgraph_instances = self
            .subgraph_instances
            .iter()
            .cloned()
            .map(|mut instance| {
                instance.id = normalize_identity_text(&instance.id);
                instance.subgraph = normalize_identity_text(&instance.subgraph);
                instance.input_bindings = instance
                    .input_bindings
                    .into_iter()
                    .map(|(name, mut value)| {
                        sort_param_value(&mut value);
                        (normalize_identity_text(&name), value)
                    })
                    .collect();
                instance
            })
            .collect::<Vec<_>>();
        subgraph_instances.sort_by(|left, right| {
            (&left.id, &left.subgraph, left.input_bindings.len()).cmp(&(
                &right.id,
                &right.subgraph,
                right.input_bindings.len(),
            ))
        });

        Graph {
            spec: self.spec.clone(),
            meta: self.meta.clone(),
            inputs,
            nondeterminism_allowed: self.nondeterminism_allowed,
            subgraphs,
            subgraph_instances,
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

pub(crate) fn is_valid_tag_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ':'))
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

fn edge_kind_order(kind: &EdgeKind) -> u8 {
    match kind {
        EdgeKind::Data => 0,
        EdgeKind::Control => 1,
        EdgeKind::Conditional => 2,
    }
}
