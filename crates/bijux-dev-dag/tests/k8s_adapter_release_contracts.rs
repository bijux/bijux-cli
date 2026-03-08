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

#[test]
fn kubernetes_fixture_corpus_exists_and_is_parseable() {
    let fixtures = [
        "evidence/battle/fixtures/kubernetes/tiny_equivalence.dag.json",
        "evidence/battle/fixtures/kubernetes/medium_fanout.dag.json",
        "evidence/battle/fixtures/kubernetes/failure_injection_image_pull_backoff.dag.json",
    ];

    for rel in fixtures {
        let raw = bijux_dag_testkit::load_graph_fixture_json(env!("CARGO_MANIFEST_DIR"), rel);
        let parsed: Result<bijux_dag_core::Graph, _> = serde_json::from_value(raw);
        assert!(
            parsed.is_ok(),
            "kubernetes fixture must parse as graph: {rel}"
        );
    }
}

#[test]
fn kubernetes_conformance_reports_and_support_matrix_are_present() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    for rel in [
        "docs/reports/foundation/k8s_conformance_gate_report.md",
        "docs/reports/foundation/k8s_replay_env_drift_report.md",
        "docs/reports/foundation/k8s_adapter_benchmarks.md",
        "docs/reference/K8S_SUPPORT_MATRIX.md",
        "evidence/battle/fixtures/kubernetes/k8s_vs_local_run_diff.json",
        "evidence/operator/fixtures/kubernetes_pod_failure_explain.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing kubernetes release contract surface: {rel}"
        );
    }
}

#[test]
fn deployment_and_support_docs_do_not_overclaim_kubernetes_execution() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root");
    let support = fs::read_to_string(root.join("docs/reference/EXECUTION_SUPPORT_POLICY.md"))
        .expect("read support policy");
    assert!(
        support.contains("| kubernetes | simulated |"),
        "support policy must keep kubernetes in simulated status"
    );

    let matrix = fs::read_to_string(root.join("docs/reference/K8S_SUPPORT_MATRIX.md"))
        .expect("read support matrix");
    assert!(
        matrix.contains("Kubernetes execution backend | simulated"),
        "k8s support matrix must keep execution backend simulated"
    );
}
