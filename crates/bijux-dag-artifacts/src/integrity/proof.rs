use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactIntegrityProof {
    pub artifact_id: String,
    pub file_sha256: String,
    pub schema_name: String,
    pub schema_version: String,
    pub producer_node_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorruptionDetectionResult {
    pub corrupt_detected: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorruptionRepairPolicy {
    pub attempt_rebuild_from_cache: bool,
    pub attempt_replay: bool,
    pub fail_if_unrecoverable: bool,
}
