use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("workspace root")
}

#[test]
fn state_machine_contract_documents_states_and_invariants() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/STATE_MACHINE_CONTRACT.md")).expect("contract");

    for token in [
        "- pending",
        "- eligible",
        "- queued",
        "- running",
        "- success",
        "- failed",
        "- skipped",
        "- cached",
        "- cancelled",
        "- submitted",
        "- planning",
        "- paused",
        "- interrupted",
        "- cancelling",
        "- succeeded",
        "INV-NODE-TRANSITION-*",
        "INV-NODE-TERMINAL-REVERT-001",
        "INV-RUN-TRANSITION-*",
        "INV-RUN-FAILED-CAUSAL-001",
        "run_dag_verify_state",
    ] {
        assert!(contract.contains(token), "state machine contract missing token: {token}");
    }
}

#[test]
fn state_machine_visualization_tracks_runtime_verification_surfaces() {
    let root = workspace_root();
    let visualization = fs::read_to_string(root.join("docs/spec/STATE_MACHINE_VISUALIZATION.md"))
        .expect("visualization");

    for token in [
        "stateDiagram-v2",
        "pending --> eligible",
        "queued --> running",
        "running --> succeeded",
        "validate_node_transition",
        "validate_run_transition",
        "verify_post_run_state_consistency",
    ] {
        assert!(
            visualization.contains(token),
            "state machine visualization missing token: {token}"
        );
    }
}

#[test]
fn state_machine_hardening_report_links_runtime_tests_and_command_surface() {
    let root = workspace_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/STATE_MACHINE_HARDENING_REPORT.md"))
            .expect("report");

    for token in [
        "docs/spec/STATE_MACHINE_CONTRACT.md",
        "docs/spec/STATE_MACHINE_VISUALIZATION.md",
        "crates/bijux-dag-runtime/src/runtime_core/execution/run_state.rs",
        "crates/bijux-dag-runtime/tests/state_machine_transitions.rs",
        "crates/bijux-dag-runtime/tests/state_machine_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_state_machine_contracts.rs",
        "crates/bijux-dev/tests/state_machine_hardening_contracts.rs",
        "run_dag_verify_state",
        "verify_post_run_state_consistency",
    ] {
        assert!(report.contains(token), "state machine hardening report missing: {token}");
    }
}
