use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn verify_commands_include_replay_and_release_set() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev/src/commands/mod.rs"))
        .expect("read commands module");
    for token in [
        "EvidenceReplay",
        "EvidenceReleaseSet",
        "ReleaseEvidenceReport",
        "verify.evidence-replay",
        "verify.evidence-release-set",
        "run_evidence_replay_verify()",
        "run_evidence_release_set_verify()",
        "run_release_evidence_report(",
    ] {
        assert!(source.contains(token), "missing evidence control-plane token: {token}");
    }
}

#[test]
fn evidence_suite_policy_covers_required_verify_surface() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/evidence_suite_policy.json"))
            .expect("read evidence suite policy"),
    )
    .expect("parse policy");
    let suites = policy["suites"].as_array().expect("suites array");
    let ids: BTreeSet<String> =
        suites.iter().map(|entry| entry["id"].as_str().expect("suite id").to_string()).collect();
    for required in [
        "evidence-schema",
        "evidence-registry",
        "evidence-authoring",
        "evidence-battle",
        "evidence-cache",
        "evidence-replay",
        "evidence-compat",
        "evidence-fault",
        "evidence-perf",
        "evidence-compare",
        "evidence-consumers",
        "evidence-drift",
        "evidence-release-set",
        "evidence-foundation",
    ] {
        assert!(ids.contains(required), "missing evidence suite policy id: {required}");
    }
}

#[test]
fn release_evidence_set_assets_exist_and_are_registered() {
    let root = repo_root();
    let release_set: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/release/release_evidence_set.json"))
            .expect("read release evidence set"),
    )
    .expect("parse release evidence set");
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/_meta/registries/evidence_registry.json"))
            .expect("read evidence registry"),
    )
    .expect("parse registry");
    let ids: BTreeSet<String> = registry["assets"]
        .as_array()
        .expect("assets array")
        .iter()
        .map(|asset| asset["id"].as_str().expect("asset id").to_string())
        .collect();
    for field in ["blocking_assets", "advisory_assets"] {
        for entry in release_set[field].as_array().expect("release set array") {
            let id = entry.as_str().expect("release evidence asset id");
            assert!(root.join(id).exists(), "release evidence asset path is missing: {id}");
            assert!(ids.contains(id), "release evidence asset id is not present in registry: {id}");
        }
    }

    for field in ["required_families", "advisory_families"] {
        let entries = release_set[field].as_array().expect("release family list should be array");
        assert!(!entries.is_empty(), "release evidence family list cannot be empty: {field}");
        for entry in entries {
            assert!(
                entry.as_str().is_some(),
                "release evidence family entry must be a string in {field}"
            );
        }
    }

    let minimum_sets = release_set["minimum_blocking_sets"]
        .as_object()
        .expect("release minimum_blocking_sets should be object");
    assert!(!minimum_sets.is_empty(), "release minimum_blocking_sets cannot be empty");
    let blocking: BTreeSet<&str> = release_set["blocking_assets"]
        .as_array()
        .expect("blocking_assets array")
        .iter()
        .map(|entry| entry.as_str().expect("blocking asset id"))
        .collect();
    for (set_id, assets) in minimum_sets {
        let assets = assets.as_array().expect("minimum blocking set entry should be array");
        assert!(!assets.is_empty(), "minimum blocking set cannot be empty: {set_id}");
        for entry in assets {
            let id = entry.as_str().expect("minimum blocking set asset id");
            assert!(
                blocking.contains(id),
                "minimum blocking set `{set_id}` references non-blocking asset `{id}`"
            );
        }
    }
}

#[test]
fn evidence_suite_summary_models_exist() {
    let root = repo_root();
    for rel in [
        "configs/dag/schema/control_plane/evidence_suite_report.schema.json",
        "evidence/reports/evidence_verification_summary.md",
        "evidence/release/release_evidence.json",
        "evidence/reports/what_this_release_proves.md",
        "evidence/reports/what_this_release_does_not_prove.md",
        "evidence/reports/unsupported_or_simulated_areas.md",
    ] {
        assert!(root.join(rel).exists(), "missing evidence suite summary surface: {rel}");
    }

    let governance_workflow = root.join(".github/workflows/repository-governance.yml");
    assert!(
        governance_workflow.exists(),
        "missing evidence verify workflow surface; expected .github/workflows/repository-governance.yml"
    );
    let workflow = fs::read_to_string(&governance_workflow).expect("read governance workflow");
    for token in [
        "id: evidence-verify",
        "verify evidence-release-set",
        "verify evidence-battle",
        "verify evidence-cache",
        "verify evidence-replay",
        "verify evidence-consumers",
    ] {
        assert!(
            workflow.contains(token),
            "governance workflow is missing evidence verify token: {token}"
        );
    }
}

#[test]
fn release_evidence_policy_exists_and_declares_governance_rules() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/dag/policy/release_evidence_policy.json"))
            .expect("read release evidence policy"),
    )
    .expect("parse release evidence policy");
    assert_eq!(
        policy["release_set_source"].as_str().expect("release_set_source"),
        "evidence/release/release_evidence_set.json"
    );
    assert!(policy["legacy_roots_must_remain_deleted"]
        .as_array()
        .expect("legacy roots array")
        .contains(&serde_json::Value::String(format!("{}/{}/", "comparisons", "scenarios"))));
}
