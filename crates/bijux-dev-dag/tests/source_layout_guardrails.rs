use bijux_dag_runtime as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
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
    let required = ["crates", "docs", "examples", "configs/nextest"];

    for rel in required {
        assert!(root.join(rel).exists(), "missing required path: {rel}");
    }
}
