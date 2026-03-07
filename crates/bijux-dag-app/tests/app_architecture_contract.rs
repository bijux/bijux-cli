use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use flate2 as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read source file")
}

#[test]
fn lib_routes_config_resolution_through_service_layer() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("show_effective_config("),
        "lib should delegate config resolution through service API"
    );
    assert!(
        lib.contains("show_effective_policy("),
        "lib should delegate policy resolution through service API"
    );
    assert!(
        !lib.contains("resolve_effective_config("),
        "lib should not call config merge primitive directly"
    );
}

#[test]
fn lib_routes_run_diff_through_replay_service() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("replay_service::run_diff_from_dirs("),
        "lib should route run diff through replay service"
    );
    assert!(
        !lib.contains("diff::build_run_diff("),
        "lib should not assemble run diff directly"
    );
}

#[test]
fn graph_helpers_use_module_not_include() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("mod graph_helpers;"),
        "graph helper functions should be in a dedicated module"
    );
    assert!(
        !lib.contains("include!(\"graph/helpers"),
        "graph helper inclusion should not rely on include!"
    );
}
