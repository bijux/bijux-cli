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
fn backend_contract_and_hardening_report_exist() {
    let root = repo_root();
    for required in [
        "docs/spec/BACKEND_CONTRACT.md",
        "docs/spec/EXECUTION_ENGINE_CONTRACT.md",
        "docs/reference/CONTAINER_REMOTE_EXECUTION_BOUNDARY.md",
        "docs/reference/BATCH_BACKEND_SIMULATION_BOUNDARY.md",
        "docs/reports/foundation/backend_hardening_report.md",
        "crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs",
        "crates/bijux-dag-runtime/tests/execution_backend_contract.rs",
    ] {
        assert!(
            root.join(required).exists(),
            "missing backend hardening surface: {required}"
        );
    }
}

#[test]
fn backend_conformance_tests_cover_lifecycle_and_capability_rules() {
    let root = repo_root();
    let payload = fs::read_to_string(
        root.join("crates/bijux-dag-runtime/tests/execution_backend_contract.rs"),
    )
    .expect("backend contract tests should exist");

    for required in [
        "fake_and_process_like_backends_have_parity_on_basic_scenario",
        "backend_prepare_failures_are_classified_correctly",
        "backend_launch_failures_do_not_corrupt_state",
        "backend_observe_timeout_has_distinct_error",
        "cleanup_runs_after_observe_and_reports_cleanup_failures",
        "cleanup_runs_when_prepare_fails",
        "backend_env_shaping_contract_is_explicitly_applied",
        "backend_output_collection_rejects_undeclared_outputs",
        "backend_registry_includes_capability_descriptors",
    ] {
        assert!(
            payload.contains(required),
            "backend conformance test missing `{required}`"
        );
    }
}

#[test]
fn foundation_and_repo_suites_keep_backend_contract_guard() {
    let root = repo_root();
    let repo_suites = fs::read_to_string(root.join("crates/bijux-dev-dag/src/suites/repo.rs"))
        .expect("repo suite list should exist");
    assert!(
        repo_suites.contains("\"backend-contract\""),
        "repo suites must include backend-contract"
    );

    let commands = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("commands source should exist");
    assert!(
        commands.contains("\"backend-contract\""),
        "foundation verification must require backend-contract"
    );
}
