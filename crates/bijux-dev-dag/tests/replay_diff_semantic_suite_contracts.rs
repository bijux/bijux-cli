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
fn replay_diff_semantic_fast_suite_keeps_proof_and_trace_contracts() {
    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/suites/replay_diff_semantic_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(suite_path).expect("suite")).expect("json");

    assert_eq!(payload["id"], "replay-diff-semantic-fast");
    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "replay_proof_contract",
        "replay_diff_hardening_contract",
        "replay_semantic_surface_contracts",
    ] {
        assert!(commands.contains(required), "missing command: {required}");
    }
}
