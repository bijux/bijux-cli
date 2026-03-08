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

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
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
    let diff_routes = read("src/routes/diff_routes.rs");
    assert!(
        lib.contains("routes::diff_routes::handle_diff_command("),
        "lib should route run diff through diff routes module"
    );
    assert!(
        diff_routes.contains("replay_service::run_diff_from_dirs("),
        "diff routes should route run diff through replay service"
    );
    assert!(
        !lib.contains("diff::build_run_diff("),
        "lib should not assemble run diff directly"
    );
}

#[test]
fn operator_ux_checklist_doc_is_linked_to_app_boundary_docs() {
    let root = repo_root();
    let checklist = fs::read_to_string(root.join("docs/spec/OPERATOR_UX_CHECKLIST.md"))
        .expect("read operator ux checklist");
    assert!(
        checklist.contains("docs/reports/foundation/app_service_boundary_report.md"),
        "operator ux checklist must link app service boundary report"
    );
    assert!(
        checklist.contains("docs/spec/OPERATOR_UX_CONTRACT.md"),
        "operator ux checklist must link operator ux contract"
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

#[test]
fn lib_uses_diagnostics_route_module_for_operator_diagnostics() {
    let lib = read("src/lib.rs");
    assert!(
        lib.contains("mod routes;"),
        "lib should declare routes module for command-family handlers"
    );
    assert!(
        lib.contains("routes::diagnostics_routes::handle_why_rerun_command("),
        "lib should delegate why-rerun handling to diagnostics routes"
    );
    assert!(
        lib.contains("routes::diagnostics_routes::handle_trace_artifact_command("),
        "lib should delegate trace-artifact handling to diagnostics routes"
    );
}
