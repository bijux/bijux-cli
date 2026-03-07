use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryComponent {
    Scheduler,
    Worker,
    Cli,
    Plugin(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BinaryProvenanceRecord {
    pub component: BinaryComponent,
    pub version: String,
    pub build_id: String,
    pub source_revision: String,
    pub build_timestamp_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PluginTrustTier {
    Untrusted,
    Community,
    Official,
    HighAssurance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PluginProvenanceRecord {
    pub plugin_name: String,
    pub version: String,
    pub source: String,
    pub trust_tier: PluginTrustTier,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SignedArtifactManifest {
    pub artifact_id: String,
    pub digest: String,
    pub signer_identity: String,
    pub signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvironmentAttestation {
    pub backend: String,
    pub capability_class: String,
    pub trust_domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RunProvenanceAttestation {
    pub run_id: String,
    pub dag_snapshot_id: String,
    pub plan_fingerprint: String,
    pub policy_bundle_version: String,
    pub output_digests: Vec<String>,
    pub binaries: Vec<BinaryProvenanceRecord>,
    pub plugins: Vec<PluginProvenanceRecord>,
    pub environment: EnvironmentAttestation,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ArtifactTrustLabel {
    Unverified,
    Verified,
    Attested,
    Approved,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComplianceEvidenceBundle {
    pub bundle_id: String,
    pub run_id: String,
    pub run_attestation: RunProvenanceAttestation,
    pub signed_manifests: Vec<SignedArtifactManifest>,
    pub generated_at_utc: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceCompletenessPolicy {
    pub require_binary_provenance: bool,
    pub require_plugin_provenance: bool,
    pub require_environment_attestation: bool,
    pub require_signed_artifacts: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PromotionPolicy {
    pub allowed_labels: BTreeSet<ArtifactTrustLabel>,
    pub require_completeness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AttestationCompatibility {
    Compatible,
    CompatibleWithUpgrade,
    MigrationRequired,
    Incompatible,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttestationFormatRule {
    pub current_format: String,
    pub candidate_format: String,
    pub compatibility: AttestationCompatibility,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProvenanceDriftReport {
    pub baseline_build_ids: BTreeMap<BinaryComponent, String>,
    pub current_build_ids: BTreeMap<BinaryComponent, String>,
    pub baseline_plugin_versions: BTreeMap<String, String>,
    pub current_plugin_versions: BTreeMap<String, String>,
    pub drifted_components: Vec<String>,
    pub drifted_plugins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayTrustWarning {
    pub run_id: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttestationVerificationResult {
    pub passed: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RegulatedWorkflowReference {
    pub name: String,
    pub requires_signed_artifacts: bool,
    pub requires_attested_promotion: bool,
    pub required_labels: BTreeSet<ArtifactTrustLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SupplyChainMaturityMatrix {
    pub local_dev: String,
    pub shared_dev: String,
    pub staging: String,
    pub production: String,
    pub high_assurance: String,
}

pub fn require_provenance_completeness(
    attestation: &RunProvenanceAttestation,
    manifests: &[SignedArtifactManifest],
    policy: &ProvenanceCompletenessPolicy,
) -> AttestationVerificationResult {
    let mut errors = Vec::new();

    if policy.require_binary_provenance && attestation.binaries.is_empty() {
        errors.push("binary provenance is required".to_string());
    }
    if policy.require_plugin_provenance && attestation.plugins.is_empty() {
        errors.push("plugin provenance is required".to_string());
    }
    if policy.require_environment_attestation
        && (attestation.environment.backend.is_empty()
            || attestation.environment.capability_class.is_empty()
            || attestation.environment.trust_domain.is_empty())
    {
        errors.push("environment attestation is incomplete".to_string());
    }
    if policy.require_signed_artifacts && manifests.is_empty() {
        errors.push("signed artifacts are required".to_string());
    }

    AttestationVerificationResult {
        passed: errors.is_empty(),
        errors,
    }
}

pub fn verify_attestation_or_fail(result: AttestationVerificationResult) -> Result<(), String> {
    if result.passed {
        Ok(())
    } else {
        Err(format!(
            "attestation verification failed: {}",
            result.errors.join("; ")
        ))
    }
}

pub fn can_promote_artifact(
    label: &ArtifactTrustLabel,
    completeness_ok: bool,
    policy: &PromotionPolicy,
) -> bool {
    if policy.require_completeness && !completeness_ok {
        return false;
    }
    policy.allowed_labels.contains(label)
}

pub fn evaluate_attestation_compatibility(
    current_format: &str,
    candidate_format: &str,
    rules: &[AttestationFormatRule],
) -> AttestationCompatibility {
    rules.iter()
        .find(|rule| {
            rule.current_format == current_format && rule.candidate_format == candidate_format
        })
        .map(|rule| rule.compatibility.clone())
        .unwrap_or(AttestationCompatibility::Incompatible)
}

pub fn build_provenance_drift_report(
    baseline_build_ids: BTreeMap<BinaryComponent, String>,
    current_build_ids: BTreeMap<BinaryComponent, String>,
    baseline_plugin_versions: BTreeMap<String, String>,
    current_plugin_versions: BTreeMap<String, String>,
) -> ProvenanceDriftReport {
    let mut drifted_components = Vec::new();
    for (component, baseline) in &baseline_build_ids {
        match current_build_ids.get(component) {
            Some(current) if current == baseline => {}
            _ => drifted_components.push(format!("{component:?}")),
        }
    }

    let mut drifted_plugins = Vec::new();
    for (name, baseline) in &baseline_plugin_versions {
        match current_plugin_versions.get(name) {
            Some(current) if current == baseline => {}
            _ => drifted_plugins.push(name.clone()),
        }
    }

    ProvenanceDriftReport {
        baseline_build_ids,
        current_build_ids,
        baseline_plugin_versions,
        current_plugin_versions,
        drifted_components,
        drifted_plugins,
    }
}

pub fn replay_trust_warnings(
    run_id: &str,
    baseline: &RunProvenanceAttestation,
    candidate: &RunProvenanceAttestation,
) -> ReplayTrustWarning {
    let mut warnings = Vec::new();
    if baseline.policy_bundle_version != candidate.policy_bundle_version {
        warnings.push("policy bundle version changed".to_string());
    }
    if baseline.environment.trust_domain != candidate.environment.trust_domain {
        warnings.push("trust domain changed".to_string());
    }
    if baseline.binaries != candidate.binaries {
        warnings.push("binary provenance changed".to_string());
    }
    if baseline.plugins != candidate.plugins {
        warnings.push("plugin provenance changed".to_string());
    }
    ReplayTrustWarning {
        run_id: run_id.to_string(),
        warnings,
    }
}

pub fn regulated_workflow_reference_example() -> RegulatedWorkflowReference {
    RegulatedWorkflowReference {
        name: "regulated-release".to_string(),
        requires_signed_artifacts: true,
        requires_attested_promotion: true,
        required_labels: BTreeSet::from([ArtifactTrustLabel::Attested, ArtifactTrustLabel::Approved]),
    }
}

pub fn default_supply_chain_maturity_matrix() -> SupplyChainMaturityMatrix {
    SupplyChainMaturityMatrix {
        local_dev: "unsigned local artifacts; provenance optional".to_string(),
        shared_dev: "binary provenance required; plugin provenance recommended".to_string(),
        staging: "binary/plugin provenance and attestation verification required".to_string(),
        production: "signed artifacts and promotion trust labels enforced".to_string(),
        high_assurance: "full attestations, signed artifacts, and approval gates required".to_string(),
    }
}
