use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatasetId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct DatasetVersionId(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetSchemaContract {
    pub dataset_id: DatasetId,
    pub version_id: DatasetVersionId,
    pub schema_ref: String,
    pub partitioned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetBinding {
    pub logical_name: String,
    pub physical_binding: String,
    pub promotion_history: Vec<DatasetVersionId>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetLineageRecord {
    pub dataset_id: DatasetId,
    pub producer_runs: BTreeSet<String>,
    pub consumer_runs: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetPartitionStrategy {
    Time,
    Key,
    Range,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetPartitionModel {
    pub strategy: DatasetPartitionStrategy,
    pub partition_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetCompleteness {
    Full,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetFreshnessPolicy {
    pub max_age_minutes: u32,
    pub staleness_action: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetQualityState {
    pub validation_outcomes: Vec<String>,
    pub quality_score: f64,
    pub acceptance_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetPublicationWorkflow {
    pub dataset_id: DatasetId,
    pub publication_steps: Vec<String>,
    pub separates_artifact_materialization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetRetentionPolicy {
    pub retention_days: u32,
    pub archival_tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetImmutability {
    AppendOnly,
    VersionedSnapshot,
    MutablePointer,
    DerivedView,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DatasetConsumptionMode {
    StableVersion(DatasetVersionId),
    LatestApproved,
    FreshnessBounded(u32),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetConsumptionContract {
    pub dataset_id: DatasetId,
    pub mode: DatasetConsumptionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetArtifactMapping {
    pub dataset_id: DatasetId,
    pub version_id: DatasetVersionId,
    pub artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetDiffReport {
    pub from_version: DatasetVersionId,
    pub to_version: DatasetVersionId,
    pub compatibility: String,
    pub differences: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DatasetProvenanceReport {
    pub dataset_id: DatasetId,
    pub producer_count: usize,
    pub consumer_count: usize,
    pub validation_pass_rate: f64,
    pub promotions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetReadinessGate {
    pub dataset_id: DatasetId,
    pub accepted: bool,
    pub required_for_schedule: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetCatalogQuery {
    pub schema_ref: Option<String>,
    pub owner: Option<String>,
    pub freshness_max_minutes: Option<u32>,
    pub quality_state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DatasetCatalogEntry {
    pub dataset_id: DatasetId,
    pub version_id: DatasetVersionId,
    pub schema_ref: String,
    pub owner: String,
    pub freshness_minutes: u32,
    pub quality_state: String,
}

pub fn dataset_consumption_satisfied(
    contract: &DatasetConsumptionContract,
    available_version: &DatasetVersionId,
    approved_latest: &DatasetVersionId,
    freshness_minutes: u32,
) -> bool {
    match &contract.mode {
        DatasetConsumptionMode::StableVersion(required) => required == available_version,
        DatasetConsumptionMode::LatestApproved => approved_latest == available_version,
        DatasetConsumptionMode::FreshnessBounded(limit) => freshness_minutes <= *limit,
    }
}

pub fn dataset_ready_for_schedule(gates: &[DatasetReadinessGate]) -> bool {
    gates.iter().all(|gate| !gate.required_for_schedule || gate.accepted)
}

pub fn dataset_diff(
    from_version: &DatasetVersionId,
    to_version: &DatasetVersionId,
    from_schema: &str,
    to_schema: &str,
) -> DatasetDiffReport {
    let compatibility = if from_schema == to_schema { "compatible" } else { "migration-required" };

    let mut differences = Vec::new();
    if from_schema != to_schema {
        differences.push(format!("schema changed: {from_schema} -> {to_schema}"));
    }

    DatasetDiffReport {
        from_version: from_version.clone(),
        to_version: to_version.clone(),
        compatibility: compatibility.to_string(),
        differences,
    }
}

pub fn dataset_catalog_query(
    entries: &[DatasetCatalogEntry],
    query: &DatasetCatalogQuery,
) -> Vec<DatasetCatalogEntry> {
    entries
        .iter()
        .filter(|entry| query.schema_ref.as_ref().is_none_or(|value| &entry.schema_ref == value))
        .filter(|entry| query.owner.as_ref().is_none_or(|value| &entry.owner == value))
        .filter(|entry| {
            query.freshness_max_minutes.is_none_or(|value| entry.freshness_minutes <= value)
        })
        .filter(|entry| {
            query.quality_state.as_ref().is_none_or(|value| &entry.quality_state == value)
        })
        .cloned()
        .collect()
}

pub fn build_dataset_provenance_report(
    dataset_id: DatasetId,
    lineage: &DatasetLineageRecord,
    quality_samples: &[DatasetQualityState],
    promotion_count: usize,
) -> DatasetProvenanceReport {
    let mut pass_count = 0usize;
    for quality in quality_samples {
        if quality.acceptance_state == "accepted" {
            pass_count += 1;
        }
    }
    let validation_pass_rate = if quality_samples.is_empty() {
        0.0
    } else {
        pass_count as f64 / quality_samples.len() as f64
    };

    DatasetProvenanceReport {
        dataset_id,
        producer_count: lineage.producer_runs.len(),
        consumer_count: lineage.consumer_runs.len(),
        validation_pass_rate,
        promotions: promotion_count,
    }
}

pub fn default_dataset_example_workflow() -> DatasetPublicationWorkflow {
    DatasetPublicationWorkflow {
        dataset_id: DatasetId("sales-mart".to_string()),
        publication_steps: vec![
            "materialize-partitions".to_string(),
            "validate-quality-contract".to_string(),
            "publish-dataset-version".to_string(),
            "promote-latest-approved".to_string(),
        ],
        separates_artifact_materialization: true,
    }
}

pub fn dataset_mapping_index(
    mappings: &[DatasetArtifactMapping],
) -> BTreeMap<(DatasetId, DatasetVersionId), Vec<String>> {
    let mut index = BTreeMap::new();
    for mapping in mappings {
        index.insert(
            (mapping.dataset_id.clone(), mapping.version_id.clone()),
            mapping.artifact_ids.clone(),
        );
    }
    index
}
