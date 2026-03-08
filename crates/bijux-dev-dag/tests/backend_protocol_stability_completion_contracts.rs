use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json::Value;
use sha2 as _;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

fn read(rel: &str) -> String {
    fs::read_to_string(repo_root().join(rel)).expect("read file")
}

#[test]
fn backend_protocol_contract_and_reports_exist() {
    for rel in [
        "docs/spec/BACKEND_PROTOCOL_STABILITY_CONTRACT.md",
        "docs/reports/foundation/backend_protocol_benchmarks.md",
        "docs/reports/foundation/backend_protocol_telemetry_report.md",
        "docs/reports/foundation/backend_protocol_anomaly_report.md",
        "docs/reports/foundation/backend_protocol_coverage_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty backend protocol artifact: {rel}"
        );
    }
}

#[test]
fn backend_protocol_corpus_and_suite_are_machine_readable() {
    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/backend_protocol/regression_corpus.json",
    ))
    .expect("parse backend protocol corpus");
    assert_eq!(corpus["version"], "v1");

    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 18, "expected broad backend protocol corpus");

    for coverage in [
        "handshake",
        "version-negotiation",
        "compatibility",
        "error-propagation",
        "timeout-handling",
        "retry-logic",
        "message-ordering",
        "corruption-detection",
        "serialization-schema",
        "replay-safety",
        "regression-fixtures",
        "stress",
        "latency-benchmarks",
        "resilience-benchmarks",
        "telemetry",
        "determinism",
        "fuzzing",
        "anomaly-detection",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing backend protocol coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/backend_protocol_verification.json"))
            .expect("parse backend protocol suite");
    assert_eq!(suite["id"], "backend-protocol-verification");
}

#[test]
fn backend_protocol_surfaces_anchor_existing_worker_and_distributed_tests() {
    let remote_worker = read("crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs");
    for token in [
        "worker_protocol_contract_doc_exists_with_required_sections",
        "distributed_runtime_contract_tests_cover_worker_protocol_semantics",
        "remote_worker_protocol_conformance_suite_exists",
    ] {
        assert!(
            remote_worker.contains(token),
            "missing remote worker protocol anchor token: {token}"
        );
    }

    let distributed = read("crates/bijux-dag-runtime/tests/distributed_contracts.rs");
    for token in [
        "duplicate_dispatch_and_lost_lease_recovery_contracts_hold",
        "status_event_ordering_and_duplicate_ack_resilience_are_explicit",
    ] {
        assert!(
            distributed.contains(token),
            "missing distributed protocol anchor token: {token}"
        );
    }

    let distributed_completion =
        read("crates/bijux-dev-dag/tests/distributed_execution_completion_contracts.rs");
    assert!(
        distributed_completion.contains("distributed-determinism"),
        "missing distributed determinism coverage anchor"
    );
}
