use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

#[test]
fn artifact_io_hardening_fast_suite_covers_direct_contract_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = root.join("configs/suites/artifact_io_hardening_fast.json");
    assert!(suite.exists(), "missing artifact io hardening fast suite");

    let payload: Value = serde_json::from_str(&fs::read_to_string(&suite).expect("read suite"))
        .expect("parse suite");
    assert_eq!(payload["id"], "artifact-io-hardening-fast");

    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "io_store_fs_contracts",
        "artifact_io_expansion_contracts",
        "artifact_storage_resilience_contracts",
        "storage_services_contracts",
        "artifact_io_store_hardening_expansion_contracts",
    ] {
        assert!(
            commands
                .iter()
                .filter_map(|v| v.as_str())
                .any(|cmd| cmd.contains(required)),
            "artifact io hardening fast suite missing {required}"
        );
    }
}
