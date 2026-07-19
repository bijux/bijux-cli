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
    artifact_integrity_holds, build_counterexample, invariant_catalog_default,
    lineage_invariants_hold, machine_checkable_invariants, policy_invariants_hold,
    replay_determinism_holds, verification_gate_passed, verification_maturity_label,
    ArtifactIntegrityInvariant, InvariantDefinition, LineageInvariantProof, PolicyInvariantProof,
    ReplayDeterminismInvariant, VerificationGate, VerificationMaturityLabel,
};
use std::collections::BTreeSet;

fn load_invariants() -> Vec<InvariantDefinition> {
    let raw = std::fs::read_to_string("tests/fixtures/verification/invariants.json")
        .expect("verification invariant fixture");
    serde_json::from_str(&raw).expect("valid invariant fixture")
}

#[test]
fn invariant_catalog_and_machine_checkable_map_are_stable() {
    let defaults = invariant_catalog_default();
    assert!(!defaults.is_empty());

    let loaded = load_invariants();
    let map = machine_checkable_invariants(&loaded);
    assert_eq!(map.len(), 3);
    assert_eq!(map.get("policy-deny-finality"), Some(&true));
}

#[test]
fn verification_gate_requires_configured_suites() {
    let gate = VerificationGate {
        invariant_suites_required: true,
        property_suites_required: true,
        model_suites_required: true,
    };

    assert!(verification_gate_passed(&gate, true, true, true));
    assert!(!verification_gate_passed(&gate, true, false, true));
}

#[test]
fn lineage_policy_artifact_and_replay_invariants_hold_when_flags_are_true() {
    let lineage = LineageInvariantProof {
        replay_consistent: true,
        retry_consistent: true,
        import_consistent: true,
        promotion_consistent: true,
    };
    assert!(lineage_invariants_hold(&lineage));

    let policy = PolicyInvariantProof { deny_never_bypassed: true, fallback_paths_checked: true };
    assert!(policy_invariants_hold(&policy));

    let artifact = ArtifactIntegrityInvariant {
        content_identity_immutable: true,
        provenance_alignment_verified: true,
    };
    assert!(artifact_integrity_holds(&artifact));

    let replay = ReplayDeterminismInvariant {
        backend_classes_checked: BTreeSet::from(["local".to_string(), "kubernetes".to_string()]),
        deterministic: true,
    };
    assert!(replay_determinism_holds(&replay));
}

#[test]
fn counterexample_and_maturity_label_are_reported() {
    let counterexample = build_counterexample(
        "schedule-idempotence",
        "duplicate run generated after restart",
        vec![
            "create trigger burst".to_string(),
            "force scheduler crash".to_string(),
            "restart and replay tick".to_string(),
        ],
    );
    assert_eq!(counterexample.invariant_id, "schedule-idempotence");
    assert_eq!(counterexample.minimal_repro_steps.len(), 3);

    let label = verification_maturity_label(true, true, true, false);
    assert_eq!(label, VerificationMaturityLabel::ModelTested);
}
