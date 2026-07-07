use crate::commands::MaterializeModeArg;
use crate::{read_file, ExitCode, Graph};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_core::DynamicExpansionRecord;
use bijux_dag_artifacts::OutputsIndex;
use bijux_dag_runtime::MaterializeMode;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize)]
pub(crate) struct GraphSnapshot {
    pub(crate) graph: Graph,
    pub(crate) graph_fingerprint: String,
    #[serde(default)]
    pub(crate) source_graph: Option<Graph>,
    #[serde(default)]
    pub(crate) source_graph_fingerprint: Option<String>,
    #[serde(default)]
    pub(crate) dynamic_expansions: Vec<DynamicExpansionRecord>,
}

pub(crate) fn env_cache_dir() -> Option<PathBuf> {
    std::env::var("BIJUX_DAG_CACHE_DIR").ok().map(PathBuf::from)
}

pub(crate) fn load_snapshot(run_dir: &Path) -> Result<GraphSnapshot, ExitCode> {
    let snap = read_file(&run_dir.join("graph.snapshot.json"))?;
    serde_json::from_str(&snap).map_err(|_| ExitCode::from(3))
}

pub(crate) fn read_node_traces(
    run_dir: &Path,
) -> Result<HashMap<String, serde_json::Value>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let trace_path = entry.path().join("trace.json");
            if trace_path.exists() {
                let data = read_file(&trace_path)?;
                let val: serde_json::Value =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                map.insert(node_id, val);
            }
        }
    }
    Ok(map)
}

pub(crate) fn read_outputs_indexes(
    run_dir: &Path,
) -> Result<HashMap<String, OutputsIndex>, ExitCode> {
    let mut map = HashMap::new();
    let nodes_dir = run_dir.join("nodes");
    if nodes_dir.exists() {
        let mut entries: Vec<_> = fs::read_dir(nodes_dir)
            .map_err(|_| ExitCode::from(3))?
            .filter_map(|e| e.ok())
            .collect();
        entries.sort_by_key(|e| e.file_name());
        for entry in entries {
            let node_id = entry.file_name().to_string_lossy().to_string();
            let index_path = entry.path().join("outputs").join("index.json");
            if index_path.exists() {
                let data = read_file(&index_path)?;
                let val: OutputsIndex =
                    serde_json::from_str(&data).map_err(|_| ExitCode::from(3))?;
                map.insert(node_id, val);
            }
        }
    }
    Ok(map)
}

pub(crate) fn collect_output_files(
    run_dir: &Path,
    outputs: &HashMap<String, OutputsIndex>,
) -> Result<serde_json::Value, ExitCode> {
    let mut out: BTreeMap<String, serde_json::Value> = BTreeMap::new();
    let mut node_ids: Vec<String> = outputs.keys().cloned().collect();
    node_ids.sort();
    for node_id in node_ids {
        let mut files: BTreeMap<String, String> = BTreeMap::new();
        if let Some(index) = outputs.get(&node_id) {
            let mut entries = index.files.clone();
            entries.sort_by(|a, b| a.path.cmp(&b.path));
            for file in entries {
                let path = run_dir.join("nodes").join(&node_id).join("outputs").join(&file.path);
                let bytes = fs::read(path).map_err(|_| ExitCode::from(3))?;
                let encoded = BASE64.encode(bytes);
                files.insert(file.path, encoded);
            }
        }
        out.insert(node_id, serde_json::to_value(files).unwrap());
    }
    Ok(serde_json::to_value(out).unwrap())
}

pub(crate) fn map_materialize_mode(arg: MaterializeModeArg) -> MaterializeMode {
    match arg {
        MaterializeModeArg::Copy => MaterializeMode::Copy,
        MaterializeModeArg::Hardlink => MaterializeMode::Hardlink,
        MaterializeModeArg::Symlink => MaterializeMode::Symlink,
    }
}

#[cfg(test)]
mod tests {
    use super::load_snapshot;
    use bijux_dag_core::parse_graph_strict;
    use std::fs;

    #[test]
    fn load_snapshot_accepts_dynamic_expansion_metadata() {
        let dir = tempfile::tempdir().expect("tempdir");
        let graph = parse_graph_strict(
            r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"publish","kind":"const","outputs":[{"name":"out","path":"publish/out"}],"params":{"value":"ok"}}],"edges":[]}"#,
        )
        .expect("graph");
        let source_graph = parse_graph_strict(
            r#"{"spec":"bijux-dag/v0.1","nodes":[{"id":"expand","kind":"const","semantic_kind":"dynamic","outputs":[{"name":"expansion","path":"expand/expansion.json","kind":"value"}],"params":{"value":{}},"dynamic":{"expansion_output":"expansion"}}],"edges":[]}"#,
        )
        .expect("source graph");
        fs::write(
            dir.path().join("graph.snapshot.json"),
            serde_json::to_vec_pretty(&serde_json::json!({
                "graph": graph.canonicalize(),
                "graph_fingerprint": "expanded-fp",
                "source_graph": source_graph.canonicalize(),
                "source_graph_fingerprint": "source-fp",
                "dynamic_expansions": [
                    {
                        "controller_node_id": "expand",
                        "expansion_output": "expansion",
                        "expansion_fingerprint": "expansion-fp",
                        "generated_node_ids": ["expand__publish"],
                        "generated_edge_count": 1
                    }
                ]
            }))
            .expect("snapshot bytes"),
        )
        .expect("write snapshot");

        let snapshot = load_snapshot(dir.path()).expect("load snapshot");
        assert_eq!(snapshot.graph_fingerprint, "expanded-fp");
        assert_eq!(snapshot.source_graph_fingerprint.as_deref(), Some("source-fp"));
        assert_eq!(snapshot.dynamic_expansions.len(), 1);
        assert_eq!(snapshot.dynamic_expansions[0].controller_node_id, "expand");
        assert_eq!(
            snapshot
                .source_graph
                .as_ref()
                .and_then(|source| source.nodes.first())
                .map(|node| node.id.as_str()),
            Some("expand")
        );
    }
}
