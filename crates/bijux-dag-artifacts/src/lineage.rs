use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactLineageEdge {
    pub artifact_id: String,
    pub producer_node_id: String,
    pub upstream_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactLineageSnapshot {
    pub schema_version: String,
    pub edges: Vec<ArtifactLineageEdge>,
}

pub fn write_lineage_snapshot(path: impl AsRef<Path>, snapshot: &ArtifactLineageSnapshot) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let payload = serde_json::to_vec_pretty(snapshot).map_err(|err| err.to_string())?;
    fs::write(path, payload).map_err(|err| err.to_string())
}
