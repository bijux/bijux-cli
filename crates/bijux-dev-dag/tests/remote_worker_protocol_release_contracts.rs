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
fn remote_worker_protocol_release_surfaces_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/WORKER_PROTOCOL_CONTRACT.md",
        "crates/bijux-dag-runtime/tests/distributed_contracts.rs",
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
