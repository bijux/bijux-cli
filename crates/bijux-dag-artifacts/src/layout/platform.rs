use crate::index::ArtifactId;
use crate::lineage::ArtifactLineageSnapshot;
use crate::store::ArtifactStoreBackend;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactStoreClass {
    HotCache,
    DurableLocal,
    RemoteObject,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactStoreRoute {
    pub artifact_id: ArtifactId,
    pub store_class: ArtifactStoreClass,
    pub storage_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactReplicationRule {
    pub from: ArtifactStoreClass,
    pub to: ArtifactStoreClass,
    pub require_integrity_proof: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactReplicationRecord {
    pub artifact_id: ArtifactId,
    pub from: ArtifactStoreClass,
    pub to: ArtifactStoreClass,
    pub promoted_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactPackingProfile {
    FastReplay,
    LongTermArchive,
    ComplianceEvidence,
    Handoff,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactCompressionPolicy {
    pub profile: ArtifactPackingProfile,
    pub schema_aware: bool,
    pub level: u8,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactChunkPolicy {
    pub chunk_bytes: u64,
    pub max_chunks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactChunkDescriptor {
    pub artifact_id: ArtifactId,
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub content_sha256: String,
    pub bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactSigningHook {
    pub signer_id: String,
    pub algorithm: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactProvenanceRecord {
    pub artifact_id: ArtifactId,
    pub producer_binary_sha256: String,
    pub adapter_version: String,
    pub environment_class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactedLineageSnapshot {
    pub schema_version: String,
    pub artifact_count: usize,
    pub edge_count: usize,
    pub producer_index: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactQuery {
    pub producer_node_id: Option<String>,
    pub schema_name: Option<String>,
    pub run_id: Option<String>,
    pub tag: Option<String>,
    pub lineage_artifact_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactSearchIndex {
    pub by_logical_name: BTreeMap<String, ArtifactId>,
    pub by_metadata_key: BTreeMap<String, Vec<ArtifactId>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactVerificationReport {
    pub generated_unix_ms: u128,
    pub verified_artifacts: Vec<ArtifactId>,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactRetentionClass {
    Transient,
    Retained,
    Release,
    Audit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactGarbageCollectionPlan {
    pub preserved_artifacts: Vec<ArtifactId>,
    pub collectable_artifacts: Vec<ArtifactId>,
    pub lineage_snapshot_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactGarbageCollectionExplainEntry {
    pub artifact_id: ArtifactId,
    pub action: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactGarbageCollectionExplain {
    pub lineage_snapshot_id: String,
    pub entries: Vec<ArtifactGarbageCollectionExplainEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactImportCompatibility {
    pub source_spec_version: String,
    pub target_spec_version: String,
    pub source_environment: String,
    pub target_environment: String,
    pub compatible: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ArtifactExportProfile {
    Handoff,
    Backup,
    Replication,
    ComplianceEvidence,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactRedactionPolicy {
    pub redact_log_values: bool,
    pub redact_metadata_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImmutableArtifactAnnotation {
    pub artifact_id: ArtifactId,
    pub key: String,
    pub value: String,
    pub created_by: String,
    pub created_unix_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayAssistContext {
    pub selected_artifact_id: ArtifactId,
    pub required_upstream_artifacts: Vec<ArtifactId>,
    pub required_nodes: Vec<String>,
}

pub fn compact_lineage(snapshot: &ArtifactLineageSnapshot) -> CompactedLineageSnapshot {
    let mut producer_index: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut artifacts = BTreeSet::new();
    let mut edges = 0usize;

    for edge in &snapshot.edges {
        artifacts.insert(edge.artifact_id.clone());
        edges += edge.upstream_artifact_ids.len();
        producer_index
            .entry(edge.producer_node_id.clone())
            .or_default()
            .push(edge.artifact_id.clone());
        for upstream in &edge.upstream_artifact_ids {
            artifacts.insert(upstream.clone());
        }
    }

    for values in producer_index.values_mut() {
        values.sort();
    }

    CompactedLineageSnapshot {
        schema_version: snapshot.schema_version.clone(),
        artifact_count: artifacts.len(),
        edge_count: edges,
        producer_index,
    }
}

pub fn plan_lineage_safe_gc(
    referenced_artifacts: &[ArtifactId],
    all_artifacts: &[ArtifactId],
    lineage_snapshot_id: impl Into<String>,
) -> ArtifactGarbageCollectionPlan {
    let referenced: BTreeSet<_> = referenced_artifacts.iter().cloned().collect();
    let mut preserved = Vec::new();
    let mut collectable = Vec::new();

    for artifact in all_artifacts {
        if referenced.contains(artifact) {
            preserved.push(artifact.clone());
        } else {
            collectable.push(artifact.clone());
        }
    }

    ArtifactGarbageCollectionPlan {
        preserved_artifacts: preserved,
        collectable_artifacts: collectable,
        lineage_snapshot_id: lineage_snapshot_id.into(),
    }
}

pub fn explain_lineage_safe_gc(
    referenced_artifacts: &[ArtifactId],
    all_artifacts: &[ArtifactId],
    lineage_snapshot_id: impl Into<String>,
) -> ArtifactGarbageCollectionExplain {
    let referenced: BTreeSet<_> = referenced_artifacts.iter().cloned().collect();
    let mut entries = Vec::new();
    for artifact in all_artifacts {
        if referenced.contains(artifact) {
            entries.push(ArtifactGarbageCollectionExplainEntry {
                artifact_id: artifact.clone(),
                action: "preserve".to_string(),
                reason: "artifact is referenced by active lineage".to_string(),
            });
        } else {
            entries.push(ArtifactGarbageCollectionExplainEntry {
                artifact_id: artifact.clone(),
                action: "collect".to_string(),
                reason: "artifact is unreferenced in active lineage snapshot".to_string(),
            });
        }
    }
    ArtifactGarbageCollectionExplain {
        lineage_snapshot_id: lineage_snapshot_id.into(),
        entries,
    }
}

pub fn lineage_dependencies(snapshot: &ArtifactLineageSnapshot, artifact_id: &str) -> Vec<String> {
    let mut deps = Vec::new();
    for edge in &snapshot.edges {
        if edge.artifact_id == artifact_id {
            deps.extend(edge.upstream_artifact_ids.clone());
        }
    }
    deps.sort();
    deps.dedup();
    deps
}

pub fn lineage_dependents(snapshot: &ArtifactLineageSnapshot, artifact_id: &str) -> Vec<String> {
    let mut dependents = Vec::new();
    for edge in &snapshot.edges {
        if edge
            .upstream_artifact_ids
            .iter()
            .any(|up| up == artifact_id)
        {
            dependents.push(edge.artifact_id.clone());
        }
    }
    dependents.sort();
    dependents.dedup();
    dependents
}

pub fn build_replay_assist(
    snapshot: &ArtifactLineageSnapshot,
    artifact_id: ArtifactId,
) -> ReplayAssistContext {
    let required_upstream_artifacts = lineage_dependencies(snapshot, &artifact_id.0)
        .into_iter()
        .map(ArtifactId)
        .collect::<Vec<_>>();

    let mut required_nodes = snapshot
        .edges
        .iter()
        .filter(|edge| edge.artifact_id == artifact_id.0)
        .map(|edge| edge.producer_node_id.clone())
        .collect::<Vec<_>>();
    required_nodes.sort();
    required_nodes.dedup();

    ReplayAssistContext {
        selected_artifact_id: artifact_id,
        required_upstream_artifacts,
        required_nodes,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactConformanceReport {
    pub backend_name: String,
    pub write_ok: bool,
    pub read_ok: bool,
    pub roundtrip_ok: bool,
    pub errors: Vec<String>,
}

pub fn run_store_conformance(
    backend_name: impl Into<String>,
    backend: &dyn ArtifactStoreBackend,
) -> ArtifactConformanceReport {
    let key = "conformance/ping.txt";
    let bytes = b"bijux-artifact-store-conformance";
    let mut errors = Vec::new();

    let write_ok = backend.write_bytes(key, bytes).is_ok();
    if !write_ok {
        errors.push("write_bytes failed".to_string());
    }

    let read_result = backend.read_bytes(key);
    let read_ok = read_result.is_ok();
    if !read_ok {
        errors.push("read_bytes failed".to_string());
    }

    let roundtrip_ok = read_result.map(|v| v == bytes).unwrap_or(false);
    if read_ok && !roundtrip_ok {
        errors.push("read_bytes content mismatch".to_string());
    }

    ArtifactConformanceReport {
        backend_name: backend_name.into(),
        write_ok,
        read_ok,
        roundtrip_ok,
        errors,
    }
}
