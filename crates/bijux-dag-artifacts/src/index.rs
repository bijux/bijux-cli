use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ArtifactId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactAlias {
    pub alias: String,
    pub artifact_id: ArtifactId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactOutputClass {
    Primary,
    Diagnostics,
    Logs,
    SideArtifact,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactMaterializationRecord {
    pub artifact_id: ArtifactId,
    pub source: ArtifactMaterializationSource,
    pub recorded_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactMaterializationSource {
    Produced,
    CacheReuse,
    Imported,
    Promoted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactPackManifest {
    pub pack_manifest_version: String,
    pub artifacts: Vec<ArtifactId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactDedupMetrics {
    pub total_artifacts: u64,
    pub unique_content_hashes: u64,
    pub deduplicated_artifacts: u64,
}

pub fn normalize_metadata_pairs(mut pairs: Vec<(String, String)>) -> Vec<(String, String)> {
    pairs.sort_by(|a, b| a.0.cmp(&b.0));
    pairs
}

pub fn dedup_metrics_for_hashes(hashes: &[String]) -> ArtifactDedupMetrics {
    use std::collections::BTreeSet;
    let unique: BTreeSet<_> = hashes.iter().collect();
    let total = hashes.len() as u64;
    let unique_count = unique.len() as u64;
    ArtifactDedupMetrics {
        total_artifacts: total,
        unique_content_hashes: unique_count,
        deduplicated_artifacts: total.saturating_sub(unique_count),
    }
}
