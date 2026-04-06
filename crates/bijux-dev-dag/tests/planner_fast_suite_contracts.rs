use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use serde_json::Value;

#[test]
fn planner_fast_suite_covers_identity_closure_and_capability_rejection() {
    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/suites/planner_identity_closure_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(suite_path).expect("suite")).expect("json");

    assert_eq!(payload["id"], "planner-identity-closure-fast");
    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "planner_contract",
        "planner_fixture_contracts",
        "planner_validation_remaining_contracts",
        "planner_error_and_schema_contracts",
    ] {
        assert!(
            commands.contains(required),
            "planner fast suite missing {required}"
        );
    }
}
