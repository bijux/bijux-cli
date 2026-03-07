use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn replay_contract_and_hardening_surfaces_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/REPLAY_CONTRACT.md",
        "docs/reports/foundation/replay_hardening_report.md",
        "configs/schema/operator/replay_diff.schema.json",
        "evidence/cache/replay/match_case.json",
        "evidence/cache/replay/mismatch_case.json",
        "evidence/cache/replay/corruption_case.json",
        "evidence/cache/replay/unsupported_version_case.json",
        "crates/bijux-dag-app/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/replay_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_replay_contracts.rs",
    ] {
        assert!(
            root.join(required).exists(),
            "missing replay hardening surface: {required}"
        );
    }
}

#[test]
fn replay_contract_covers_definition_explainability_and_non_goals() {
    let root = repo_root();
    let contract = fs::read_to_string(root.join("docs/spec/REPLAY_CONTRACT.md"))
        .expect("replay contract should exist");
    for token in [
        "## Replay definition",
        "## Authoritative inputs",
        "## Semantic diff mode",
        "## Replay explain mode",
        "## What replay cannot prove",
    ] {
        assert!(
            contract.contains(token),
            "replay contract missing required section `{token}`"
        );
    }
}

#[test]
fn battle_policy_keeps_replay_equivalence_trust_property_mapped() {
    let root = repo_root();
    let raw = fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
        .expect("battle trust policy should exist");
    let policy: serde_json::Value =
        serde_json::from_str(&raw).expect("battle trust policy should parse");

    let trust_properties = policy
        .get("trust_properties")
        .and_then(serde_json::Value::as_array)
        .expect("trust_properties should exist");
    assert!(
        trust_properties.iter().any(|item| {
            item.get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == "tp_replay_equivalence")
        }),
        "battle trust policy must include tp_replay_equivalence"
    );

    let scenario_map = policy
        .get("scenario_trust_map")
        .and_then(serde_json::Value::as_object)
        .expect("scenario_trust_map should exist");
    assert!(
        scenario_map.values().any(|value| {
            value.as_array().is_some_and(|ids| {
                ids.iter()
                    .any(|id| id.as_str().is_some_and(|v| v == "tp_replay_equivalence"))
            })
        }),
        "scenario_trust_map must include tp_replay_equivalence"
    );
}
