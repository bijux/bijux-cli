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
fn foundation_now_suite_lists_identity_replay_artifact_and_scheduler_checks() {
    let suite_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/suites/foundation_now.json");
    let suite: Value =
        serde_json::from_str(&std::fs::read_to_string(&suite_path).expect("read suite"))
            .expect("parse suite");

    assert_eq!(suite["id"], "foundation-now");
    let commands = suite["commands"].as_array().expect("commands array");
    let command_text = commands
        .iter()
        .filter_map(|v| v.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "graph_identity_contracts",
        "replay_diff_hardening_contract",
        "artifact_identity_and_lineage_contracts",
        "scheduler_invariants_contracts",
        "smoke_pipeline",
    ] {
        assert!(
            command_text.contains(required),
            "foundation-now suite must include {required}"
        );
    }
}
