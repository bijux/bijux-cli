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
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn graph_identity_docs_and_schema_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/GRAPH_IDENTITY_CONTRACT.md",
        "docs/spec/FINGERPRINTS_v0.1.md",
        "configs/schema/graph_fingerprint_explain.schema.json",
        "configs/schema/graph_canonical_diff.schema.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing graph identity surface: {rel}"
        );
    }
}

#[test]
fn cli_contract_mentions_hash_graph_and_canonical_diff_surfaces() {
    let root = repo_root();
    let cli_contract =
        fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md")).expect("read cli contract");
    for token in [
        "dag hash graph",
        "dag canonical-diff",
        "dag canonical-bytes",
    ] {
        assert!(
            cli_contract.contains(token),
            "cli contract missing graph identity command token: {token}"
        );
    }
}
