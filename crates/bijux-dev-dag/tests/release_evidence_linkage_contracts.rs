use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn release_evidence_outputs_and_verify_surfaces_exist() {
    let root = repo_root();
    let release_set: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
            .expect("read release evidence set"),
    )
    .expect("parse release evidence set");
    let claimed = release_set["claimed_proof_surfaces"]
        .as_object()
        .expect("claimed_proof_surfaces object");
    for required in ["replay", "cache", "operator"] {
        let values = claimed
            .get(required)
            .and_then(serde_json::Value::as_array)
            .unwrap_or_else(|| panic!("missing claimed proof surface: {required}"));
        assert!(
            !values.is_empty(),
            "claimed proof surface cannot be empty: {required}"
        );
    }

    for rel in [
        "evidence/release/release_evidence_set.json",
        "evidence/release/release_evidence.json",
        "evidence/reports/what_this_release_proves.md",
        "evidence/reports/what_this_release_does_not_prove.md",
        "evidence/reports/unsupported_or_simulated_areas.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing release evidence output: {rel}"
        );
    }

    let command_source = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read commands source");
    for token in [
        "verify.evidence-release-set",
        "repo.release-evidence-report",
        "run_evidence_release_set_verify()",
        "run_release_evidence_report(",
    ] {
        assert!(
            command_source.contains(token),
            "missing release evidence control-plane surface: {token}"
        );
    }
}

#[test]
fn release_verify_enforces_manifest_drift_and_ambiguous_classification_failure() {
    let root = repo_root();
    let source = fs::read_to_string(
        root.join("crates/bijux-dev-dag/src/commands/evidence_control_plane.rs"),
    )
    .expect("read evidence control plane source");

    assert!(
        source.contains("release evidence manifest drift detected"),
        "release-set verify must fail on release manifest drift"
    );
    assert!(
        source.contains("ambiguous evidence classification"),
        "release-set verify must fail ambiguous blocking/advisory classification"
    );
    for token in [
        "must have one clear owner in registry",
        "must have at least one consumer suite",
        "must declare trust-property mapping",
        "claimed proof surface `operator` must reference operator-kind blocking asset",
    ] {
        assert!(
            source.contains(token),
            "release-set verify missing governance token: {token}"
        );
    }
}

#[test]
fn release_ci_and_release_notes_are_proof_centered() {
    let root = repo_root();
    let workflow = fs::read_to_string(root.join(".github/workflows/release-verify.yml"))
        .expect("read release verify workflow");
    for token in [
        "verify release evidence set",
        "generate release proof reports",
        "release-proof-report",
    ] {
        assert!(
            workflow.contains(token),
            "release CI must remain centered on release proof token: {token}"
        );
    }

    let release_template = fs::read_to_string(root.join("docs/reference/RELEASE_NOTE_TEMPLATE.md"))
        .expect("read release note template");
    for token in [
        "what this release proves",
        "what this release does not prove",
        "unsupported or simulated areas",
    ] {
        assert!(
            release_template.contains(token),
            "release notes template missing evidence link token: {token}"
        );
    }
}
