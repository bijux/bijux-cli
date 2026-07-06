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
fn scheduler_contract_documents_determinism_and_budget_invariants() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/SCHEDULER_CONTRACT.md")).expect("contract");

    for section in [
        "## Scope",
        "## Canonical scheduling model",
        "## Determinism invariants",
        "## Budget and cancellation invariants",
        "## Failure and retry semantics",
        "## Observability proof",
    ] {
        assert!(contract.contains(section), "scheduler contract missing section: {section}");
    }

    for token in [
        "scheduler_contract_profile()",
        "deterministic_schedule_order",
        "PriorityCpuMemoryFitThenNodeId",
        "scheduler_invariants_hold",
        "blocked_by_budget",
        "run_dag_scheduler_timeline",
    ] {
        assert!(contract.contains(token), "scheduler contract missing token: {token}");
    }
}

#[test]
fn scheduler_state_transition_contract_covers_runtime_methods_and_events() {
    let root = workspace_root();
    let contract = fs::read_to_string(root.join("docs/spec/SCHEDULER_STATE_TRANSITIONS.md"))
        .expect("state transition contract");

    for token in [
        "SchedulerState",
        "complete_success",
        "complete_cached",
        "complete_skipped",
        "complete_failed",
        "queue_retry",
        "requeue_retries",
        "NodeRetryQueued",
        "NodeRetryRequeued",
        "ExecutionCheckpoint",
    ] {
        assert!(
            contract.contains(token),
            "scheduler state transition contract missing token: {token}"
        );
    }
}

#[test]
fn scheduler_hardening_report_links_runtime_and_command_surfaces() {
    let root = workspace_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/SCHEDULER_HARDENING_REPORT.md"))
            .expect("report");

    for token in [
        "docs/spec/SCHEDULER_CONTRACT.md",
        "docs/spec/SCHEDULER_STATE_TRANSITIONS.md",
        "crates/bijux-dag-runtime/src/runtime_core/execution/scheduler.rs",
        "crates/bijux-dag-runtime/tests/scheduler_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs",
        "crates/bijux-dev/tests/scheduler_hardening_contracts.rs",
        "run_dag_scheduler_timeline",
        "observability.timeline.json",
    ] {
        assert!(report.contains(token), "scheduler hardening report missing: {token}");
    }
}
