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
use tar as _;
use tempfile as _;
use thiserror as _;

use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn lib_routes_command_families_through_route_modules() {
    let root = repo_root();
    let app_lib =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs")).expect("read app lib");
    for token in [
        "routes::validate_routes::handle_validate_command",
        "routes::command_routes::handle_command_catalog_command",
        "routes::adapter_routes::handle_adapters_command",
        "routes::plan_routes::handle_plan_command",
        "routes::run_routes::handle_run_command",
        "routes::inspect_routes::handle_explain_command",
        "routes::replay_routes::handle_replay_command",
        "routes::diff_routes::handle_diff_command",
        "routes::prove_verify_routes::handle_prove_command",
        "routes::prove_verify_routes::handle_verify_command",
        "routes::export_import_routes::handle_export_command",
        "routes::export_import_routes::handle_import_command",
        "routes::artifact_routes::handle_artifact_inspect_command",
        "routes::inspect_routes::handle_status_command",
        "routes::prove_verify_routes::handle_fsck_command",
        "routes::surface_routes::handle_capabilities_command",
        "routes::surface_routes::handle_semantic_portability_command",
    ] {
        assert!(app_lib.contains(token), "missing route delegation token: {token}");
    }
}

#[test]
fn extracted_route_modules_exist_for_command_families() {
    let root = repo_root();
    for rel in [
        "crates/bijux-dag-app/src/routes/export_import_routes.rs",
        "crates/bijux-dag-app/src/routes/command_routes.rs",
        "crates/bijux-dag-app/src/routes/adapter_routes.rs",
        "crates/bijux-dag-app/src/routes/artifact_routes.rs",
        "crates/bijux-dag-app/src/routes/inspect_routes.rs",
        "crates/bijux-dag-app/src/routes/replay_routes.rs",
        "crates/bijux-dag-app/src/routes/diff_routes.rs",
        "crates/bijux-dag-app/src/routes/plan_routes.rs",
        "crates/bijux-dag-app/src/routes/prove_verify_routes.rs",
        "crates/bijux-dag-app/src/routes/run_routes.rs",
        "crates/bijux-dag-app/src/routes/surface_routes.rs",
        "crates/bijux-dag-app/src/routes/validate_routes.rs",
    ] {
        assert!(root.join(rel).exists(), "missing extracted route module: {rel}");
    }
}
