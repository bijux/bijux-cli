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
use std::path::PathBuf;
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn backend_equivalence_contract_and_support_surfaces_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/BACKEND_EQUIVALENCE_CONTRACT.md",
        "docs/reference/BACKEND_NON_EQUIVALENCES.md",
        "docs/reference/K8S_SUPPORT_MATRIX.md",
        "docs/reference/HPC_SUPPORT_MATRIX.md",
        "docs/reference/REMOTE_SUPPORT_MATRIX.md",
        "evidence/reports/backend_capability_matrix_generated.json",
        "docs/reports/foundation/backend_equivalence_quality_benchmark.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing backend equivalence surface: {rel}"
        );
    }
}

#[test]
fn backend_equivalence_fixtures_exist_for_cross_backend_pairs() {
    let root = repo_root();
    for rel in [
        "evidence/compat/backend_equivalence/local_vs_k8s.json",
        "evidence/compat/backend_equivalence/local_vs_hpc.json",
        "evidence/compat/backend_equivalence/local_vs_remote.json",
        "evidence/compat/backend_equivalence/k8s_vs_imported_local_replay.json",
        "evidence/compat/backend_equivalence/hpc_vs_imported_local_replay.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing backend equivalence fixture: {rel}"
        );
    }
}

#[test]
fn generated_matrix_and_support_docs_are_aligned_on_status() {
    let root = repo_root();
    let matrix: Value = serde_json::from_str(
        &fs::read_to_string(root.join("evidence/reports/backend_capability_matrix_generated.json"))
            .expect("read matrix"),
    )
    .expect("parse matrix");
    let backends = matrix["backends"].as_array().expect("backends array");

    let k8s_doc =
        fs::read_to_string(root.join("docs/reference/K8S_SUPPORT_MATRIX.md")).expect("k8s");
    let hpc_doc =
        fs::read_to_string(root.join("docs/reference/HPC_SUPPORT_MATRIX.md")).expect("hpc");
    let remote_doc =
        fs::read_to_string(root.join("docs/reference/REMOTE_SUPPORT_MATRIX.md")).expect("remote");

    for backend in backends {
        let name = backend["backend"].as_str().expect("backend name");
        let status = backend["status"].as_str().expect("backend status");
        match name {
            "kubernetes" => assert!(
                k8s_doc.contains(status),
                "k8s doc must include status {status}"
            ),
            "hpc" => assert!(
                hpc_doc.contains(status),
                "hpc doc must include status {status}"
            ),
            "remote" => assert!(
                remote_doc.contains(status),
                "remote doc must include status {status}"
            ),
            _ => panic!("unexpected backend in generated matrix: {name}"),
        }
    }
}
