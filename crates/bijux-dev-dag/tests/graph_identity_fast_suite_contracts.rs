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

#[test]
fn graph_identity_fast_suite_includes_property_and_regression_contracts() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root");
    let payload: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("configs/suites/graph_identity_regression_fast.json"))
            .expect("read suite"),
    )
    .expect("parse suite");
    assert_eq!(payload["id"], "graph-identity-regression-fast");
    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for required in [
        "graph_identity_property_contracts",
        "graph_identity_contract",
        "graph_identity_expansion_contract",
    ] {
        assert!(
            commands.contains(required),
            "graph identity fast suite missing {required}"
        );
    }
}
