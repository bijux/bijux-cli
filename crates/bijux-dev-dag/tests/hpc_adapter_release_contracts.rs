use bijux_dag_testkit as _;
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
fn hpc_fixture_corpus_exists_and_is_parseable() {
    let root = repo_root();
    let graph_fixtures = [
        "evidence/battle/fixtures/hpc/simple_equivalence.dag.json",
        "evidence/battle/fixtures/hpc/staged_input_equivalence.dag.json",
        "evidence/battle/fixtures/hpc/checkpointed_partial_replay.dag.json",
    ];
    for rel in graph_fixtures {
        let path = root.join(rel);
        assert!(path.exists(), "missing hpc graph fixture: {rel}");
        let raw = fs::read_to_string(&path).expect("read hpc graph fixture");
        let parsed: Result<bijux_dag_core::Graph, _> = serde_json::from_str(&raw);
        assert!(parsed.is_ok(), "hpc graph fixture must parse: {rel}");
    }

    for rel in [
        "evidence/battle/fixtures/hpc/delayed_scheduler_state_propagation.json",
        "evidence/operator/fixtures/hpc/queue_rejection_explain.json",
        "evidence/operator/fixtures/hpc/preemption_explain.json",
    ] {
        assert!(root.join(rel).exists(), "missing hpc fixture: {rel}");
    }
}

#[test]
fn hpc_reports_and_support_matrix_are_present() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/hpc_adapter_benchmarks.md",
        "docs/reports/foundation/hpc_conformance_gate_report.md",
        "docs/reports/foundation/hpc_replay_scheduler_drift_report.md",
        "docs/reports/foundation/hpc_proof_example.md",
        "docs/reference/HPC_SUPPORT_MATRIX.md",
        "docs/reference/HPC_AND_DNA_BOUNDARY.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing hpc release contract surface: {rel}"
        );
    }
}

#[test]
fn support_docs_keep_hpc_execution_in_simulated_status() {
    let root = repo_root();
    let support = fs::read_to_string(root.join("docs/reference/EXECUTION_SUPPORT_POLICY.md"))
        .expect("read support policy");
    assert!(
        support.contains("| batch/HPC | simulated |"),
        "support policy must keep hpc in simulated status"
    );

    let matrix = fs::read_to_string(root.join("docs/reference/HPC_SUPPORT_MATRIX.md"))
        .expect("read hpc support matrix");
    assert!(
        matrix.contains("HPC execution backend | simulated"),
        "hpc support matrix must keep execution backend simulated"
    );
}
