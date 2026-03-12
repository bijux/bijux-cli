#![forbid(unsafe_code)]
//! Runtime identity unity tests across binary and Python bridge entrypoints.

use bijux_cli as _;
use bijux_cli_python as _;
use serde_json as _;
use std::fs;
use std::path::PathBuf;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates dir")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn python_bridge_invokes_same_core_entrypoint_as_binary() {
    let root = workspace_root();
    let bindings = fs::read_to_string(root.join("crates/bijux-cli-python/src/bindings.rs"))
        .expect("read python bindings");
    let bin_main =
        fs::read_to_string(root.join("crates/bijux-cli/src/bin/bijux.rs")).expect("read core bin");

    assert!(bindings.contains("use bijux_cli::api::runtime::{run_app, AppRunResult};"));
    assert!(bindings.contains("normalized_argv"));
    assert!(bindings.contains("match run_app(&argv)"));
    assert!(bin_main.contains("run_cli_from_env()"));
}
