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
fn runtime_helper_fast_suite_covers_helper_invariant_targets() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let suite = root.join("configs/suites/runtime_helper_invariants_fast.json");
    assert!(suite.exists(), "missing runtime helper fast suite");

    let payload: Value =
        serde_json::from_str(&fs::read_to_string(&suite).expect("read suite")).expect("parse suite");
    assert_eq!(payload["id"], "runtime-helper-invariants-fast");

    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "runtime_execution_helper_expansion_contracts",
        "runtime_scheduler_state_machine_invariants_contracts",
        "scheduler_workload_contracts",
        "runtime_semantics_contracts",
        "planner_analysis_contract",
    ] {
        assert!(
            commands
                .iter()
                .filter_map(|v| v.as_str())
                .any(|cmd| cmd.contains(required)),
            "runtime helper fast suite missing {required}"
        );
    }
}
