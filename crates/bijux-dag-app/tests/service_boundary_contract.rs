use base64 as _;
use bijux_dag_app as _;
use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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

#[test]
fn lib_dispatches_command_families_through_route_modules() {
    let lib = read("src/lib.rs");
    for required in [
        "routes::inspect_routes::handle_explain_command",
        "routes::replay_routes::handle_replay_command",
        "routes::diff_routes::handle_diff_command",
        "routes::export_import_routes::handle_export_command",
        "routes::export_import_routes::handle_import_command",
        "routes::surface_routes::handle_capabilities_command",
        "routes::surface_routes::handle_semantic_portability_command",
    ] {
        assert!(
            lib.contains(required),
            "lib command dispatch missing required route delegation: {required}"
        );
    }
}

#[test]
fn route_modules_stay_within_service_boundaries() {
    let inspect_routes = read("src/routes/inspect_routes.rs");
    let replay_routes = read("src/routes/replay_routes.rs");
    let diff_routes = read("src/routes/diff_routes.rs");
    let export_import_routes = read("src/routes/export_import_routes.rs");

    assert!(
        !inspect_routes.contains("replay_service::"),
        "inspect routes must not call replay service directly"
    );
    assert!(
        replay_routes.contains("replay_service::"),
        "replay routes must route replay behavior through replay service helpers"
    );
    assert!(
        diff_routes.contains("replay_service::run_diff_from_dirs("),
        "diff routes must route through replay diff service"
    );
    assert!(
        !export_import_routes.contains("replay_service::"),
        "export/import routes must not call replay service directly"
    );
}
