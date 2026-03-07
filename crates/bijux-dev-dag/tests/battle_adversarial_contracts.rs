use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn required_adversarial_ids() -> Vec<&'static str> {
    vec![
        "adversarial-concurrency-retry-determinism",
        "adversarial-post-success-artifact-corruption",
        "adversarial-cache-proof-corruption-plausible-outputs",
        "adversarial-replay-semantic-drift-detection",
        "adversarial-import-export-semantic-loss-rejected",
        "adversarial-operator-only-recovery-path",
        "adversarial-policy-denial-blocks-unsafe-execution",
        "adversarial-missing-outputs-superficial-success-rejected",
        "adversarial-tie-break-stability-under-contention",
        "adversarial-cancel-retry-bookkeeping-integrity",
        "adversarial-path-escape-via-declared-outputs-blocked",
        "adversarial-env-leakage-via-adapters-blocked",
        "adversarial-partial-run-dir-not-finalized",
        "adversarial-imported-runs-remain-visible",
    ]
}

#[test]
fn adversarial_battle_scenarios_exist_and_are_mapped() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .expect("read battle trust policy"),
    )
    .expect("parse battle trust policy");

    let required: BTreeSet<String> = policy["required_scenarios"]
        .as_array()
        .expect("required_scenarios array")
        .iter()
        .map(|v| v.as_str().expect("scenario id").to_string())
        .collect();

    for scenario in required_adversarial_ids() {
        let is_advisory = matches!(
            scenario,
            "adversarial-partial-run-dir-not-finalized"
                | "adversarial-imported-runs-remain-visible"
        );
        if !is_advisory {
            assert!(
                required.contains(scenario),
                "missing release-blocking adversarial scenario in required_scenarios: {scenario}"
            );
        }
        let mapped = policy["scenario_trust_map"]
            .get(scenario)
            .and_then(serde_json::Value::as_array)
            .expect("scenario trust mapping");
        assert!(
            !mapped.is_empty(),
            "adversarial scenario missing trust-property mapping: {scenario}"
        );
        assert!(
            mapped.len() <= 3,
            "adversarial scenario maps too many trust properties: {scenario}"
        );
    }
}

#[test]
fn adversarial_metadata_has_invariant_operator_and_replay_cache_bundles() {
    let root = repo_root();
    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/battle/metadata.json"))
            .expect("read battle metadata"),
    )
    .expect("parse battle metadata");

    for scenario in required_adversarial_ids() {
        let entry = metadata["scenarios"]
            .get(scenario)
            .unwrap_or_else(|| panic!("metadata missing scenario: {scenario}"));
        assert!(
            entry["expected_invariant_classes"]
                .as_array()
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            "scenario missing invariant bundle: {scenario}"
        );
        assert!(
            entry["expected_operator_inspection_surfaces"]
                .as_array()
                .map(|v| !v.is_empty())
                .unwrap_or(false),
            "scenario missing operator-visible expectations: {scenario}"
        );
        let replay_cache = &entry["replay_cache_implications"];
        assert!(
            replay_cache["replay_equivalence_expected"].is_boolean(),
            "scenario missing replay implication flag: {scenario}"
        );
        assert!(
            replay_cache["cache_integrity_expected"].is_boolean(),
            "scenario missing cache implication flag: {scenario}"
        );
    }
}

#[test]
fn adversarial_release_blocking_subset_is_enforced_and_registry_has_no_duplicates() {
    let root = repo_root();
    let subset: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_release_blocking_subset.json"))
            .expect("read release subset policy"),
    )
    .expect("parse release subset policy");
    let metadata: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/battle/metadata.json"))
            .expect("read battle metadata"),
    )
    .expect("parse battle metadata");
    let registry: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/battle/registries/scenario_registry.json"))
            .expect("read battle scenario registry"),
    )
    .expect("parse battle scenario registry");

    for scenario in subset["release_blocking_scenarios"]
        .as_array()
        .expect("release blocking scenarios array")
    {
        let id = scenario.as_str().expect("scenario id");
        assert_eq!(
            metadata["scenarios"][id]["release_blocking"].as_bool(),
            Some(true),
            "release-blocking subset scenario must be release_blocking=true: {id}"
        );
    }

    for scenario in subset["advisory_scenarios"]
        .as_array()
        .expect("advisory scenarios array")
    {
        let id = scenario.as_str().expect("scenario id");
        assert_eq!(
            metadata["scenarios"][id]["release_blocking"].as_bool(),
            Some(false),
            "advisory subset scenario must be release_blocking=false: {id}"
        );
    }

    let mut seen = BTreeSet::new();
    for entry in registry["entries"].as_array().expect("registry entries") {
        let scenario = entry["scenario"].as_str().expect("registry scenario");
        assert!(
            seen.insert(scenario.to_string()),
            "duplicate scenario in registry: {scenario}"
        );
    }
}

#[test]
fn adversarial_concentration_report_exists() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/battle_adversarial_concentration_report.md"),
    )
    .expect("read adversarial concentration report");
    assert!(
        report.contains("release-blocking") && report.contains("duplicate"),
        "concentration report must describe release-blocking subset and overlap reduction"
    );
}
