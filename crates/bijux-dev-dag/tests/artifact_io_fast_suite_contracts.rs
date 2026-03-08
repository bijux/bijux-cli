use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

#[test]
fn artifact_io_fast_suite_covers_store_fs_and_inspect_corruption_contracts() {
    let suite = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/suites/artifact_io_zero_coverage_fast.json");
    let payload: Value = serde_json::from_str(&std::fs::read_to_string(suite).expect("suite"))
        .expect("json");
    assert_eq!(payload["id"], "artifact-io-zero-coverage-fast");

    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "io_store_fs_contracts",
        "artifact_io_expansion_contracts",
        "artifact_inspect_storage_contracts",
    ] {
        assert!(commands.contains(required), "missing suite command: {required}");
    }
}
