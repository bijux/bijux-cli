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

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn adapter_interface_and_runtime_specs_exist() {
    for rel in [
        "docs/spec/ADAPTER_CONTRACT.md",
        "docs/spec/ADAPTER_INTERFACE_SPEC_v0.1.md",
        "docs/spec/ADAPTER_RUNTIME_CONTRACT_v0.1.md",
        "docs/spec/K8S_ADAPTER_CONTRACT.md",
        "docs/spec/HPC_ADAPTER_CONTRACT.md",
        "docs/spec/WORKER_PROTOCOL_CONTRACT.md",
    ] {
        let body = fs::read_to_string(root().join(rel)).expect("read adapter spec");
        assert!(!body.trim().is_empty(), "empty adapter spec: {rel}");
    }
}

#[test]
fn adapter_regression_corpus_is_machine_readable_and_complete() {
    let payload: Value = serde_json::from_str(
        &fs::read_to_string(
            root().join("evidence/compat/backend_equivalence/adapter_regression_corpus.json"),
        )
        .expect("read corpus"),
    )
    .expect("parse corpus");
    assert_eq!(payload["version"], "v1");
    let cases = payload["cases"].as_array().expect("cases");
    assert!(cases.len() >= 7);
    for coverage in [
        "registry",
        "duplicate_registration",
        "invalid_configuration",
        "metadata_persistence",
        "capability_query",
        "kubernetes",
        "hpc",
        "remote_worker_protocol",
        "compatibility",
        "determinism",
        "concurrency",
        "stress",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|v| v == coverage))
            }),
            "missing coverage class in adapter regression corpus: {coverage}"
        );
    }
}

#[test]
fn runtime_and_dev_tests_anchor_adapter_contract_surfaces() {
    let runtime_registry_tests = fs::read_to_string(
        root().join("crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs"),
    )
    .expect("read runtime adapter tests");
    for token in [
        "adapter_registry_rejects_duplicate_identities_by_reported_list",
        "incomplete_capability_declaration_is_rejected_by_conformance",
        "backend_capability_query_output_stability_for_kubernetes_contract",
        "backend_capability_query_output_stability_for_hpc_contract",
        "adapter_metadata_persistence_contracts_cover_export_import_and_replay_surfaces",
        "adapter_metadata_exclusion_from_graph_identity_is_explicit_in_contracts",
        "hpc_resource_fingerprint_is_stable_for_identical_input",
    ] {
        assert!(
            runtime_registry_tests.contains(token),
            "runtime adapter contract token missing: {token}"
        );
    }

    let remote_worker_tests = fs::read_to_string(
        root().join("crates/bijux-dev-dag/tests/remote_worker_protocol_contracts.rs"),
    )
    .expect("read remote worker tests");
    assert!(
        remote_worker_tests.contains("remote_worker_protocol_conformance_suite_exists"),
        "remote worker protocol conformance anchor missing"
    );
}

#[test]
fn backend_capability_matrix_and_scope_reports_exist() {
    for rel in [
        "docs/reports/foundation/backend_capability_query_reference.md",
        "docs/reports/foundation/backend_capability_matrix.md",
        "docs/reports/foundation/runtime_adapter_registry_coverage_dashboard.md",
    ] {
        let text = fs::read_to_string(root().join(rel)).expect("read backend report");
        assert!(!text.trim().is_empty(), "empty backend report: {rel}");
    }
}
