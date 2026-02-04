use bijux_dag_artifacts::OutputsIndex;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeSet, HashMap};

#[derive(Debug, Serialize)]
pub struct RunDiff {
    pub manifest_changes: Value,
    pub graph_fingerprint: Value,
    pub node_changes: Value,
    pub output_changes: Value,
}

#[allow(clippy::too_many_arguments)]
pub fn build_run_diff(
    manifest_a: Value,
    manifest_b: Value,
    graph_fp_a: String,
    graph_fp_b: String,
    nodes_a: &HashMap<String, Value>,
    nodes_b: &HashMap<String, Value>,
    outputs_a: &HashMap<String, OutputsIndex>,
    outputs_b: &HashMap<String, OutputsIndex>,
) -> RunDiff {
    let mut node_diff = serde_json::Map::new();
    let mut all_nodes: BTreeSet<String> = BTreeSet::new();
    for k in nodes_a.keys() {
        all_nodes.insert(k.clone());
    }
    for k in nodes_b.keys() {
        all_nodes.insert(k.clone());
    }
    for node_id in all_nodes {
        let a = nodes_a.get(&node_id);
        let b = nodes_b.get(&node_id);
        let status_a = a.and_then(|v| v.get("status")).cloned();
        let status_b = b.and_then(|v| v.get("status")).cloned();
        let fp_a = a.and_then(|v| v.get("fingerprint")).cloned();
        let fp_b = b.and_then(|v| v.get("fingerprint")).cloned();
        if status_a != status_b || fp_a != fp_b {
            node_diff.insert(
                node_id,
                json!({
                    "status": {"a": status_a, "b": status_b},
                    "fingerprint": {"a": fp_a, "b": fp_b},
                }),
            );
        }
    }

    let mut out_diff = serde_json::Map::new();
    let mut all_nodes: BTreeSet<String> = BTreeSet::new();
    for k in outputs_a.keys() {
        all_nodes.insert(k.clone());
    }
    for k in outputs_b.keys() {
        all_nodes.insert(k.clone());
    }
    for node_id in all_nodes {
        let a = outputs_a.get(&node_id).map(outputs_index_to_map);
        let b = outputs_b.get(&node_id).map(outputs_index_to_map);
        if a != b {
            out_diff.insert(node_id, json!({ "a": a, "b": b }));
        }
    }

    RunDiff {
        manifest_changes: json!({ "a": manifest_a, "b": manifest_b }),
        graph_fingerprint: json!({ "a": graph_fp_a, "b": graph_fp_b }),
        node_changes: json!(node_diff),
        output_changes: json!(out_diff),
    }
}

fn outputs_index_to_map(index: &OutputsIndex) -> Value {
    let mut files = index.files.clone();
    files.sort_by(|a, b| a.path.cmp(&b.path));
    let mut map = serde_json::Map::new();
    for f in files {
        map.insert(f.path, json!(f.sha256));
    }
    Value::Object(map)
}
