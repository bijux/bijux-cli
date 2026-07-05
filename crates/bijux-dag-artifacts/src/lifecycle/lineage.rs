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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactLineageVisualization {
    pub schema_version: String,
    pub nodes: Vec<String>,
    pub links: Vec<(String, String)>,
}

pub fn write_lineage_snapshot(
    path: impl AsRef<Path>,
    snapshot: &ArtifactLineageSnapshot,
) -> Result<(), String> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let payload = serialize_lineage_snapshot(snapshot)?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

pub fn serialize_lineage_snapshot(snapshot: &ArtifactLineageSnapshot) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(snapshot).map_err(|err| err.to_string())
}

pub fn export_lineage_visualization(
    path: impl AsRef<Path>,
    snapshot: &ArtifactLineageSnapshot,
) -> Result<(), String> {
    let visualization = build_lineage_visualization(snapshot);
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let payload = serialize_lineage_visualization(&visualization)?;
    fs::write(path, payload).map_err(|err| err.to_string())
}

pub fn serialize_lineage_visualization(
    visualization: &ArtifactLineageVisualization,
) -> Result<Vec<u8>, String> {
    serde_json::to_vec_pretty(visualization).map_err(|err| err.to_string())
}

pub fn build_lineage_visualization(
    snapshot: &ArtifactLineageSnapshot,
) -> ArtifactLineageVisualization {
    let mut nodes: Vec<String> = snapshot
        .edges
        .iter()
        .flat_map(|edge| {
            let mut list = vec![edge.artifact_id.clone()];
            list.extend(edge.upstream_artifact_ids.clone());
            list
        })
        .collect();
    nodes.sort();
    nodes.dedup();
    let links: Vec<(String, String)> = snapshot
        .edges
        .iter()
        .flat_map(|edge| {
            edge.upstream_artifact_ids
                .iter()
                .map(|up| (up.clone(), edge.artifact_id.clone()))
                .collect::<Vec<_>>()
        })
        .collect();
    ArtifactLineageVisualization { schema_version: snapshot.schema_version.clone(), nodes, links }
}
