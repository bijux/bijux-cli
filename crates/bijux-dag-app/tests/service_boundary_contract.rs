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
use std::fs;
use tar as _;
use tempfile as _;
use thiserror as _;

use std::path::Path;

fn read(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).expect("read source")
}

#[test]
fn inspect_routes_do_not_call_replay_or_config_services() {
    let inspect_routes = read("src/routes/inspect_routes.rs");
    assert!(
        !inspect_routes.contains("replay_service::"),
        "inspect routes must not call replay service directly"
    );
    assert!(
        !inspect_routes.contains("show_effective_config("),
        "inspect routes must not call config resolution directly"
    );
}

#[test]
fn replay_diff_routes_call_replay_service_for_diff_logic() {
    let diff_routes = read("src/routes/diff_routes.rs");
    assert!(
        diff_routes.contains("replay_service::run_diff_from_dirs("),
        "diff routes must call replay service diff helper"
    );
}

#[test]
fn lib_routes_config_resolution_through_config_helpers_only() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("show_effective_config(") && lib.contains("show_effective_policy("),
        "lib must route config/policy through config helpers"
    );
    assert!(
        !lib.contains("resolve_effective_config("),
        "lib must not call low-level config merge directly"
    );
}
