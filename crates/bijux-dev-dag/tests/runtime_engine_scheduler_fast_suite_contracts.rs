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
fn runtime_engine_scheduler_fast_suite_covers_execution_helpers_and_scheduler_invariants() {
    let suite = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../configs/dag/suites/runtime_engine_scheduler_fast.json");
    let payload: Value =
        serde_json::from_str(&std::fs::read_to_string(suite).expect("suite")).expect("json");
    assert_eq!(payload["id"], "runtime-engine-scheduler-fast");

    let commands = payload["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    for required in [
        "runtime_execution_module_entrypoints_contracts",
        "runtime_execution_resilience_contracts",
        "runtime_scheduler_state_machine_invariants_contracts",
        "scheduler_workload_contracts",
    ] {
        assert!(
            commands.contains(required),
            "runtime engine/scheduler fast suite missing {required}"
        );
    }
}
