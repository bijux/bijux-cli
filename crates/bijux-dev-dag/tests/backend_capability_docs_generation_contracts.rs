use bijux_dag_testkit as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn adapter_coverage_matrix_is_generated_and_references_conformance_suites() {
    let path = repo_root().join("docs/reports/foundation/adapter_conformance_coverage_matrix.json");
    let raw = std::fs::read_to_string(path).expect("coverage matrix");
    let value: serde_json::Value = serde_json::from_str(&raw).expect("matrix json");
    assert_eq!(value["format"], "adapter-conformance-coverage/v1");
    assert_eq!(value["generated_from"], "crates/bijux-dag-runtime/tests");
    let rows = value["rows"].as_array().expect("rows");
    assert!(rows.iter().any(|r| r["backend"] == "kubernetes"));
    assert!(rows.iter().any(|r| r["backend"] == "hpc"));
    assert!(rows.iter().any(|r| r["backend"] == "remote"));
}

#[test]
fn capability_query_docs_are_generated_not_handwritten() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/backend_capability_query_reference.md"),
    )
    .expect("capability query docs");
    assert!(raw.contains("generated_from:"));
    assert!(raw.contains("format: `capabilities/v1`"));
    assert!(raw.contains("local"));
    assert!(raw.contains("kubernetes"));
    assert!(raw.contains("hpc"));
    assert!(raw.contains("remote"));
}

#[test]
fn backend_claims_report_links_claims_to_evidence_suites() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/backend_claims_evidence_links.md"),
    )
    .expect("claims report");
    assert!(raw.contains("generated_from:"));
    for token in [
        "k8s_adapter_release_contracts.rs",
        "hpc_adapter_release_contracts.rs",
        "remote_worker_protocol_release_contracts.rs",
    ] {
        assert!(raw.contains(token), "missing evidence token {token}");
    }
}
