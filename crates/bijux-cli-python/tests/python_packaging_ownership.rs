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
fn workspace_root_pyproject_is_not_runtime_distribution() {
    let root = workspace_root();
    let root_pyproject =
        fs::read_to_string(root.join("pyproject.toml")).expect("read workspace pyproject");
    assert!(
        root_pyproject.contains("name = \"bijux-cli-workspace-tools\""),
        "root pyproject must stay tooling-only and avoid canonical distribution name",
    );
    assert!(
        !root_pyproject.contains("bijux = \"bijux_cli.core.bootstrap:main\""),
        "root pyproject must not define legacy bijux runtime console script",
    );
}
