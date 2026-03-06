use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CompatibilityClass {
    FullyCompatible,
    ReplayCompatible,
    ReadOnlyCompatible,
    MigrationRequired,
    Breaking,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityPolicy {
    pub dag_spec: CompatibilityClass,
    pub run_manifest: CompatibilityClass,
    pub artifact_manifest: CompatibilityClass,
    pub api_contract: CompatibilityClass,
    pub plugin_interface: CompatibilityClass,
    pub scheduler_state: CompatibilityClass,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityRule {
    pub surface: String,
    pub from_version: String,
    pub to_version: String,
    pub class: CompatibilityClass,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ManifestMigrationPlan {
    pub surface: String,
    pub from_version: String,
    pub to_version: String,
    pub transform_steps: Vec<String>,
    pub post_migration_checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DurableStateMigrationContract {
    pub scheduler_store: String,
    pub registry_store: String,
    pub requires_backup: bool,
    pub requires_lock: bool,
    pub rollback_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginVersionWindow {
    pub interface: String,
    pub min_supported: String,
    pub max_supported: String,
    pub deprecation_deadline_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CrossVersionMatrixRow {
    pub reader_version: String,
    pub producer_version: String,
    pub can_read_runs: bool,
    pub can_read_artifacts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DowngradeRiskReport {
    pub target_binary_version: String,
    pub incompatible_surfaces: Vec<String>,
    pub blocking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureLifecycleState {
    Experimental,
    Preview,
    Stable,
    Deprecated,
    Removed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeatureFlagRecord {
    pub name: String,
    pub state: FeatureLifecycleState,
    pub owner: String,
    pub removal_target_release: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeprecationDiagnostic {
    pub surface: String,
    pub item: String,
    pub replacement: Option<String>,
    pub remove_after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MigrationImpactEstimate {
    pub affected_runs: usize,
    pub affected_artifacts: usize,
    pub requires_downtime: bool,
    pub estimated_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpgradeRolloutPlan {
    pub control_plane_strategy: String,
    pub worker_strategy: String,
    pub canary_steps: Vec<String>,
    pub verification_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerStateCompatibilityCheck {
    pub from_state_version: String,
    pub to_state_version: String,
    pub ha_ready: bool,
    pub shard_ready: bool,
    pub required_migrations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpgradePathPolicy {
    pub supported_paths: BTreeSet<String>,
    pub unsupported_jumps: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityAcceptanceSuite {
    pub suite_name: String,
    pub required_for_release: bool,
    pub checks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseGateOutcome {
    pub passed: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LongTermSupportPolicy {
    pub core_spec_months: u32,
    pub api_months: u32,
    pub plugin_months: u32,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatibilityDashboard {
    pub policy: CompatibilityPolicy,
    pub rule_count: usize,
    pub features_by_state: BTreeMap<FeatureLifecycleState, usize>,
    pub suites_required: usize,
    pub downgrade_risk_blocking: bool,
}

pub fn classify_compatibility(rule: &CompatibilityRule) -> CompatibilityClass {
    rule.class.clone()
}

pub fn simulate_migration_impact(
    run_count: usize,
    artifact_count: usize,
    migration_steps: usize,
) -> MigrationImpactEstimate {
    MigrationImpactEstimate {
        affected_runs: run_count,
        affected_artifacts: artifact_count,
        requires_downtime: migration_steps > 3,
        estimated_minutes: (migration_steps as u32) * 15,
    }
}

pub fn validate_upgrade_path(policy: &UpgradePathPolicy, from: &str, to: &str) -> Result<(), String> {
    let key = format!("{from}->{to}");
    if policy.unsupported_jumps.contains(&key) {
        return Err(format!("unsupported upgrade jump: {key}"));
    }
    if policy.supported_paths.contains(&key) {
        return Ok(());
    }
    Err(format!("unknown upgrade path: {key}"))
}

pub fn evaluate_release_gate(
    unreviewed_breaking_changes: usize,
    acceptance_suites_passed: bool,
) -> ReleaseGateOutcome {
    let mut reasons = Vec::new();
    if unreviewed_breaking_changes > 0 {
        reasons.push("unreviewed breaking changes detected".to_string());
    }
    if !acceptance_suites_passed {
        reasons.push("required compatibility acceptance suites not passed".to_string());
    }
    ReleaseGateOutcome {
        passed: reasons.is_empty(),
        reasons,
    }
}

pub fn build_compatibility_dashboard(
    policy: CompatibilityPolicy,
    rules: &[CompatibilityRule],
    features: &[FeatureFlagRecord],
    suites: &[CompatibilityAcceptanceSuite],
    downgrade_risk: &DowngradeRiskReport,
) -> CompatibilityDashboard {
    let mut features_by_state: BTreeMap<FeatureLifecycleState, usize> = BTreeMap::new();
    for feature in features {
        *features_by_state.entry(feature.state.clone()).or_default() += 1;
    }

    CompatibilityDashboard {
        policy,
        rule_count: rules.len(),
        features_by_state,
        suites_required: suites.iter().filter(|suite| suite.required_for_release).count(),
        downgrade_risk_blocking: downgrade_risk.blocking,
    }
}
