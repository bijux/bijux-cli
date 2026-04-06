use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PackagingMode {
    StandaloneBinary,
    ContainerImage,
    ReferenceDeploymentBundle,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PackagingStrategy {
    pub official_modes: BTreeSet<PackagingMode>,
    pub signing_required: bool,
    pub verification_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentProfileBundle {
    pub profile_name: String,
    pub target_mode: String,
    pub components: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ClusterDeploymentReference {
    pub flavor: String,
    pub manifests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VersionedCompatibilityMatrix {
    pub release: String,
    pub supported_backends: BTreeSet<String>,
    pub supported_stores: BTreeSet<String>,
    pub supported_auth_providers: BTreeSet<String>,
    pub supported_plugins: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationSupportPolicy {
    pub official_integrations: BTreeSet<String>,
    pub community_integrations: BTreeSet<String>,
    pub support_sla_by_tier: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SampleDeploymentCatalog {
    pub local: String,
    pub ha: String,
    pub multi_tenant: String,
    pub geo_federated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EcosystemCatalog {
    pub adapters: BTreeSet<String>,
    pub stores: BTreeSet<String>,
    pub executors: BTreeSet<String>,
    pub exporters: BTreeSet<String>,
    pub policy_bundles: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DistributionSignatureRecord {
    pub artifact_name: String,
    pub signature: String,
    pub signer: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UpgradeBundle {
    pub from_version: String,
    pub to_version: String,
    pub includes_migration_checks: bool,
    pub includes_compatibility_report: bool,
    pub includes_rollback_guidance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseTransparencyReport {
    pub benchmark_summary: String,
    pub capability_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InstallationDiagnostics {
    pub checks: Vec<String>,
    pub all_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeploymentConformanceResult {
    pub mode: String,
    pub passed: bool,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProductTierPolicy {
    pub tier_name: String,
    pub included_capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OnboardingGuideCatalog {
    pub dag_author_guide: String,
    pub tenant_admin_guide: String,
    pub platform_admin_guide: String,
    pub operator_guide: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityDiscoveryReport {
    pub active_capabilities: BTreeSet<String>,
    pub preview_capabilities: BTreeSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReleaseNoteRecord {
    pub release: String,
    pub contract_changes: Vec<String>,
    pub migration_impacts: Vec<String>,
    pub maturity_movements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IntegrationGovernanceRule {
    pub contribution_requirements: Vec<String>,
    pub integration_qualification_requirements: Vec<String>,
    pub official_adoption_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum StabilityClass {
    Stable,
    Preview,
    Experimental,
    Research,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StabilityMap {
    pub subsystem_stability: BTreeMap<String, StabilityClass>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReferenceEnvironmentVerification {
    pub environment_name: String,
    pub continuously_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PlatformAdoptionScorecard {
    pub installability_score: u8,
    pub operability_score: u8,
    pub integration_breadth_score: u8,
    pub guarantees_clarity_score: u8,
}

pub fn packaging_ready(strategy: &PackagingStrategy) -> bool {
    strategy.signing_required
        && strategy.verification_required
        && !strategy.official_modes.is_empty()
}

pub fn upgrade_bundle_valid(bundle: &UpgradeBundle) -> bool {
    bundle.includes_migration_checks
        && bundle.includes_compatibility_report
        && bundle.includes_rollback_guidance
}

pub fn conformance_passes(results: &[DeploymentConformanceResult]) -> bool {
    results.iter().all(|result| result.passed)
}

pub fn release_note_summary(record: &ReleaseNoteRecord) -> String {
    format!(
        "release={}, contracts={}, migrations={}, maturity={}",
        record.release,
        record.contract_changes.len(),
        record.migration_impacts.len(),
        record.maturity_movements.len()
    )
}

pub fn integration_governance_ready(rule: &IntegrationGovernanceRule) -> bool {
    !rule.contribution_requirements.is_empty()
        && !rule.integration_qualification_requirements.is_empty()
        && !rule.official_adoption_requirements.is_empty()
}

pub fn adoption_score(scorecard: &PlatformAdoptionScorecard) -> f64 {
    (scorecard.installability_score as f64
        + scorecard.operability_score as f64
        + scorecard.integration_breadth_score as f64
        + scorecard.guarantees_clarity_score as f64)
        / 4.0
}
