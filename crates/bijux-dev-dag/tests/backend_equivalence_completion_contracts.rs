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
fn backend_equivalence_specs_and_support_documents_exist() {
    for rel in [
        "docs/spec/BACKEND_EQUIVALENCE_CONTRACT.md",
        "docs/spec/BACKEND_CONTRACT.md",
        "docs/spec/PORTABILITY_GUARANTEES.md",
        "docs/reference/BACKEND_NON_EQUIVALENCES.md",
        "docs/reference/K8S_SUPPORT_MATRIX.md",
        "docs/reference/HPC_SUPPORT_MATRIX.md",
        "docs/reference/REMOTE_SUPPORT_MATRIX.md",
    ] {
        let text = read(rel);
        assert!(!text.trim().is_empty(), "empty backend equivalence surface: {rel}");
    }
}

#[test]
fn backend_equivalence_routes_and_contract_tests_cover_operator_surfaces() {
    let routes = read("crates/bijux-dag-app/src/routes/surface_routes.rs");
    for token in [
        "handle_semantic_portability_command",
        "handle_equivalence_proof_command",
        "unsupported backend target",
        "equivalence proof downgraded due to unsupported backend or semantic divergence",
    ] {
        assert!(
            routes.contains(token),
            "missing backend operator route behavior token: {token}"
        );
    }

    let cli_contract = read("crates/bijux-dag-cli/tests/contract_surface.rs");
    for token in [
        "semantic_portability_backend_query_surface_is_available",
        "equivalence_proof_surface_reports_for_two_runs",
    ] {
        assert!(
            cli_contract.contains(token),
            "missing backend equivalence cli contract token: {token}"
        );
    }

    let runtime_contract =
        read("crates/bijux-dag-runtime/tests/adapter_registry_capability_contracts.rs");
    for token in [
        "adapter_metadata_exclusion_from_graph_identity_is_explicit_in_contracts",
        "adapter_metadata_persistence_contracts_cover_export_import_and_replay_surfaces",
    ] {
        assert!(
            runtime_contract.contains(token),
            "missing backend metadata identity contract token: {token}"
        );
    }
}

#[test]
fn backend_equivalence_corpus_and_stress_suite_are_machine_readable() {
    for rel in [
        "evidence/compat/backend_equivalence/local_vs_k8s.json",
        "evidence/compat/backend_equivalence/local_vs_hpc.json",
        "evidence/compat/backend_equivalence/local_vs_remote.json",
        "evidence/compat/backend_equivalence/k8s_vs_imported_local_replay.json",
        "evidence/compat/backend_equivalence/hpc_vs_imported_local_replay.json",
        "evidence/compat/backend_equivalence/generated_fixture_corpus.json",
        "configs/suites/backend_equivalence_portability_stress.json",
        "docs/reports/foundation/backend_equivalence_quality_benchmark.md",
        "docs/reports/foundation/backend_equivalence_performance_benchmarks.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing backend equivalence artifact: {rel}"
        );
    }

    let corpus: Value = serde_json::from_str(&read(
        "evidence/compat/backend_equivalence/generated_fixture_corpus.json",
    ))
    .expect("parse generated backend equivalence corpus");
    assert_eq!(corpus["version"], "v1");
    let cases = corpus["cases"].as_array().expect("cases");
    assert!(cases.len() >= 8, "expected broad backend equivalence corpus");
    for coverage in [
        "equivalence-proof-output",
        "semantic-portability-reporting",
        "unsupported-backend-rejection",
        "backend-metadata-excluded-from-graph-identity",
        "backend-metadata-included-in-run-identity",
        "cross-backend-replay-compatibility",
        "artifact-integrity-preservation",
        "run-diff-correctness",
    ] {
        assert!(
            cases.iter().any(|case| {
                case["coverage"]
                    .as_array()
                    .is_some_and(|arr| arr.iter().any(|item| item == coverage))
            }),
            "missing backend equivalence coverage class: {coverage}"
        );
    }

    let suite: Value = serde_json::from_str(&read(
        "configs/suites/backend_equivalence_portability_stress.json",
    ))
    .expect("parse backend equivalence stress suite");
    assert_eq!(suite["id"], "backend-equivalence-portability-stress");
    let commands = suite["commands"]
        .as_array()
        .expect("commands")
        .iter()
        .filter_map(|item| item.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    for token in [
        "replay_semantic_surface_contracts",
        "equivalence_proof_surface_reports_for_two_runs",
        "adapter_registry_capability_contracts",
        "backend_equivalence_contracts",
        "backend_equivalence_completion_contracts",
    ] {
        assert!(
            commands.contains(token),
            "missing backend stress suite command token: {token}"
        );
    }
}
