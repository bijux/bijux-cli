use crate::{
    sha256_hex, write_json_atomic_durable, ArtifactError, Manifest, PromotedOutputSummary,
};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PromotionEnvironment {
    Local,
    Staging,
    Release,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionLineageSummary {
    pub subject_artifact_id: String,
    pub subject_legacy_artifact_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub upstream_artifact_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub downstream_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactPromotionRecord {
    pub schema_version: String,
    pub canonical_artifact_id: String,
    pub legacy_artifact_id: String,
    pub source_run_id: String,
    pub source_node_id: String,
    pub source_output_name: String,
    pub source_output_path: String,
    pub artifact_sha256: String,
    pub payload_kind: String,
    pub payload_relpath: String,
    pub destination_path: String,
    pub from: PromotionEnvironment,
    pub to: PromotionEnvironment,
    pub promoted_unix_ms: u128,
    pub lineage: PromotionLineageSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactPromotionIndex {
    pub schema_version: String,
    pub records: Vec<ArtifactPromotionRecord>,
}

impl PromotionEnvironment {
    pub fn label(&self) -> &str {
        match self {
            Self::Local => "local",
            Self::Staging => "staging",
            Self::Release => "release",
            Self::Custom(label) => label.as_str(),
        }
    }
}

pub fn build_promoted_output_summary(record: &ArtifactPromotionRecord) -> PromotedOutputSummary {
    PromotedOutputSummary {
        canonical_artifact_id: record.canonical_artifact_id.clone(),
        legacy_artifact_id: record.legacy_artifact_id.clone(),
        node_id: record.source_node_id.clone(),
        output_name: record.source_output_name.clone(),
        artifact_sha256: record.artifact_sha256.clone(),
        destination_path: record.destination_path.clone(),
        target_environment: record.to.label().to_string(),
        promoted_unix_ms: record.promoted_unix_ms,
    }
}

pub fn append_promotion_summary(manifest: &mut Manifest, summary: PromotedOutputSummary) {
    let run_summary = manifest.run_summary.get_or_insert_with(|| crate::RunSummary {
        total_nodes: manifest.node_counts.success
            + manifest.node_counts.failed
            + manifest.node_counts.skipped
            + manifest.node_counts.cached
            + manifest.node_counts.cancelled,
        success: manifest.node_counts.success,
        failed: manifest.node_counts.failed,
        skipped: manifest.node_counts.skipped,
        cached: manifest.node_counts.cached,
        cancelled: manifest.node_counts.cancelled,
        promoted_outputs: Vec::new(),
    });
    if let Some(existing) = run_summary.promoted_outputs.iter_mut().find(|entry| {
        entry.canonical_artifact_id == summary.canonical_artifact_id
            && entry.destination_path == summary.destination_path
    }) {
        *existing = summary;
        return;
    }
    run_summary.promoted_outputs.push(summary);
    run_summary.promoted_outputs.sort_by(|left, right| {
        (&left.node_id, &left.output_name, &left.destination_path).cmp(&(
            &right.node_id,
            &right.output_name,
            &right.destination_path,
        ))
    });
}

pub fn append_promotion_record(
    run_dir: impl AsRef<Path>,
    record: &ArtifactPromotionRecord,
) -> Result<(), ArtifactError> {
    let run_dir = run_dir.as_ref();
    let record_path = promotion_record_path(run_dir, &record.canonical_artifact_id);
    let index_path = run_dir.join("promotions").join("index.json");

    let mut index = if index_path.exists() {
        let raw = fs::read_to_string(&index_path)?;
        serde_json::from_str::<ArtifactPromotionIndex>(&raw)?
    } else {
        ArtifactPromotionIndex {
            schema_version: "artifact-promotions/v0.1".to_string(),
            records: Vec::new(),
        }
    };

    if let Some(existing) = index.records.iter_mut().find(|entry| {
        entry.canonical_artifact_id == record.canonical_artifact_id
            && entry.destination_path == record.destination_path
    }) {
        *existing = record.clone();
    } else {
        index.records.push(record.clone());
        index.records.sort_by(|left, right| {
            (&left.source_node_id, &left.source_output_name, &left.destination_path).cmp(&(
                &right.source_node_id,
                &right.source_output_name,
                &right.destination_path,
            ))
        });
    }

    let record_value = serde_json::to_value(record)?;
    write_json_atomic_durable(&record_path, &record_value)?;
    let index_value = serde_json::to_value(&index)?;
    write_json_atomic_durable(index_path, &index_value)
}

pub fn promotion_record_path(run_dir: impl AsRef<Path>, canonical_artifact_id: &str) -> PathBuf {
    let slug = &sha256_hex(canonical_artifact_id.as_bytes())[..24];
    run_dir.as_ref().join("promotions").join(format!("{slug}.json"))
}

#[cfg(test)]
mod tests {
    use super::{
        append_promotion_record, append_promotion_summary, build_promoted_output_summary,
        ArtifactPromotionRecord, PromotionEnvironment, PromotionLineageSummary,
    };
    use crate::{Manifest, NodeCounts, PolicyInfo};

    fn sample_record(destination_path: &str) -> ArtifactPromotionRecord {
        ArtifactPromotionRecord {
            schema_version: "artifact-promotion/v0.1".to_string(),
            canonical_artifact_id:
                "run=run-1;node=publish;path=nodes/publish/outputs/report.json;sha256=abc"
                    .to_string(),
            legacy_artifact_id: "publish:report.json".to_string(),
            source_run_id: "run-1".to_string(),
            source_node_id: "publish".to_string(),
            source_output_name: "report".to_string(),
            source_output_path: "nodes/publish/outputs/report.json".to_string(),
            artifact_sha256: "abc".to_string(),
            payload_kind: "file".to_string(),
            payload_relpath: "payload/report.json".to_string(),
            destination_path: destination_path.to_string(),
            from: PromotionEnvironment::Local,
            to: PromotionEnvironment::Release,
            promoted_unix_ms: 42,
            lineage: PromotionLineageSummary {
                subject_artifact_id:
                    "run=run-1;node=publish;path=nodes/publish/outputs/report.json;sha256=abc"
                        .to_string(),
                subject_legacy_artifact_id: "publish:report.json".to_string(),
                upstream_artifact_ids: vec!["extract:seed.csv".to_string()],
                downstream_artifact_ids: Vec::new(),
            },
        }
    }

    fn sample_manifest() -> Manifest {
        Manifest {
            manifest_version: "run-manifest/v0.1".to_string(),
            run_id: "run-1".to_string(),
            created_unix_ms: 1,
            started_unix_ms: 1,
            finished_unix_ms: 2,
            graph_snapshot: "graph.snapshot.json".to_string(),
            status: "success".to_string(),
            spec: "bijux-dag/v0.1".to_string(),
            graph_fingerprint: "g1".to_string(),
            planner_contract_version: "planner-contract/v0.1".to_string(),
            planner_fingerprint: None,
            execution_fingerprint: None,
            evidence_fingerprint: None,
            tool_version: "0.4.0".to_string(),
            jobs: 1,
            adapters: Vec::new(),
            outputs: Vec::new(),
            node_counts: NodeCounts { success: 1, failed: 0, skipped: 0, cached: 0, cancelled: 0 },
            policy: PolicyInfo {
                deny_network: true,
                deny_env: true,
                deny_clock: true,
                clean_env: true,
                container_image_reference_policy:
                    crate::ContainerImageReferencePolicy::RequireDigest,
            },
            cache_mode: None,
            cache_dir: None,
            run_timeout_ms: None,
            run_timeout_behavior: None,
            run_cancellation_cause: None,
            run_metadata: None,
            run_summary: None,
        }
    }

    #[test]
    fn append_promotion_summary_deduplicates_by_artifact_and_destination() {
        let mut manifest = sample_manifest();
        let summary = build_promoted_output_summary(&sample_record("deliverables/run-1/publish"));
        append_promotion_summary(&mut manifest, summary.clone());
        append_promotion_summary(&mut manifest, summary);
        assert_eq!(manifest.run_summary.as_ref().expect("summary").promoted_outputs.len(), 1);
    }

    #[test]
    fn append_promotion_record_upserts_existing_destination_record() {
        let dir = tempfile::tempdir().expect("tmp");
        let mut first = sample_record("deliverables/run-1/publish");
        append_promotion_record(dir.path(), &first).expect("first record");
        first.promoted_unix_ms = 99;
        append_promotion_record(dir.path(), &first).expect("upsert");
        let raw = std::fs::read_to_string(dir.path().join("promotions").join("index.json"))
            .expect("index");
        let parsed: super::ArtifactPromotionIndex =
            serde_json::from_str(&raw).expect("parse index");
        assert_eq!(parsed.records.len(), 1);
        assert_eq!(parsed.records[0].promoted_unix_ms, 99);
    }
}
