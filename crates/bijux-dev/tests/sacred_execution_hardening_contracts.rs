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
fn sacred_execution_contract_documents_hook_boundary() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/SACRED_EXECUTION_FLOW.md")).expect("contract");

    for token in [
        "run_materialize_inputs",
        "run_cache_lookup",
        "run_retry_logic",
        "run_write_trace",
        "run_cache_write",
        "resolve_dependencies",
    ] {
        assert!(contract.contains(token), "sacred execution contract missing: {token}");
    }
}

#[test]
fn sacred_execution_report_links_runtime_and_maintainer_surfaces() {
    let root = workspace_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/SACRED_EXECUTION_HARDENING_REPORT.md"),
    )
    .expect("report");

    for token in [
        "docs/spec/SACRED_EXECUTION_FLOW.md",
        "docs/bijux-dag/architecture/runtime-execution-flow.md",
        "crates/bijux-dag-runtime/src/runtime_core/governance/sacred_execution.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine.rs",
        "crates/bijux-dag-runtime/tests/sacred_execution_flow_contracts.rs",
        "crates/bijux-dev/tests/sacred_execution_hardening_contracts.rs",
    ] {
        assert!(report.contains(token), "sacred execution report missing: {token}");
    }
}
