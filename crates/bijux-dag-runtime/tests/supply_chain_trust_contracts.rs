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
    build_provenance_drift_report, can_promote_artifact, evaluate_attestation_compatibility,
    regulated_workflow_reference_example, replay_trust_warnings, require_provenance_completeness,
    ArtifactTrustLabel, AttestationCompatibility, AttestationFormatRule, BinaryComponent,
    PromotionPolicy, ProvenanceCompletenessPolicy, RunProvenanceAttestation,
    SignedArtifactManifest,
};
use std::collections::{BTreeMap, BTreeSet};

mod support;

#[test]
fn completeness_policy_requires_expected_material() {
    let attestation: RunProvenanceAttestation = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/provenance/run_attestation.json",
    );
    let signed: Vec<SignedArtifactManifest> = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/provenance/signed_artifacts.json",
    );
    let policy = ProvenanceCompletenessPolicy {
        require_binary_provenance: true,
        require_plugin_provenance: true,
        require_environment_attestation: true,
        require_signed_artifacts: true,
    };

    let result = require_provenance_completeness(&attestation, &signed, &policy);
    assert!(result.passed);
    assert!(result.errors.is_empty());
}

#[test]
fn promotion_policy_enforces_label_and_completeness() {
    let policy = PromotionPolicy {
        allowed_labels: BTreeSet::from([
            ArtifactTrustLabel::Attested,
            ArtifactTrustLabel::Approved,
        ]),
        require_completeness: true,
    };

    assert!(can_promote_artifact(&ArtifactTrustLabel::Approved, true, &policy));
    assert!(!can_promote_artifact(&ArtifactTrustLabel::Verified, true, &policy));
    assert!(!can_promote_artifact(&ArtifactTrustLabel::Attested, false, &policy));
}

#[test]
fn compatibility_rule_lookup_is_deterministic() {
    let rules = vec![AttestationFormatRule {
        current_format: "attestation-v1".to_string(),
        candidate_format: "attestation-v2".to_string(),
        compatibility: AttestationCompatibility::CompatibleWithUpgrade,
    }];

    let result = evaluate_attestation_compatibility("attestation-v1", "attestation-v2", &rules);
    assert_eq!(result, AttestationCompatibility::CompatibleWithUpgrade);

    let missing = evaluate_attestation_compatibility("attestation-v2", "attestation-v3", &rules);
    assert_eq!(missing, AttestationCompatibility::Incompatible);
}

#[test]
fn drift_report_captures_binary_and_plugin_changes() {
    let baseline_builds = BTreeMap::from([
        (BinaryComponent::Scheduler, "build-a".to_string()),
        (BinaryComponent::Worker, "build-b".to_string()),
    ]);
    let current_builds = BTreeMap::from([
        (BinaryComponent::Scheduler, "build-a".to_string()),
        (BinaryComponent::Worker, "build-c".to_string()),
    ]);

    let baseline_plugins = BTreeMap::from([("object-store".to_string(), "1.0.0".to_string())]);
    let current_plugins = BTreeMap::from([("object-store".to_string(), "1.1.0".to_string())]);

    let report = build_provenance_drift_report(
        baseline_builds,
        current_builds,
        baseline_plugins,
        current_plugins,
    );

    assert_eq!(report.drifted_components, vec!["Worker"]);
    assert_eq!(report.drifted_plugins, vec!["object-store"]);
}

#[test]
fn replay_warnings_surface_trust_input_changes() {
    let baseline: RunProvenanceAttestation = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/provenance/run_attestation.json",
    );
    let mut candidate: RunProvenanceAttestation = support::load_workspace_fixture_typed(
        env!("CARGO_MANIFEST_DIR"),
        "crates/bijux-dag-runtime/tests/fixtures/provenance/run_attestation.json",
    );
    candidate.policy_bundle_version = "policy-v1.4.3".to_string();
    candidate.environment.trust_domain = "prod-eu".to_string();

    let warnings = replay_trust_warnings("run-2026-03-06-001", &baseline, &candidate);
    assert_eq!(warnings.run_id, "run-2026-03-06-001");
    assert_eq!(warnings.warnings.len(), 2);
}

#[test]
fn regulated_reference_requires_signed_attested_release_path() {
    let reference = regulated_workflow_reference_example();
    assert!(reference.requires_signed_artifacts);
    assert!(reference.requires_attested_promotion);
    assert!(reference.required_labels.contains(&ArtifactTrustLabel::Attested));
    assert!(reference.required_labels.contains(&ArtifactTrustLabel::Approved));
}
