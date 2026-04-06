use crate::commands::MaterializeModeArg;
use crate::{read_file, ExitCode, Graph};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use bijux_dag_artifacts::OutputsIndex;
use bijux_dag_runtime::MaterializeMode;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(serde::Deserialize)]
pub(crate) struct GraphSnapshot {
    pub(crate) graph: Graph,
    pub(crate) graph_fingerprint: String,
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
                let path = run_dir
                    .join("nodes")
                    .join(&node_id)
                    .join("outputs")
                    .join(&file.path);
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
