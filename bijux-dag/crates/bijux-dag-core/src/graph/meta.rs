use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DagId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DagVersionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotMetadata {
    pub dag_id: DagId,
    pub dag_version_id: DagVersionId,
    pub created_at: String,
    pub created_by: String,
    pub spec_version: String,
    pub graph_fingerprint: String,
}
