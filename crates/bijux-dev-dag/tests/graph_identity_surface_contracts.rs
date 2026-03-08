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
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn graph_and_node_identity_contract_docs_exist_and_link_to_sources() {
    let root = repo_root();
    let graph_doc = fs::read_to_string(root.join("docs/spec/GRAPH_IDENTITY_CONTRACT.md"))
        .expect("read graph identity contract");
    let node_doc = fs::read_to_string(root.join("docs/spec/NODE_IDENTITY_CONTRACT.md"))
        .expect("read node identity contract");

    assert!(graph_doc.contains("GraphId"));
    assert!(graph_doc.contains("crates/bijux-dag-core/src/lib.rs"));
    assert!(graph_doc.contains("graph/canonical.rs"));
    assert!(graph_doc.contains("analysis/fingerprint.rs"));

    assert!(node_doc.contains("node.id"));
    assert!(node_doc.contains("graph/topology.rs"));
    assert!(node_doc.contains("pipeline/validate.rs"));
}

#[test]
fn fingerprint_explain_and_canonical_bytes_surfaces_remain_documented_and_schema_backed() {
    let root = repo_root();
    let cli_contract =
        fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md")).expect("read CLI contract");
    assert!(
        cli_contract.contains("dag fingerprint") && cli_contract.contains("dag canonical-bytes"),
        "CLI contract must keep fingerprint explain and canonical bytes command surfaces"
    );

    assert!(
        root.join("configs/schema/graph_fingerprint_explain.schema.json")
            .exists(),
        "graph fingerprint explain schema must exist"
    );
    assert!(
        root.join("configs/schema/execution_plan.schema.json")
            .exists(),
        "execution plan schema must exist"
    );
}
