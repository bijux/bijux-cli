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
fn scheduler_contract_covers_canonical_semantics() {
    let root = repo_root();
    let contract = fs::read_to_string(root.join("docs/spec/SCHEDULER_CONTRACT.md"))
        .expect("scheduler contract should exist");

    for required in [
        "Canonical Unit",
        "node",
        "Tie-breaking",
        "lexical order",
        "Retry Semantics",
        "complete_cached",
        "complete_skipped",
        "Failure propagation",
        "SchedulerState",
    ] {
        assert!(
            contract.contains(required),
            "scheduler contract missing required token `{required}`"
        );
    }
}

#[test]
fn scheduler_invariant_surfaces_and_report_exist() {
    let root = repo_root();
    for required in [
        "crates/bijux-dag-runtime/tests/scheduler_contract.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs",
        "docs/reports/foundation/scheduler_hardening_report.md",
    ] {
        assert!(
            root.join(required).exists(),
            "missing scheduler hardening surface: {required}"
        );
    }
}

#[test]
fn foundation_guard_keeps_scheduler_invariants_mandatory() {
    let root = repo_root();
    let repo_suites = fs::read_to_string(root.join("crates/bijux-dev-dag/src/suites/repo.rs"))
        .expect("repo suites should exist");
    assert!(
        repo_suites.contains("\"scheduler-invariants\""),
        "repo suite must keep scheduler-invariants guard"
    );

    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("commands module should exist");
    assert!(
        commands.contains("\"scheduler-invariants\""),
        "foundation verification must require scheduler-invariants"
    );
    assert!(
        commands.contains("DagCommand::SchedulerTimeline"),
        "scheduler timeline surface must remain present"
    );
}
