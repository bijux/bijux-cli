//! Packaging ownership contracts for the Python bridge distribution.

use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn canonical_distribution_is_owned_by_python_bridge_crate() {
    let root = workspace_root();
    let crate_pyproject = fs::read_to_string(root.join("crates/bijux-cli-python/pyproject.toml"))
        .expect("read crate pyproject");
    assert!(
        crate_pyproject.contains("name = \"bijux-cli\""),
        "crate-local pyproject must own canonical distribution name",
    );
    assert!(
        crate_pyproject.contains("bijux = \"bijux_cli_py.cli:main\""),
        "crate-local pyproject must own bijux console script",
    );
}

#[test]
fn workspace_root_has_no_python_distribution_pyproject() {
    let root = workspace_root();
    assert!(
        !root.join("pyproject.toml").exists(),
        "workspace root must not own a Python distribution pyproject",
    );
    assert!(
        !root.join("configs/python/pyproject.toml").exists(),
        "configs/python must not own package metadata after crate-local consolidation",
    );
}
