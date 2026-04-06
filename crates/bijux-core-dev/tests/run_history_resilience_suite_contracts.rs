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
fn run_history_resilience_fast_suite_keeps_damaged_run_no_panic_coverage() {
    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/suites/run_history_resilience_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(suite_path).expect("suite")).expect("json");

    assert_eq!(payload["id"], "run-history-resilience-fast");
    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "run_history_contract",
        "run_history_hardening_contract",
        "run_history_reliability_contract",
        "run_history_ancestry_contracts",
    ] {
        assert!(
            commands.contains(required),
            "run history resilience suite missing {required}"
        );
    }
}
