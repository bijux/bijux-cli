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

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn state_machine_contract_covers_states_transitions_and_invariants() {
    let root = repo_root();
    let contract = fs::read_to_string(root.join("docs/spec/STATE_MACHINE_CONTRACT.md"))
        .expect("state machine contract should exist");

    for token in [
        "pending",
        "eligible",
        "queued",
        "running",
        "success",
        "failed",
        "skipped",
        "cached",
        "cancelled",
        "submitted",
        "planning",
        "paused",
        "interrupted",
        "cancelling",
        "succeeded",
        "INV-NODE-TRANSITION-*",
        "INV-NODE-TERMINAL-REVERT-001",
        "INV-RUN-TRANSITION-*",
        "INV-RUN-FAILED-CAUSAL-001",
    ] {
        assert!(
            contract.contains(token),
            "state machine contract missing required token `{token}`"
        );
    }
}

#[test]
fn state_machine_hardening_surfaces_and_reference_traces_exist() {
    let root = repo_root();
    for required in [
        "docs/reports/foundation/state_machine_hardening_report.md",
        "crates/bijux-dag-runtime/tests/state_machine_transitions.rs",
        "crates/bijux-dag-runtime/tests/state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/evolution_trace.json",
        "crates/bijux-dag-runtime/tests/fixtures/state_machine/cancellation_trace.json",
    ] {
        assert!(
            root.join(required).exists(),
            "missing state machine hardening surface: {required}"
        );
    }
}

#[test]
fn battle_policy_keeps_state_machine_legality_mandatory() {
    let root = repo_root();
    let raw = fs::read_to_string(root.join("configs/policy/battle_trust_properties.json"))
        .expect("battle trust policy should exist");
    let policy: serde_json::Value =
        serde_json::from_str(&raw).expect("battle trust policy should parse");

    let trust_properties = policy
        .get("trust_properties")
        .and_then(serde_json::Value::as_array)
        .expect("trust_properties should exist");
    assert!(
        trust_properties.iter().any(|item| {
            item.get("id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|id| id == "tp_state_machine_legality")
        }),
        "battle trust policy must include tp_state_machine_legality"
    );

    let scenario_map = policy
        .get("scenario_trust_map")
        .and_then(serde_json::Value::as_object)
        .expect("scenario_trust_map should exist");
    assert!(
        scenario_map.values().any(|value| {
            value
                .as_array()
                .is_some_and(|ids| ids.iter().any(|id| id == "tp_state_machine_legality"))
        }),
        "scenario_trust_map must map at least one scenario to tp_state_machine_legality"
    );
}
