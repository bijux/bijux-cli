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

#[test]
fn repository_layout_contains_required_roots() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let required = ["crates", "docs", "evidence", "configs", "make"];

    for rel in required {
        assert!(root.join(rel).exists(), "missing required path: {rel}");
    }
}

#[test]
fn repository_proof_roots_remain_concentrated_in_evidence() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let forbidden_proof_roots = ["examples", "benchmarks", "comparisons"];
    for rel in forbidden_proof_roots {
        assert!(
            !root.join(rel).exists(),
            "forbidden proof root is present; evidence must remain sole proof pillar: {rel}"
        );
    }
}

#[test]
fn repository_root_does_not_contain_target_directory() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    assert!(
        !root.join("target").exists(),
        "root target directory is forbidden; use artifacts/target via CARGO_TARGET_DIR"
    );
}
