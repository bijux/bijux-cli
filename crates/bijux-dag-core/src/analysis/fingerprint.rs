//! Graph fingerprint entrypoints.

use crate::canonical::{normalize_identity_text, normalize_rel_path, sort_value_maps};
use crate::expansion::expand_graph;
use crate::resolve::{resolve_command_argv_templates, resolve_param_value};
use crate::{Graph, GraphError, GraphFingerprintExplain, GraphId, Node, ParamValue};
use sha2::{Digest, Sha256};

impl Graph {
    pub fn graph_fingerprint(&self) -> Result<String, GraphError> {
        let expanded = expand_graph(self).map_err(|_| GraphError::ValidationFailed)?;
        let canonical_json = serde_json::to_string_pretty(&expanded.canonicalize())?;
        Ok(hash_bytes(canonical_json.as_bytes()))
    }

    pub fn graph_id(&self) -> Result<GraphId, GraphError> {
        Ok(GraphId(self.graph_fingerprint()?))
    }

    pub fn graph_fingerprint_explain(&self) -> Result<GraphFingerprintExplain, GraphError> {
        let expanded = expand_graph(self).map_err(|_| GraphError::ValidationFailed)?;
        let canonical_json = serde_json::to_string_pretty(&expanded.canonicalize())?;
        Ok(GraphFingerprintExplain {
            graph_id: GraphId(hash_bytes(canonical_json.as_bytes())),
            canonical_json_bytes_len: canonical_json.len(),
            canonical_json,
            hash_algorithm: "sha256".to_string(),
        })
    }

    pub fn node_fingerprint(&self, node: &Node) -> Result<String, GraphError> {
        let resolved = resolve_param_value(&node.params, self)?;
        self.node_fingerprint_with_params(node, &resolved)
    }

    pub fn node_fingerprint_with_params(
        &self,
        node: &Node,
        resolved_params: &serde_json::Value,
    ) -> Result<String, GraphError> {
        let mut node = node.clone();
        node.id = normalize_identity_text(&node.id);
        node.inputs = node.inputs.iter().map(|input| normalize_identity_text(input)).collect();
        let mut params = resolved_params.clone();
        sort_value_maps(&mut params);
        node.params = ParamValue::Literal(params);
        node.inputs.sort();
        for output in &mut node.outputs {
            output.name = normalize_identity_text(&output.name);
            output.path = normalize_rel_path(&output.path);
        }
        node.outputs.sort_by(|left, right| left.name.cmp(&right.name));
        node.effects.sort_by_key(|effect| match effect {
            crate::Effect::Filesystem => 0,
            crate::Effect::Network => 1,
            crate::Effect::Env => 2,
            crate::Effect::Clock => 3,
        });
        if let Some(argv) = node.container.as_ref().map(|container| container.argv.clone()) {
            let resolved_argv =
                resolve_command_argv_templates(self, &node, &argv, resolved_params)?;
            if let Some(container) = node.container.as_mut() {
                container.argv = resolved_argv;
            }
        }
        node.env_allowlist =
            node.env_allowlist.iter().map(|entry| normalize_identity_text(entry)).collect();
        node.env_allowlist.sort();
        node.group = None;
        let json = serde_json::to_string_pretty(&node)?;
        Ok(hash_bytes(json.as_bytes()))
    }
}

pub fn graph_fingerprint(graph: &Graph) -> Result<String, GraphError> {
    graph.graph_fingerprint()
}

pub fn canonical_json(graph: &Graph) -> Result<String, GraphError> {
    graph.to_canonical_json()
}

pub(crate) fn hash_bytes(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}
