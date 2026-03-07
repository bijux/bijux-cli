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
fn cache_contract_and_hardening_report_surfaces_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/CACHE_CONTRACT.md",
        "docs/spec/CACHE_EVOLUTION_MODEL.md",
        "docs/reports/foundation/cache_hardening_report.md",
        "docs/tracking/CACHE_CORRECTNESS_COVERAGE.md",
        "crates/bijux-dag-app/tests/cache_evolution_contract.rs",
        "crates/bijux-dag-runtime/tests/cache_contracts.rs",
    ] {
        assert!(
            root.join(required).exists(),
            "missing cache hardening surface: {required}"
        );
    }
}

#[test]
fn cache_contract_covers_identity_proof_and_inspection_surfaces() {
    let root = repo_root();
    let contract = fs::read_to_string(root.join("docs/spec/CACHE_CONTRACT.md"))
        .expect("cache contract should exist");
    for token in [
        "Cache identity inputs",
        "Proof model",
        "Metadata version",
        "Lineage model",
        "dag cache explain",
        "dag cache verify",
        "dag cache stats",
        "dag cache diff",
    ] {
        assert!(
            contract.contains(token),
            "cache contract missing required token `{token}`"
        );
    }
}

#[test]
fn battle_policy_keeps_cache_integrity_trust_property_mapped() {
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
                .is_some_and(|id| id == "tp_cache_integrity")
        }),
        "battle trust policy must include tp_cache_integrity"
    );

    let scenario_map = policy
        .get("scenario_trust_map")
        .and_then(serde_json::Value::as_object)
        .expect("scenario_trust_map should exist");
    assert!(
        scenario_map.values().any(|value| {
            value.as_array().is_some_and(|ids| {
                ids.iter()
                    .any(|id| id.as_str().is_some_and(|v| v == "tp_cache_integrity"))
            })
        }),
        "scenario_trust_map must include tp_cache_integrity"
    );
}
