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
fn run_dir_and_import_export_contract_surfaces_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/RUN_DIR_CONTRACT.md",
        "docs/spec/IMPORT_EXPORT_CONTRACT.md",
        "docs/spec/RUN_DIR_OWNERSHIP.md",
        "docs/reports/foundation/run_dir_import_export_hardening_report.md",
        "crates/bijux-dag-artifacts/src/storage/hardening.rs",
        "crates/bijux-dag-artifacts/tests/artifact_hardening_contracts.rs",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "evidence/compat/export_bundle/v0_1_supported/bundle.json",
        "evidence/compat/export_bundle/unsupported_past/bundle.json",
    ] {
        assert!(
            root.join(required).exists(),
            "missing run-dir/import-export hardening surface: {required}"
        );
    }
}

#[test]
fn run_dir_contract_documents_authoritative_optional_and_verification_sections() {
    let root = repo_root();
    let run_dir_contract = fs::read_to_string(root.join("docs/spec/RUN_DIR_CONTRACT.md"))
        .expect("run-dir contract should exist");
    for token in [
        "Required entries (authoritative)",
        "Optional entries",
        "Derived artifacts (non-authoritative)",
        "dag verify --strict",
    ] {
        assert!(
            run_dir_contract.contains(token),
            "run-dir contract missing required token `{token}`"
        );
    }
}

#[test]
fn battle_policy_keeps_run_dir_and_import_export_trust_properties() {
    let root = repo_root();
    let raw = fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
        .expect("battle trust policy should exist");
    let policy: serde_json::Value =
        serde_json::from_str(&raw).expect("battle trust policy should parse");

    let trust_properties = policy
        .get("trust_properties")
        .and_then(serde_json::Value::as_array)
        .expect("trust_properties should exist");
    for required in ["tp_run_dir_resilience", "tp_import_export_compatibility"] {
        assert!(
            trust_properties.iter().any(|item| {
                item.get("id")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|id| id == required)
            }),
            "battle trust policy must include {required}"
        );
    }

    let scenario_map = policy
        .get("scenario_trust_map")
        .and_then(serde_json::Value::as_object)
        .expect("scenario_trust_map should exist");
    let mapped_ids = scenario_map
        .values()
        .filter_map(serde_json::Value::as_array)
        .flat_map(|arr| arr.iter())
        .filter_map(serde_json::Value::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    assert!(
        mapped_ids.contains("tp_run_dir_resilience"),
        "scenario_trust_map must include tp_run_dir_resilience"
    );
    assert!(
        mapped_ids.contains("tp_import_export_compatibility"),
        "scenario_trust_map must include tp_import_export_compatibility"
    );
}
