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

use bijux_dag_runtime::{
    build_compatibility_dashboard, evaluate_release_gate, simulate_migration_impact,
    validate_upgrade_path, CompatibilityAcceptanceSuite, CompatibilityClass, CompatibilityPolicy,
    CompatibilityRule, DowngradeRiskReport, FeatureFlagRecord, FeatureLifecycleState,
    UpgradePathPolicy,
};

fn fixture_paths() -> UpgradePathPolicy {
    let raw = std::fs::read_to_string("tests/fixtures/compatibility/upgrade_paths.json")
        .expect("upgrade path fixture");
    serde_json::from_str(&raw).expect("valid upgrade path fixture")
}

#[test]
fn validates_supported_and_blocks_unsupported_upgrade_paths() {
    let policy = fixture_paths();
    assert!(validate_upgrade_path(&policy, "1.7", "1.8").is_ok());
    assert!(validate_upgrade_path(&policy, "1.7", "1.9").is_err());
}

#[test]
fn estimates_migration_impact_from_counts_and_steps() {
    let estimate = simulate_migration_impact(12_000, 42_000, 4);
    assert_eq!(estimate.affected_runs, 12_000);
    assert_eq!(estimate.affected_artifacts, 42_000);
    assert!(estimate.requires_downtime);
}

#[test]
fn release_gate_blocks_unreviewed_breaking_changes() {
    let outcome = evaluate_release_gate(2, true);
    assert!(!outcome.passed);
    assert_eq!(outcome.reasons.len(), 1);
}

#[test]
fn compatibility_dashboard_counts_states_and_required_suites() {
    let policy = CompatibilityPolicy {
        dag_spec: CompatibilityClass::ReplayCompatible,
        run_manifest: CompatibilityClass::FullyCompatible,
        artifact_manifest: CompatibilityClass::MigrationRequired,
        api_contract: CompatibilityClass::ReplayCompatible,
        plugin_interface: CompatibilityClass::ReadOnlyCompatible,
        scheduler_state: CompatibilityClass::MigrationRequired,
    };

    let rules = vec![CompatibilityRule {
        surface: "dag-spec".to_string(),
        from_version: "1.7".to_string(),
        to_version: "1.8".to_string(),
        class: CompatibilityClass::ReplayCompatible,
        notes: "deterministic replay preserved".to_string(),
    }];

    let features = vec![
        FeatureFlagRecord {
            name: "scheduler-epoch-fencing".to_string(),
            state: FeatureLifecycleState::Stable,
            owner: "platform".to_string(),
            removal_target_release: None,
        },
        FeatureFlagRecord {
            name: "api-migration-preview".to_string(),
            state: FeatureLifecycleState::Preview,
            owner: "control-plane".to_string(),
            removal_target_release: Some("2.0".to_string()),
        },
    ];

    let suites = vec![CompatibilityAcceptanceSuite {
        suite_name: "cross-version-read".to_string(),
        required_for_release: true,
        checks: vec!["run-manifest-read".to_string(), "artifact-manifest-read".to_string()],
    }];

    let risk = DowngradeRiskReport {
        target_binary_version: "1.7".to_string(),
        incompatible_surfaces: vec!["scheduler-state".to_string()],
        blocking: true,
    };

    let dashboard = build_compatibility_dashboard(policy, &rules, &features, &suites, &risk);
    assert_eq!(dashboard.rule_count, 1);
    assert_eq!(dashboard.suites_required, 1);
    assert!(dashboard.downgrade_risk_blocking);
}
