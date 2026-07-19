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
    build_consistency_catalog, classify_resource_consistency, default_split_brain_mitigation,
    geo_ready, region_write_allowed, ConsistencyBoundaryNote, ConsistencyClass,
    GeoReadyAcceptanceGate, RegionId, WriteRoutingRule,
};
use std::collections::BTreeSet;

fn load_consistency_boundaries() -> Vec<ConsistencyBoundaryNote> {
    let raw = std::fs::read_to_string("tests/fixtures/geo/consistency_boundaries.json")
        .expect("geo consistency fixture");
    serde_json::from_str(&raw).expect("valid geo consistency fixture")
}

#[test]
fn region_write_routing_allows_only_configured_regions() {
    let rule = WriteRoutingRule {
        resource: "registry-entry".to_string(),
        global_visible: true,
        write_regions: BTreeSet::from([
            RegionId("eu-north".to_string()),
            RegionId("us-east".to_string()),
        ]),
    };

    assert!(region_write_allowed(&rule, &RegionId("eu-north".to_string())));
    assert!(!region_write_allowed(&rule, &RegionId("ap-south".to_string())));
}

#[test]
fn consistency_classification_defaults_to_eventual() {
    let boundaries = load_consistency_boundaries();
    let registry = classify_resource_consistency("dag-registry", &boundaries);
    let unknown = classify_resource_consistency("unknown-resource", &boundaries);

    assert_eq!(registry, ConsistencyClass::RegionallyConsistent);
    assert_eq!(unknown, ConsistencyClass::EventuallyReplicated);
}

#[test]
fn geo_ready_gate_requires_all_domains() {
    let gate = GeoReadyAcceptanceGate {
        registry_ready: true,
        scheduler_ready: true,
        lineage_ready: true,
        observability_ready: true,
    };
    assert!(geo_ready(&gate));

    let blocked = GeoReadyAcceptanceGate { observability_ready: false, ..gate };
    assert!(!geo_ready(&blocked));
}

#[test]
fn split_brain_mitigation_is_fencing_first() {
    let mitigation = default_split_brain_mitigation();
    assert!(mitigation.fencing_required);
    assert!(mitigation.detection_signals.iter().any(|signal| signal.contains("dual-leader")));
}

#[test]
fn builds_consistency_catalog_for_reporting() {
    let boundaries = load_consistency_boundaries();
    let catalog = build_consistency_catalog(&boundaries);
    assert_eq!(catalog.len(), 3);
    assert_eq!(catalog.get("schedule-leases"), Some(&ConsistencyClass::StronglyConsistent));
}
