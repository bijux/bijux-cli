use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum VerificationMaturityLabel {
    Specified,
    PropertyTested,
    ModelTested,
    FormallyConstrained,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct InvariantDefinition {
    pub id: String,
    pub subsystem: String,
    pub statement: String,
    pub machine_checkable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerifiedCoreScope {
    pub subsystems: BTreeSet<String>,
    pub notes: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropertyTestSuite {
    pub name: String,
    pub target_subsystem: String,
    pub generators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModelTestSuite {
    pub name: String,
    pub state_machine: String,
    pub explored_states: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchedulerStateSpaceCheck {
    pub duplicate_run_prevention_proven: bool,
    pub fairness_guard_proven: bool,
    pub explored_paths: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LineageInvariantProof {
    pub replay_consistent: bool,
    pub retry_consistent: bool,
    pub import_consistent: bool,
    pub promotion_consistent: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PolicyInvariantProof {
    pub deny_never_bypassed: bool,
    pub fallback_paths_checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArtifactIntegrityInvariant {
    pub content_identity_immutable: bool,
    pub provenance_alignment_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayDeterminismInvariant {
    pub backend_classes_checked: BTreeSet<String>,
    pub deterministic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DiffSemanticSpec {
    pub spec_name: String,
    pub critical_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct HaVerificationHarness {
    pub failover_checked: bool,
    pub fencing_checked: bool,
    pub restart_idempotence_checked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CounterexampleReport {
    pub invariant_id: String,
    pub minimal_repro_steps: Vec<String>,
    pub observed_violation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationGate {
    pub invariant_suites_required: bool,
    pub property_suites_required: bool,
    pub model_suites_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FuzzingStrategy {
    pub parser_targets: Vec<String>,
    pub planner_targets: Vec<String>,
    pub scheduler_targets: Vec<String>,
    pub manifest_targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AdversarialFixtureSet {
    pub malformed_bundles: usize,
    pub lineage_cycles: usize,
    pub policy_corruption_cases: usize,
    pub split_brain_timing_cases: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FormalAssuranceRoadmap {
    pub near_term: Vec<String>,
    pub mid_term: Vec<String>,
    pub long_term: Vec<String>,
}

pub fn invariant_catalog_default() -> Vec<InvariantDefinition> {
    vec![
        InvariantDefinition {
            id: "dag-compile-determinism".to_string(),
            subsystem: "planner".to_string(),
            statement: "identical DAG and policy inputs yield identical plan fingerprint".to_string(),
            machine_checkable: true,
        },
        InvariantDefinition {
            id: "schedule-idempotence".to_string(),
            subsystem: "scheduler".to_string(),
            statement: "replayed trigger evaluation cannot emit duplicate run records".to_string(),
            machine_checkable: true,
        },
        InvariantDefinition {
            id: "artifact-provenance-alignment".to_string(),
            subsystem: "artifacts".to_string(),
            statement: "artifact identity and provenance references remain coherent".to_string(),
            machine_checkable: true,
        },
    ]
}

pub fn verification_gate_passed(
    gate: &VerificationGate,
    invariant_ok: bool,
    property_ok: bool,
    model_ok: bool,
) -> bool {
    (!gate.invariant_suites_required || invariant_ok)
        && (!gate.property_suites_required || property_ok)
        && (!gate.model_suites_required || model_ok)
}

pub fn machine_checkable_invariants(
    invariants: &[InvariantDefinition],
) -> BTreeMap<String, bool> {
    invariants
        .iter()
        .map(|inv| (inv.id.clone(), inv.machine_checkable))
        .collect()
}

pub fn lineage_invariants_hold(proof: &LineageInvariantProof) -> bool {
    proof.replay_consistent
        && proof.retry_consistent
        && proof.import_consistent
        && proof.promotion_consistent
}

pub fn policy_invariants_hold(proof: &PolicyInvariantProof) -> bool {
    proof.deny_never_bypassed && proof.fallback_paths_checked
}

pub fn artifact_integrity_holds(invariant: &ArtifactIntegrityInvariant) -> bool {
    invariant.content_identity_immutable && invariant.provenance_alignment_verified
}

pub fn replay_determinism_holds(invariant: &ReplayDeterminismInvariant) -> bool {
    invariant.deterministic && !invariant.backend_classes_checked.is_empty()
}

pub fn build_counterexample(
    invariant_id: &str,
    observed_violation: &str,
    repro_steps: Vec<String>,
) -> CounterexampleReport {
    CounterexampleReport {
        invariant_id: invariant_id.to_string(),
        minimal_repro_steps: repro_steps,
        observed_violation: observed_violation.to_string(),
    }
}

pub fn verification_maturity_label(
    specified: bool,
    property_tested: bool,
    model_tested: bool,
    formally_constrained: bool,
) -> VerificationMaturityLabel {
    if formally_constrained {
        VerificationMaturityLabel::FormallyConstrained
    } else if model_tested {
        VerificationMaturityLabel::ModelTested
    } else if property_tested {
        VerificationMaturityLabel::PropertyTested
    } else if specified {
        VerificationMaturityLabel::Specified
    } else {
        VerificationMaturityLabel::Specified
    }
}
