use bijux_dag_artifacts::OutputsIndex;
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Serialize)]
pub struct RunDiff {
    pub manifest: BTreeMap<String, Value>,
    pub graph_fingerprint: Option<Value>,
    pub nodes: BTreeMap<String, NodeDiff>,
    pub outputs: BTreeMap<String, OutputDiff>,
    pub replay_equivalence: ReplayEquivalenceReport,
}

#[derive(Debug, Serialize)]
pub struct NodeDiff {
    pub status_a: Option<Value>,
    pub status_b: Option<Value>,
    pub fp_a: Option<Value>,
    pub fp_b: Option<Value>,
}

#[derive(Debug, Serialize)]
pub struct OutputDiff {
    pub added: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ReplayEquivalenceReport {
    pub equivalent: bool,
    pub reasons: Vec<String>,
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
    let mut manifest_diff: BTreeMap<String, Value> = BTreeMap::new();
    let mut ignore = BTreeSet::new();
    ignore.insert("run_id");
    ignore.insert("created_unix_ms");
    ignore.insert("started_unix_ms");
    ignore.insert("finished_unix_ms");
    if let (Some(a), Some(b)) = (manifest_a.as_object(), manifest_b.as_object()) {
        let mut keys = BTreeSet::new();
        for k in a.keys() {
            keys.insert(k.as_str());
        }
        for k in b.keys() {
            keys.insert(k.as_str());
        }
        for k in keys {
            if ignore.contains(k) {
                continue;
            }
            let va = a.get(k);
            let vb = b.get(k);
            if va != vb {
                manifest_diff.insert(k.to_string(), json!({ "a": va, "b": vb }));
            }
        }
    }

    let graph_fingerprint = if graph_fp_a == graph_fp_b {
        None
    } else {
        Some(json!({ "a": graph_fp_a, "b": graph_fp_b }))
    };

    let mut node_diff: BTreeMap<String, NodeDiff> = BTreeMap::new();
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
                NodeDiff {
                    status_a,
                    status_b,
                    fp_a,
                    fp_b,
                },
            );
        }
    }

    let mut out_diff: BTreeMap<String, OutputDiff> = BTreeMap::new();
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
        let mut added = Vec::new();
        let mut removed = Vec::new();
        let mut changed = Vec::new();
        let map_a = a.as_ref().and_then(|v| v.as_object());
        let map_b = b.as_ref().and_then(|v| v.as_object());
        let mut keys = BTreeSet::new();
        if let Some(m) = map_a {
            for k in m.keys() {
                keys.insert(k.clone());
            }
        }
        if let Some(m) = map_b {
            for k in m.keys() {
                keys.insert(k.clone());
            }
        }
        for k in keys {
            let va = map_a.and_then(|m| m.get(&k));
            let vb = map_b.and_then(|m| m.get(&k));
            match (va, vb) {
                (None, Some(_)) => added.push(k),
                (Some(_), None) => removed.push(k),
                (Some(a), Some(b)) => {
                    if a != b {
                        changed.push(k);
                    }
                }
                _ => {}
            }
        }
        added.sort();
        removed.sort();
        changed.sort();
        if !(added.is_empty() && removed.is_empty() && changed.is_empty()) {
            out_diff.insert(
                node_id,
                OutputDiff {
                    added,
                    removed,
                    changed,
                },
            );
        }
    }

    let mut reasons = Vec::new();
    if !manifest_diff.is_empty() {
        reasons.push("manifest fields differ".to_string());
    }
    if graph_fingerprint.is_some() {
        reasons.push("graph fingerprint differs".to_string());
    }
    if !node_diff.is_empty() {
        reasons.push("node status or fingerprint differs".to_string());
    }
    if !out_diff.is_empty() {
        reasons.push("output content differs".to_string());
    }

    RunDiff {
        manifest: manifest_diff,
        graph_fingerprint,
        nodes: node_diff,
        outputs: out_diff,
        replay_equivalence: ReplayEquivalenceReport {
            equivalent: reasons.is_empty(),
            reasons,
        },
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

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dag_artifacts::OutputFile;

    fn index(files: Vec<(&str, &str)>) -> OutputsIndex {
        OutputsIndex {
            files: files
                .into_iter()
                .map(|(p, h)| OutputFile {
                    path: p.to_string(),
                    sha256: h.to_string(),
                    node_id: "n".to_string(),
                    node_fingerprint: "fp".to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn diff_empty_when_identical() {
        let m = json!({"spec":"v","jobs":1});
        let diff = build_run_diff(
            m.clone(),
            m,
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
            &HashMap::new(),
        );
        assert!(diff.manifest.is_empty());
        assert!(diff.graph_fingerprint.is_none());
        assert!(diff.nodes.is_empty());
        assert!(diff.outputs.is_empty());
        assert!(diff.replay_equivalence.equivalent);
        assert!(diff.replay_equivalence.reasons.is_empty());
    }

    #[test]
    fn diff_output_changes_detected() {
        let mut out_a = HashMap::new();
        let mut out_b = HashMap::new();
        out_a.insert("n".to_string(), index(vec![("a.txt", "1")]));
        out_b.insert("n".to_string(), index(vec![("a.txt", "2"), ("b.txt", "3")]));
        let diff = build_run_diff(
            json!({}),
            json!({}),
            "fp".to_string(),
            "fp".to_string(),
            &HashMap::new(),
            &HashMap::new(),
            &out_a,
            &out_b,
        );
        let d = diff.outputs.get("n").unwrap();
        assert_eq!(d.added, vec!["b.txt"]);
        assert_eq!(d.changed, vec!["a.txt"]);
        assert!(!diff.replay_equivalence.equivalent);
        assert!(!diff.replay_equivalence.reasons.is_empty());
    }
}
