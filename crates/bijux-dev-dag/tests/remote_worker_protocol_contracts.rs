use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn worker_protocol_contract_doc_exists_with_required_sections() {
    let root = repo_root();
    let path = root.join("docs/spec/WORKER_PROTOCOL_CONTRACT.md");
    assert!(path.exists(), "missing worker protocol contract doc");
    let text = fs::read_to_string(path).expect("read worker protocol contract");

    for token in [
        "Task lease semantics",
        "Heartbeat semantics",
        "Duplicate dispatch prevention",
        "Worker identity model",
        "Artifact upload and commit semantics",
        "Status event ordering guarantees",
        "Cancellation delivery semantics",
        "Worker version and capability negotiation",
    ] {
        assert!(
            text.contains(token),
            "worker protocol contract missing required section: {token}"
        );
    }
}

#[test]
fn distributed_runtime_contract_tests_cover_worker_protocol_semantics() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dag-runtime/tests/distributed_contracts.rs"))
        .expect("read distributed contract tests");

    for token in [
        "validate_task_lease_semantics",
        "classify_heartbeat",
        "is_duplicate_dispatch",
        "validate_worker_identity",
        "artifact_upload_can_commit",
        "verify_remote_artifact_integrity",
        "normalize_status_events",
        "cancellation_delivered_in_time",
        "reject_worker_version_mismatch",
        "worker_pool_satisfies_capability_request",
    ] {
        assert!(
            source.contains(token),
            "distributed contract tests missing worker protocol token: {token}"
        );
    }
}

#[test]
fn remote_worker_protocol_conformance_suite_exists() {
    let root = repo_root();
    let path = root.join("crates/bijux-dag-runtime/tests/remote_worker_protocol_conformance.rs");
    assert!(path.exists(), "missing remote worker conformance suite");
    let source = fs::read_to_string(path).expect("read conformance suite");
    for token in [
        "conformance_heartbeat_classification_is_stable",
        "conformance_duplicate_dispatch_and_event_dedup_hold",
        "conformance_upload_commit_version_gate_and_capability_negotiation_hold",
    ] {
        assert!(
            source.contains(token),
            "remote worker conformance suite missing token: {token}"
        );
    }
}
