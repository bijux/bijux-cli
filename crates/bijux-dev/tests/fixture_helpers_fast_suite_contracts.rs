use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use tempfile as _;

#[test]
fn fixture_helpers_fast_suite_covers_shared_loader_and_governance_contracts() {
    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/suites/fixture_helpers_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(&suite_path).expect("read suite"))
            .expect("parse suite");

    assert_eq!(payload["id"], "fixture-helpers-fast");
    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "cargo test -p bijux-dag-testkit --test fixture_loader_contracts",
        "cargo test -p bijux-dev --test fixture_loader_governance_contracts",
    ] {
        assert!(commands.contains(required), "fixture helper fast suite missing {required}");
    }
}
