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
fn graph_core_fast_suite_covers_canonical_topology_validate_scope() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = root.join("configs/suites/graph_core_canonical_topology_validate_fast.json");
    assert!(suite.exists(), "missing graph core fast suite");

    let payload: Value = serde_json::from_str(&fs::read_to_string(&suite).expect("read suite"))
        .expect("parse suite");
    assert_eq!(payload["id"], "graph-core-canonical-topology-validate-fast");

    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "direct_module_entrypoints_contracts",
        "graph_pipeline_planner_expansion_contracts",
        "graph_identity_property_contracts",
        "planner_contract",
        "planner_validation_remaining_contracts",
    ] {
        assert!(
            commands
                .iter()
                .filter_map(|v| v.as_str())
                .any(|cmd| cmd.contains(required)),
            "graph core fast suite missing {required}"
        );
    }
}
