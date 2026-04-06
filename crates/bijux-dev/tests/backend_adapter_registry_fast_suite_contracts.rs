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
fn backend_adapter_registry_fast_suite_covers_runtime_registry_surfaces() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = root.join("configs/dag/suites/backend_adapter_registry_fast.json");
    assert!(suite.exists(), "missing backend adapter registry fast suite");

    let payload: Value = serde_json::from_str(&fs::read_to_string(&suite).expect("read suite"))
        .expect("parse suite");
    assert_eq!(payload["id"], "backend-adapter-registry-fast");

    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "adapter_registry_capability_contracts",
        "backend_capability_boundary_contracts",
        "backend_contract",
        "execution_backend_contract",
        "backend_capability_docs_generation_contracts",
    ] {
        assert!(
            commands.iter().filter_map(|v| v.as_str()).any(|cmd| cmd.contains(required)),
            "backend adapter registry fast suite missing {required}"
        );
    }
}
