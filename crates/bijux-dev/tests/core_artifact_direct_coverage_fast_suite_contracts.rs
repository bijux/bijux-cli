use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[test]
fn core_artifact_fast_suite_covers_direct_coverage_contract_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = root.join("configs/dag/suites/core_artifact_direct_coverage_fast.json");
    assert!(suite.exists(), "missing core/artifact direct coverage fast suite");

    let payload: Value = serde_json::from_str(&fs::read_to_string(&suite).expect("read suite"))
        .expect("parse suite");

    assert_eq!(payload["id"], "core-artifact-direct-coverage-fast");
    let commands = payload["commands"].as_array().expect("commands array");
    let required = [
        "canonical_contract",
        "direct_module_entrypoints_contracts",
        "validation_coverage",
        "graph_identity_property_contracts",
        "io_store_fs_contracts",
        "artifact_io_expansion_contracts",
        "storage_services_contracts",
        "artifact_storage_resilience_contracts",
    ];

    for name in required {
        assert!(
            commands.iter().filter_map(|v| v.as_str()).any(|cmd| cmd.contains(name)),
            "core/artifact fast suite missing {name}"
        );
    }
}
