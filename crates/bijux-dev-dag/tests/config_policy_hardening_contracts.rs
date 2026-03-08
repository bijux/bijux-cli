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
fn config_policy_contract_and_report_surfaces_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/CONFIG_PRECEDENCE_CONTRACT.md",
        "docs/spec/POLICY_EVALUATION_TRACE.md",
        "docs/reports/foundation/config_policy_determinism_report.md",
        "docs/reports/foundation/config_inventory_report.md",
        "crates/bijux-dag-app/tests/config_precedence_contract.rs",
        "crates/bijux-dag-app/tests/config_validation_contract.rs",
        "crates/bijux-dag-app/tests/config_effective_command_contract.rs",
        "crates/bijux-dag-runtime/tests/security_model_contracts.rs",
    ] {
        assert!(
            root.join(required).exists(),
            "missing config/policy determinism surface: {required}"
        );
    }
}

#[test]
fn config_precedence_contract_covers_required_behavior() {
    let root = repo_root();
    let contract = fs::read_to_string(root.join("docs/spec/CONFIG_PRECEDENCE_CONTRACT.md"))
        .expect("config precedence contract should exist");
    for token in [
        "CLI > explicit config file > environment > defaults",
        "Unknown fields in explicit config must fail before execution.",
        "Malformed config files must fail before execution.",
        "Policy evaluation trace must be available for operator/debug inspection.",
        "dag config show-effective",
        "dag policy show-effective",
    ] {
        assert!(
            contract.contains(token),
            "config precedence contract missing required token `{token}`"
        );
    }
}

#[test]
fn battle_policy_keeps_config_policy_determinism_trust_property_mapped() {
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
                .is_some_and(|id| id == "tp_config_policy_determinism")
        }),
        "battle trust policy must include tp_config_policy_determinism"
    );

    let scenario_map = policy
        .get("scenario_trust_map")
        .and_then(serde_json::Value::as_object)
        .expect("scenario_trust_map should exist");
    assert!(
        scenario_map.values().any(|value| {
            value.as_array().is_some_and(|ids| {
                ids.iter().any(|id| {
                    id.as_str()
                        .is_some_and(|v| v == "tp_config_policy_determinism")
                })
            })
        }),
        "scenario_trust_map must include tp_config_policy_determinism"
    );
}
