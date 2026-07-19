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
    adoption_score, conformance_passes, integration_governance_ready, packaging_ready,
    release_note_summary, upgrade_bundle_valid, DeploymentConformanceResult,
    IntegrationGovernanceRule, PackagingMode, PackagingStrategy, PlatformAdoptionScorecard,
    ReleaseNoteRecord, UpgradeBundle,
};
use std::collections::BTreeSet;

fn load_conformance() -> Vec<DeploymentConformanceResult> {
    let raw = std::fs::read_to_string("tests/fixtures/packaging/deployment_conformance.json")
        .expect("packaging conformance fixture");
    serde_json::from_str(&raw).expect("valid packaging conformance fixture")
}

#[test]
fn packaging_strategy_requires_signed_verified_official_modes() {
    let strategy = PackagingStrategy {
        official_modes: BTreeSet::from([
            PackagingMode::StandaloneBinary,
            PackagingMode::ContainerImage,
        ]),
        signing_required: true,
        verification_required: true,
    };
    assert!(packaging_ready(&strategy));
}

#[test]
fn upgrade_bundle_requires_migration_compatibility_and_rollback_material() {
    let bundle = UpgradeBundle {
        from_version: "1.9".to_string(),
        to_version: "2.0".to_string(),
        includes_migration_checks: true,
        includes_compatibility_report: true,
        includes_rollback_guidance: true,
    };
    assert!(upgrade_bundle_valid(&bundle));
}

#[test]
fn deployment_conformance_passes_only_when_all_modes_pass() {
    let conformance = load_conformance();
    assert!(conformance_passes(&conformance));

    let mut broken = conformance.clone();
    broken[1].passed = false;
    assert!(!conformance_passes(&broken));
}

#[test]
fn release_note_summary_is_machine_readable() {
    let note = ReleaseNoteRecord {
        release: "2.0.0".to_string(),
        contract_changes: vec!["new-api-contract".to_string()],
        migration_impacts: vec!["state-migration-required".to_string()],
        maturity_movements: vec!["scheduler->stable".to_string()],
    };
    let summary = release_note_summary(&note);
    assert!(summary.contains("release=2.0.0"));
}

#[test]
fn integration_governance_and_adoption_score_are_reported() {
    let governance = IntegrationGovernanceRule {
        contribution_requirements: vec!["signed-cla".to_string()],
        integration_qualification_requirements: vec!["contract-tests".to_string()],
        official_adoption_requirements: vec!["support-review".to_string()],
    };
    assert!(integration_governance_ready(&governance));

    let scorecard = PlatformAdoptionScorecard {
        installability_score: 85,
        operability_score: 80,
        integration_breadth_score: 78,
        guarantees_clarity_score: 88,
    };
    assert_eq!(adoption_score(&scorecard), 82.75);
}
