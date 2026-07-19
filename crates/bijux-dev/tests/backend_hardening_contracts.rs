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
fn backend_contract_documents_lifecycle_and_failure_classes() {
    let root = workspace_root();
    let contract =
        fs::read_to_string(root.join("docs/spec/BACKEND_CONTRACT.md")).expect("contract");

    for token in [
        "BackendBindingRequest",
        "BackendCapabilities",
        "BackendError::Prepare",
        "BackendError::Launch",
        "BackendError::ObserveTimeout",
        "BackendError::Finalize",
        "BackendError::Cleanup",
        "fake_and_process_like_backends_have_parity_on_basic_scenario",
    ] {
        assert!(contract.contains(token), "backend contract missing token: {token}");
    }
}

#[test]
fn execution_engine_contract_and_attempt_schema_track_backend_surface() {
    let root = workspace_root();
    let engine_contract = fs::read_to_string(root.join("docs/spec/EXECUTION_ENGINE_CONTRACT.md"))
        .expect("engine contract");
    let attempt_schema =
        fs::read_to_string(root.join("docs/spec/ATTEMPT_TRACE_SCHEMA.md")).expect("attempt schema");

    for token in [
        "execute_with_backend",
        "ExecutionAttemptRecord",
        "prepare -> launch -> observe -> finalize -> cleanup",
        "BackendLifecycleResult",
    ] {
        assert!(
            engine_contract.contains(token),
            "execution engine contract missing token: {token}"
        );
    }

    for token in ["node_id", "attempt", "backend_kind", "status", "exit_code", "EngineOutcome"] {
        assert!(attempt_schema.contains(token), "attempt trace schema missing token: {token}");
    }
}

#[test]
fn backend_hardening_report_links_docs_runtime_and_tests() {
    let root = workspace_root();
    let report =
        fs::read_to_string(root.join("docs/reports/foundation/BACKEND_HARDENING_REPORT.md"))
            .expect("report");

    for token in [
        "docs/spec/BACKEND_CONTRACT.md",
        "docs/spec/EXECUTION_ENGINE_CONTRACT.md",
        "docs/spec/ATTEMPT_TRACE_SCHEMA.md",
        "docs/bijux-dag/architecture/engine-backend-responsibilities.md",
        "crates/bijux-dag-runtime/src/backend/runtime/execution_backend.rs",
        "crates/bijux-dag-runtime/tests/execution_backend_contract.rs",
        "crates/bijux-dag-runtime/tests/engine_flow_contract.rs",
        "crates/bijux-dev/tests/backend_hardening_contracts.rs",
        "declared output targets must be authorized before backend launch",
    ] {
        assert!(report.contains(token), "backend hardening report missing: {token}");
    }
}
