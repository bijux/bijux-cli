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
fn helper_fast_suite_config_includes_required_contract_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite_path = root.join("configs/dag/suites/dev_dag_helpers_fast.json");
    let payload: Value =
        serde_json::from_str(&fs::read_to_string(&suite_path).expect("read suite"))
            .expect("parse suite");

    assert_eq!(payload["suite"], "dev-dag-helpers-fast");
    assert_eq!(payload["lane"], "fast");

    let tests = payload["tests"].as_array().expect("tests array");
    for required in [
        "crates/bijux-dev/tests/dev_dag_direct_test_presence_contracts.rs",
        "crates/bijux-dev/tests/dev_dag_helpers_fast_suite_contracts.rs",
        "crates/bijux-dev/tests/dev_dag_helper_small_module_test_gate_contracts.rs",
        "crates/bijux-dev/tests/dev_dag_reporting_integrity_contracts.rs",
    ] {
        assert!(
            tests.iter().any(|v| v.as_str() == Some(required)),
            "helper fast suite missing required contract {required}"
        );
    }
}
