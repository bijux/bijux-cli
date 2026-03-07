use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BinaryProvenanceRecord {
    pub component: String,
    pub version: String,
    pub build_id: String,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PluginProvenanceRecord {
    pub plugin_name: String,
    pub plugin_version: String,
    pub source: String,
    pub trust_tier: String,
    pub approval_state: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedArtifactManifest {
    pub artifact_id: String,
    pub signature_algorithm: String,
    pub signer_identity: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunProvenanceAttestation {
    pub run_id: String,
    pub dag_snapshot_id: String,
    pub plan_fingerprint: String,
    pub policy_bundle_version: String,
    pub binary_build_ids: Vec<String>,
    pub output_artifact_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EnvironmentAttestation {
    pub run_id: String,
    pub execution_backend: String,
    pub capability_class: String,
    pub trust_domain: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ComplianceEvidenceBundle {
    pub bundle_id: String,
    pub run_id: String,
    pub artifacts: Vec<String>,
    pub attestations: Vec<String>,
    pub immutable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenancePolicyGate {
    pub require_run_attestation: bool,
    pub require_environment_attestation: bool,
    pub require_signed_artifacts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationVerificationResult {
    pub verified: bool,
    pub reasons: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceExport {
    pub bundle_id: String,
    pub export_profile: String,
    pub immutable_hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SoftwareBillOfMaterialsHook {
    pub component: String,
    pub sbom_format: String,
    pub generated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceAwareReplayCheck {
    pub run_id: String,
    pub changed_inputs: Vec<String>,
    pub warning: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactTrustLabel {
    Unverified,
    Verified,
    Attested,
    Approved,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PromotionTrustPolicy {
    pub minimum_required_label: ArtifactTrustLabel,
    pub require_provenance_completeness: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceDriftReport {
    pub baseline_id: String,
    pub current_id: String,
    pub drifts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AttestationFormatCompatibility {
    pub source_format: String,
    pub target_format: String,
    pub compatible: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SupplyChainTrustMaturityMatrix {
    pub local_dev: u8,
    pub ci: u8,
    pub staging: u8,
    pub high_assurance_production: u8,
}

pub fn verify_attestations(
    run_attestation: Option<&RunProvenanceAttestation>,
    environment_attestation: Option<&EnvironmentAttestation>,
    signed_artifacts: &[SignedArtifactManifest],
    gate: &ProvenancePolicyGate,
) -> AttestationVerificationResult {
    let mut reasons = Vec::new();
    if gate.require_run_attestation && run_attestation.is_none() {
        reasons.push("run attestation missing".to_string());
    }
    if gate.require_environment_attestation && environment_attestation.is_none() {
        reasons.push("environment attestation missing".to_string());
    }
    if gate.require_signed_artifacts && signed_artifacts.is_empty() {
        reasons.push("signed artifacts missing".to_string());
    }
    AttestationVerificationResult {
        verified: reasons.is_empty(),
        reasons,
    }
}

pub fn provenance_complete_for_promotion(
    trust_label: &ArtifactTrustLabel,
    policy: &PromotionTrustPolicy,
    attestation_verified: bool,
) -> bool {
    let label_rank = |label: &ArtifactTrustLabel| match label {
        ArtifactTrustLabel::Unverified => 0,
        ArtifactTrustLabel::Verified => 1,
        ArtifactTrustLabel::Attested => 2,
        ArtifactTrustLabel::Approved => 3,
    };
    let meets_label = label_rank(trust_label) >= label_rank(&policy.minimum_required_label);
    let meets_attestation = !policy.require_provenance_completeness || attestation_verified;
    meets_label && meets_attestation
}

pub fn replay_provenance_warning(changed_inputs: &[String]) -> Option<String> {
    if changed_inputs.is_empty() {
        None
    } else {
        Some(format!(
            "replay trust inputs changed: {}",
            changed_inputs.join(",")
        ))
    }
}

pub fn provenance_drift(
    baseline_binaries: &[BinaryProvenanceRecord],
    current_binaries: &[BinaryProvenanceRecord],
    baseline_plugins: &[PluginProvenanceRecord],
    current_plugins: &[PluginProvenanceRecord],
    baseline_id: &str,
    current_id: &str,
) -> ProvenanceDriftReport {
    let mut drifts = BTreeSet::new();
    let base_binary: BTreeMap<_, _> = baseline_binaries
        .iter()
        .map(|b| (b.component.clone(), b.sha256.clone()))
        .collect();
    for cur in current_binaries {
        match base_binary.get(&cur.component) {
            Some(sha) if sha != &cur.sha256 => {
                drifts.insert(format!("binary drift in {}", cur.component));
            }
            None => {
                drifts.insert(format!("new binary component {}", cur.component));
            }
            _ => {}
        }
    }
    let base_plugin: BTreeMap<_, _> = baseline_plugins
        .iter()
        .map(|p| (p.plugin_name.clone(), p.plugin_version.clone()))
        .collect();
    for cur in current_plugins {
        match base_plugin.get(&cur.plugin_name) {
            Some(v) if v != &cur.plugin_version => {
                drifts.insert(format!("plugin version drift in {}", cur.plugin_name));
            }
            None => {
                drifts.insert(format!("new plugin {}", cur.plugin_name));
            }
            _ => {}
        }
    }
    ProvenanceDriftReport {
        baseline_id: baseline_id.to_string(),
        current_id: current_id.to_string(),
        drifts: drifts.into_iter().collect(),
    }
}

pub fn compatibility_of_attestation_format(
    source: &str,
    target: &str,
    supported_pairs: &[(&str, &str)],
) -> AttestationFormatCompatibility {
    let compatible = supported_pairs
        .iter()
        .any(|(s, t)| *s == source && *t == target);
    AttestationFormatCompatibility {
        source_format: source.to_string(),
        target_format: target.to_string(),
        compatible,
    }
}

pub fn summarize_trust_labels(labels: &[ArtifactTrustLabel]) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for label in labels {
        *map.entry(format!("{:?}", label)).or_insert(0) += 1;
    }
    map
}
