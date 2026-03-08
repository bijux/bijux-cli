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
fn distributed_execution_specs_and_reports_exist() {
    for rel in [
        "docs/spec/DISTRIBUTED_EXECUTION_ARCHITECTURE_CONTRACT.md",
        "docs/spec/WORKER_PROTOCOL_CONTRACT.md",
        "docs/spec/REMOTE_EXECUTION_MODEL.md",
        "docs/spec/DISTRIBUTED_COORDINATION_MODEL.md",
        "docs/reports/foundation/distributed_execution_benchmarks.md",
        "docs/reports/foundation/distributed_execution_telemetry_report.md",
    ] {
        let body = read(rel);
        assert!(
            !body.trim().is_empty(),
            "empty distributed execution surface: {rel}"
        );
    }
}

#[test]
fn distributed_execution_corpus_and_stress_suite_are_machine_readable() {
    for rel in [
        "evidence/cache/distributed_execution/regression_corpus.json",
        "configs/suites/distributed_execution_stress.json",
        "evidence/battle/fixtures/remote/simple_worker_pool.dag.json",
        "evidence/battle/fixtures/remote/worker_protocol_failure_injection.json",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing distributed execution artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read(
        "evidence/cache/distributed_execution/regression_corpus.json",
    ))
    .expect("parse distributed corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 14, "expected distributed corpus breadth");
    for coverage in [
        "worker-registration",
        "identity-preservation",
        "capability-reporting",
        "task-dispatch",
        "task-completion",
        "failure-reporting",
        "timeout-detection",
        "retry-scheduling",
        "artifact-upload",
        "artifact-download",
        "replay-compatibility",
        "provenance-preservation",
        "network-failure",
        "latency-tolerance",
        "distributed-determinism",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing distributed coverage class: {coverage}"
        );
    }

    let suite: Value =
        serde_json::from_str(&read("configs/suites/distributed_execution_stress.json"))
            .expect("parse distributed suite");
    assert_eq!(suite["id"], "distributed-execution-stress");
}

#[test]
fn runtime_and_dev_tests_anchor_distributed_execution_contracts() {
    let runtime = read("crates/bijux-dag-runtime/tests/distributed_contracts.rs");
    for token in [
        "submit_distributed",
        "worker_liveness_and_reassignment_follow_contract",
        "status_event_ordering_and_duplicate_ack_resilience_are_explicit",
    ] {
        assert!(
            runtime.contains(token),
            "missing runtime distributed token: {token}"
        );
    }

    let worker_protocol = read("crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs");
    assert!(
        worker_protocol.contains("distributed_runtime_contract_tests_cover_worker_protocol_semantics"),
        "missing remote worker protocol anchor token"
    );

    let worker_release =
        read("crates/bijux-dev-dag/tests/remote_worker_protocol_release_contracts.rs");
    assert!(
        worker_release.contains("execution support policy must keep remote distributed in simulated status"),
        "missing remote worker release anchor token"
    );
}
