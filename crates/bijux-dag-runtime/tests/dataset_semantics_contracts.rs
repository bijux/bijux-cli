use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

use bijux_dag_runtime::simulated_platform::{
    build_dataset_provenance_report, dataset_catalog_query, dataset_consumption_satisfied,
    dataset_diff, dataset_mapping_index, dataset_ready_for_schedule,
    default_dataset_example_workflow, DatasetArtifactMapping, DatasetCatalogEntry,
    DatasetCatalogQuery, DatasetConsumptionContract, DatasetConsumptionMode, DatasetId,
    DatasetLineageRecord, DatasetQualityState, DatasetReadinessGate, DatasetVersionId,
};
use std::collections::BTreeSet;

fn load_catalog() -> Vec<DatasetCatalogEntry> {
    let raw = std::fs::read_to_string("tests/fixtures/datasets/catalog_entries.json")
        .expect("dataset catalog fixture");
    serde_json::from_str(&raw).expect("valid dataset catalog fixture")
}

#[test]
fn consumption_contract_supports_stable_latest_and_freshness_modes() {
    let available = DatasetVersionId("v2026-03-07".to_string());
    let approved = DatasetVersionId("v2026-03-07".to_string());

    let stable = DatasetConsumptionContract {
        dataset_id: DatasetId("sales-mart".to_string()),
        mode: DatasetConsumptionMode::StableVersion(available.clone()),
    };
    assert!(dataset_consumption_satisfied(&stable, &available, &approved, 30));

    let latest = DatasetConsumptionContract {
        dataset_id: DatasetId("sales-mart".to_string()),
        mode: DatasetConsumptionMode::LatestApproved,
    };
    assert!(dataset_consumption_satisfied(&latest, &available, &approved, 30));

    let freshness = DatasetConsumptionContract {
        dataset_id: DatasetId("sales-mart".to_string()),
        mode: DatasetConsumptionMode::FreshnessBounded(60),
    };
    assert!(dataset_consumption_satisfied(&freshness, &available, &approved, 30));
    assert!(!dataset_consumption_satisfied(&freshness, &available, &approved, 90));
}

#[test]
fn readiness_gate_blocks_schedule_when_required_dataset_not_accepted() {
    let gates = vec![
        DatasetReadinessGate {
            dataset_id: DatasetId("sales-mart".to_string()),
            accepted: true,
            required_for_schedule: true,
        },
        DatasetReadinessGate {
            dataset_id: DatasetId("risk-features".to_string()),
            accepted: false,
            required_for_schedule: true,
        },
    ];
    assert!(!dataset_ready_for_schedule(&gates));
}

#[test]
fn dataset_diff_reports_schema_compatibility() {
    let diff = dataset_diff(
        &DatasetVersionId("v1".to_string()),
        &DatasetVersionId("v2".to_string()),
        "schema/sales/v1",
        "schema/sales/v2",
    );
    assert_eq!(diff.compatibility, "migration-required");
    assert_eq!(diff.differences.len(), 1);
}

#[test]
fn catalog_query_filters_by_owner_freshness_and_quality() {
    let entries = load_catalog();
    let query = DatasetCatalogQuery {
        schema_ref: Some("schema/sales/v1".to_string()),
        owner: Some("analytics".to_string()),
        freshness_max_minutes: Some(60),
        quality_state: Some("accepted".to_string()),
    };

    let filtered = dataset_catalog_query(&entries, &query);
    assert_eq!(filtered.len(), 1);
    assert_eq!(filtered[0].version_id, DatasetVersionId("v2026-03-07".to_string()));
}

#[test]
fn provenance_report_and_mapping_index_are_deterministic() {
    let lineage = DatasetLineageRecord {
        dataset_id: DatasetId("sales-mart".to_string()),
        producer_runs: BTreeSet::from(["run-a".to_string(), "run-b".to_string()]),
        consumer_runs: BTreeSet::from(["run-c".to_string()]),
    };
    let quality = vec![
        DatasetQualityState {
            validation_outcomes: vec!["not-null".to_string()],
            quality_score: 0.99,
            acceptance_state: "accepted".to_string(),
        },
        DatasetQualityState {
            validation_outcomes: vec!["range".to_string()],
            quality_score: 0.91,
            acceptance_state: "accepted".to_string(),
        },
    ];

    let report =
        build_dataset_provenance_report(DatasetId("sales-mart".to_string()), &lineage, &quality, 3);
    assert_eq!(report.producer_count, 2);
    assert_eq!(report.consumer_count, 1);
    assert_eq!(report.validation_pass_rate, 1.0);

    let mappings = vec![DatasetArtifactMapping {
        dataset_id: DatasetId("sales-mart".to_string()),
        version_id: DatasetVersionId("v2026-03-07".to_string()),
        artifact_ids: vec!["artifact-1".to_string(), "artifact-2".to_string()],
    }];

    let index = dataset_mapping_index(&mappings);
    assert_eq!(index.len(), 1);
    assert!(index.contains_key(&(
        DatasetId("sales-mart".to_string()),
        DatasetVersionId("v2026-03-07".to_string()),
    )));
}

#[test]
fn example_workflow_separates_publication_from_materialization() {
    let workflow = default_dataset_example_workflow();
    assert!(workflow.separates_artifact_materialization);
    assert_eq!(workflow.publication_steps.len(), 4);
}
