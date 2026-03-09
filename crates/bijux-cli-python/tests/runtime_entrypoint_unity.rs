#![forbid(unsafe_code)]
//! Runtime identity unity tests across binary and Python bridge entrypoints.

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use serde_json as _;
use std::fs;
use std::path::PathBuf;
use thiserror as _;

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
    let bin_main = fs::read_to_string(root.join("crates/bijux-cli-bin/src/main.rs"))
        .expect("read bin main");

    assert!(bindings.contains("use bijux_cli_core::app::{run_app, AppRunResult};"));
    assert!(bindings.contains("match run_app(argv)"));
    assert!(bin_main.contains("run_app(&argv)"));
}
