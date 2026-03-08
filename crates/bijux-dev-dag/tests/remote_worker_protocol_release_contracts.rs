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
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn remote_worker_protocol_release_surfaces_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/WORKER_PROTOCOL_CONTRACT.md",
        "docs/spec/REMOTE_DELIVERY_GUARANTEES.md",
        "docs/reports/foundation/remote_worker_adapter_benchmarks.md",
        "docs/reports/foundation/remote_worker_proof_example.md",
        "docs/reports/foundation/remote_worker_protocol_conformance_gate_report.md",
        "evidence/battle/fixtures/remote/simple_worker_pool.dag.json",
        "evidence/battle/fixtures/remote/fanout_many_small_nodes.dag.json",
        "evidence/battle/fixtures/remote/worker_protocol_failure_injection.json",
        "evidence/operator/fixtures/remote/worker_version_mismatch_explain.json",
        "crates/bijux-dag-runtime/tests/distributed_contracts.rs",
        "crates/bijux-dag-runtime/tests/remote_worker_protocol_conformance.rs",
        "crates/bijux-dag-app/tests/run_dir_import_export_contract.rs",
        "crates/bijux-dag-cli/tests/contract_surface.rs",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing remote worker protocol release surface: {rel}"
        );
    }
}

#[test]
fn remote_capabilities_surface_remains_simulated() {
    let root = repo_root();
    let cli_doc = fs::read_to_string(root.join("docs/CLI.md")).expect("read cli doc");
    assert!(
        cli_doc.contains("bijux dag capabilities --backend remote --json"),
        "cli docs must list remote backend capabilities query"
    );

    let support = fs::read_to_string(root.join("docs/reference/EXECUTION_SUPPORT_POLICY.md"))
        .expect("read support policy");
    assert!(
        support.contains("| remote distributed | simulated |"),
        "execution support policy must keep remote distributed in simulated status"
    );
}

#[test]
fn release_gate_report_references_required_remote_conformance_suites() {
    let root = repo_root();
    let report = fs::read_to_string(
        root.join("docs/reports/foundation/remote_worker_protocol_conformance_gate_report.md"),
    )
    .expect("read remote conformance gate report");

    for token in [
        "remote_worker_protocol_conformance.rs",
        "run_dir_import_export_contract.rs",
        "remote_worker_protocol_contracts.rs",
        "remote_worker_protocol_release_contracts.rs",
    ] {
        assert!(
            report.contains(token),
            "remote release gate report missing required suite token: {token}"
        );
    }
}
