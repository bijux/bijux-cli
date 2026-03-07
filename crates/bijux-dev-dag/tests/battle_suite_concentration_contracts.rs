use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct BattleTrustPolicy {
    trust_properties: Vec<TrustProperty>,
    release_top_trust_properties: Vec<String>,
    required_scenarios: Vec<String>,
    scenario_trust_map: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct TrustProperty {
    id: String,
}

#[derive(Debug, Deserialize)]
struct BattleMetadata {
    scenarios: BTreeMap<String, ScenarioMetadata>,
}

#[derive(Debug, Deserialize)]
struct ScenarioMetadata {
    grade: String,
    why_exists: String,
    delete_review: String,
    release_blocking: bool,
    owning_family: String,
    expected_outcome_class: String,
    expected_invariant_classes: Vec<String>,
    expected_operator_inspection_surfaces: Vec<String>,
    replay_cache_implications: ReplayCacheImplications,
}

#[derive(Debug, Deserialize)]
struct ReplayCacheImplications {
    replay_equivalence_expected: bool,
    cache_integrity_expected: bool,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

fn battle_fixture_ids(root: &Path) -> BTreeSet<String> {
    let fixture_dir = root.join("evidence/battle/workflows");
    let mut files = Vec::new();
    collect_json_files(&fixture_dir, &mut files);
    let mut ids = BTreeSet::new();
    for path in files {
        let raw = fs::read_to_string(&path).expect("battle fixture read");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("battle fixture parse");
        let scenario = doc
            .get("scenario")
            .and_then(serde_json::Value::as_str)
            .or_else(|| doc.get("scenario_id").and_then(serde_json::Value::as_str))
            .expect("scenario id")
            .to_string();
        if doc.get("scenario").is_some() {
            let stem = path
                .file_stem()
                .and_then(|s| s.to_str())
                .expect("file stem utf8");
            assert_eq!(scenario, stem, "scenario id must match filename");
        }
        ids.insert(scenario);
    }
    ids
}

fn collect_json_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("read battle fixture dir") {
        let path = entry.expect("fixture entry").path();
        if path.is_dir() {
            collect_json_files(&path, out);
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            out.push(path);
        }
    }
}

#[test]
fn battle_trust_mapping_and_metadata_have_no_orphans() {
    let root = repo_root();
    let release_subset: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_release_blocking_subset.json"))
            .expect("read release subset policy"),
    )
    .expect("parse release subset policy");
    let advisory: BTreeSet<String> = release_subset["advisory_scenarios"]
        .as_array()
        .expect("advisory_scenarios array")
        .iter()
        .map(|value| value.as_str().expect("advisory scenario id").to_string())
        .collect();
    let policy: BattleTrustPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .expect("battle policy"),
    )
    .expect("battle policy parse");
    let metadata: BattleMetadata = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/battle/metadata.json")).expect("battle metadata"),
    )
    .expect("battle metadata parse");

    let trust_property_ids: BTreeSet<String> = policy
        .trust_properties
        .iter()
        .map(|property| property.id.clone())
        .collect();
    assert!(
        trust_property_ids.len() >= 12,
        "battle trust property count must remain at least 12"
    );
    assert!(
        trust_property_ids.contains("tp_plan_truth"),
        "battle trust properties must include tp_plan_truth"
    );
    assert!(
        !policy.release_top_trust_properties.is_empty(),
        "battle policy must define a non-empty release_top_trust_properties set"
    );
    for top in &policy.release_top_trust_properties {
        assert!(
            trust_property_ids.contains(top),
            "top trust property is not defined in trust_properties: {top}"
        );
        let covered = policy
            .scenario_trust_map
            .values()
            .any(|mapped| mapped.iter().any(|trust| trust == top));
        assert!(
            covered,
            "top trust property has no scenario coverage: {top}"
        );
    }

    let fixture_ids = battle_fixture_ids(&root);
    for scenario in &policy.required_scenarios {
        assert!(
            fixture_ids.contains(scenario),
            "required scenario missing: {scenario}"
        );
    }

    for (scenario, mapped) in &policy.scenario_trust_map {
        assert!(
            fixture_ids.contains(scenario),
            "orphan trust mapping: {scenario}"
        );
        assert!(!mapped.is_empty(), "empty trust mapping: {scenario}");
        assert!(
            mapped.len() <= 3,
            "overloaded battle scenario should be split: {scenario} maps {} trust properties",
            mapped.len()
        );
        for trust_property in mapped {
            assert!(
                trust_property_ids.contains(trust_property),
                "unknown trust property `{trust_property}` for scenario `{scenario}`"
            );
        }
    }

    for (scenario, scenario_metadata) in &metadata.scenarios {
        assert!(
            fixture_ids.contains(scenario),
            "orphan metadata entry: {scenario}"
        );
        assert_eq!(scenario_metadata.grade, "battle");
        assert!(!scenario_metadata.why_exists.trim().is_empty());
        assert_eq!(scenario_metadata.delete_review, "retain");
        if advisory.contains(scenario) {
            assert!(
                !scenario_metadata.release_blocking,
                "advisory scenario must not be release-blocking: {scenario}"
            );
        } else {
            assert!(scenario_metadata.release_blocking);
        }
        assert!(!scenario_metadata.owning_family.trim().is_empty());
        assert!(!scenario_metadata.expected_outcome_class.trim().is_empty());
        assert!(!scenario_metadata.expected_invariant_classes.is_empty());
        assert!(!scenario_metadata
            .expected_operator_inspection_surfaces
            .is_empty());
        let mapped = policy
            .scenario_trust_map
            .get(scenario)
            .expect("scenario trust map entry");
        let protects_top = mapped
            .iter()
            .any(|trust| policy.release_top_trust_properties.contains(trust));
        assert!(
            protects_top,
            "release-blocking scenario must protect at least one top trust property: {scenario}"
        );
        assert_eq!(
            scenario_metadata
                .replay_cache_implications
                .replay_equivalence_expected,
            mapped.iter().any(|id| id == "tp_replay_equivalence")
        );
        assert_eq!(
            scenario_metadata
                .replay_cache_implications
                .cache_integrity_expected,
            mapped.iter().any(|id| id == "tp_cache_integrity")
        );
    }
}

#[test]
fn foundation_repo_suite_keeps_battle_guard() {
    let root = repo_root();
    let repo_suites = fs::read_to_string(root.join("crates/bijux-dev-dag/src/suites/repo.rs"))
        .expect("repo suites");
    assert!(
        repo_suites.contains("\"battle-suite-mandatory\""),
        "repo suite must keep battle-suite-mandatory guard"
    );
}

#[test]
fn battle_query_commands_are_wired() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read command source");
    for token in [
        "BattleScenariosByTrust",
        "BattleTrustByScenario",
        "repo.battle-scenarios-by-trust",
        "repo.battle-trust-by-scenario",
        "repo.battle-coverage-report",
    ] {
        assert!(
            source.contains(token),
            "missing battle query/report command token: {token}"
        );
    }
}

#[test]
fn battle_trust_property_registry_exists() {
    let root = repo_root();
    let registry_path = root.join("evidence/battle/registries/trust_property_registry.json");
    assert!(
        registry_path.exists(),
        "battle trust-property registry is missing: {}",
        registry_path.display()
    );
    let payload =
        fs::read_to_string(&registry_path).expect("read battle trust-property registry payload");
    let parsed: serde_json::Value =
        serde_json::from_str(&payload).expect("parse battle trust registry");
    assert!(
        parsed
            .get("trust_properties")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|arr| !arr.is_empty()),
        "battle trust-property registry must declare trust_properties"
    );
}
