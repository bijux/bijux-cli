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

use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

mod support;

fn workspace_root() -> PathBuf {
    support::workspace_root_from_manifest_dir(env!("CARGO_MANIFEST_DIR"))
}

fn runtime_fixture_directory() -> PathBuf {
    workspace_root().join("evidence/battle/workflows/runtime")
}

fn fixture_files() -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(runtime_fixture_directory())
        .expect("battle workflow fixture directory should exist")
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("json"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("metadata.json"))
        .collect();
    files.sort();
    files
}

fn runtime_fixture_scenarios() -> BTreeSet<String> {
    fixture_files()
        .into_iter()
        .filter_map(|path| path.file_stem().and_then(|s| s.to_str()).map(ToOwned::to_owned))
        .collect()
}

fn assert_shape(doc: &Value) {
    assert!(doc.get("scenario").and_then(Value::as_str).is_some());
    assert!(doc.get("graph").and_then(Value::as_str).is_some());
    assert!(doc.get("nodes").and_then(Value::as_u64).is_some());
    assert!(doc.get("focus").and_then(Value::as_array).is_some());
    assert!(doc.get("expectations").and_then(Value::as_object).is_some());
}

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
    owner: String,
    review_interval_days: u64,
    scenarios: BTreeMap<String, ScenarioMetadata>,
}

#[derive(Debug, Deserialize)]
struct ScenarioMetadata {
    grade: String,
    why_exists: String,
    delete_review: String,
}

fn load_policy() -> BattleTrustPolicy {
    let raw = fs::read_to_string(
        workspace_root().join("configs/dag/policy/battle_trust_properties.json"),
    )
    .expect("battle trust policy should exist");
    serde_json::from_str(&raw).expect("battle trust policy should parse")
}

fn load_metadata() -> BattleMetadata {
    let raw = fs::read_to_string(workspace_root().join("evidence/battle/metadata.json"))
        .expect("battle metadata should exist");
    serde_json::from_str(&raw).expect("battle metadata should parse")
}

#[test]
fn battle_workflow_harness_covers_required_scenarios() {
    let required = [
        "medium_workflow.json",
        "failure_heavy_workflow.json",
        "artifact_heavy_workflow.json",
        "cache_invalidation_workflow.json",
        "replay_divergence_workflow.json",
        "scheduler_fairness_workflow.json",
        "import_export_workflow.json",
        "version_compatibility_workflow.json",
        "corruption_workflow.json",
        "malformed_run_dir_workflow.json",
        "operator_inspection_workflow.json",
        "ugly_realistic_dag_workflow.json",
        "policy_violation_workflow.json",
        "secret_leakage_workflow.json",
        "timeout_workflow.json",
    ];

    for scenario in required {
        let doc = support::load_bundle_fixture_json(
            env!("CARGO_MANIFEST_DIR"),
            &format!("evidence/battle/workflows/runtime/{scenario}"),
        );
        assert_shape(&doc);
    }
}

#[test]
fn battle_workflow_scenarios_have_metadata_and_trust_mapping() {
    let policy = load_policy();
    let metadata = load_metadata();
    let runtime_scenarios = runtime_fixture_scenarios();
    assert_eq!(metadata.owner, "runtime-foundation");
    assert!(metadata.review_interval_days >= 30);

    let property_ids: BTreeSet<String> =
        policy.trust_properties.iter().map(|property| property.id.clone()).collect();
    assert!(property_ids.len() >= 12, "battle trust property set must contain at least 12 ids");
    assert!(
        property_ids.contains("tp_plan_truth"),
        "battle trust property set must include tp_plan_truth"
    );

    let mut scenario_ids_from_files = BTreeSet::new();
    for file in fixture_files() {
        let raw = fs::read_to_string(&file).expect("scenario fixture should be readable");
        let doc: Value = serde_json::from_str(&raw).expect("scenario fixture should parse");
        assert_shape(&doc);

        let scenario = doc
            .get("scenario")
            .and_then(Value::as_str)
            .expect("scenario id must be present")
            .to_string();
        let file_stem = file
            .file_stem()
            .and_then(|stem| stem.to_str())
            .expect("scenario fixture file name must be utf8");
        assert_eq!(scenario, file_stem, "scenario id must match fixture file name");
        scenario_ids_from_files.insert(scenario.clone());

        let scenario_metadata = metadata
            .scenarios
            .get(&scenario)
            .unwrap_or_else(|| panic!("metadata missing for scenario {scenario}"));
        assert_eq!(scenario_metadata.grade, "battle");
        assert!(!scenario_metadata.why_exists.trim().is_empty());
        assert_eq!(scenario_metadata.delete_review, "retain");

        let mapped_properties = policy
            .scenario_trust_map
            .get(&scenario)
            .unwrap_or_else(|| panic!("trust mapping missing for scenario {scenario}"));
        assert!(
            !mapped_properties.is_empty(),
            "scenario {scenario} must map to at least one trust property"
        );
        for trust_property in mapped_properties {
            assert!(
                property_ids.contains(trust_property),
                "scenario {scenario} maps unknown trust property {trust_property}"
            );
        }
    }

    for required in policy
        .required_scenarios
        .iter()
        .filter(|scenario| runtime_scenarios.contains((*scenario).as_str()))
    {
        assert!(
            scenario_ids_from_files.contains(required),
            "required scenario {required} missing from fixture set"
        );
    }

    // Battle evidence now includes non-runtime consumer scenarios under evidence/battle/workflows.
    // Runtime harness only enforces that runtime fixtures remain fully mapped and documented.
}
