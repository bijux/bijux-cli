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
    let fixture_dir = root.join("crates/bijux-dag-runtime/tests/fixtures/battle_workflows");
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(&fixture_dir).expect("battle fixture dir") {
        let path = entry.expect("fixture entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if path.file_name().and_then(|name| name.to_str()) == Some("metadata.json") {
            continue;
        }
        let raw = fs::read_to_string(&path).expect("battle fixture read");
        let doc: serde_json::Value = serde_json::from_str(&raw).expect("battle fixture parse");
        let scenario = doc
            .get("scenario")
            .and_then(serde_json::Value::as_str)
            .expect("scenario id")
            .to_string();
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .expect("file stem utf8");
        assert_eq!(scenario, stem, "scenario id must match filename");
        ids.insert(scenario);
    }
    ids
}

#[test]
fn battle_trust_mapping_and_metadata_have_no_orphans() {
    let root = repo_root();
    let policy: BattleTrustPolicy = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
            .expect("battle policy"),
    )
    .expect("battle policy parse");
    let metadata: BattleMetadata = serde_json::from_str(
        &fs::read_to_string(
            root.join("crates/bijux-dag-runtime/tests/fixtures/battle_workflows/metadata.json"),
        )
        .expect("battle metadata"),
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
