use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::fs;
use std::path::Path;
use tempfile as _;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn app_lib_dispatches_hot_command_families_to_route_modules() {
    let root = repo_root();
    let lib = fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs")).expect("read lib");
    for snippet in [
        "Commands::Plan { command } => routes::plan_routes::handle_plan_command",
        "Commands::Replay",
        "routes::replay_routes::handle_replay_command",
        "Commands::Diff",
        "routes::diff_routes::handle_diff_command",
        "Commands::Explain",
        "routes::inspect_routes::handle_explain_command",
        "Commands::Capabilities",
        "routes::surface_routes::handle_capabilities_command",
        "Commands::ArtifactInspect",
        "routes::artifact_routes::handle_artifact_inspect_command",
        "Commands::Runs { command } => routes::runs_routes::handle_runs_command",
    ] {
        assert!(
            lib.contains(snippet),
            "missing route delegation snippet: {snippet}"
        );
    }
}

#[test]
fn renderers_do_not_read_filesystem_state() {
    let root = repo_root();
    let renderer =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/renderer.rs")).expect("read");
    for forbidden in ["std::fs", "fs::", "read_to_string", "read_dir", "metadata("] {
        assert!(
            !renderer.contains(forbidden),
            "renderer must not read filesystem state: {forbidden}"
        );
    }
}

#[test]
fn response_builders_do_not_read_filesystem_state() {
    let root = repo_root();
    let response =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/response.rs")).expect("read");
    for forbidden in ["std::fs", "fs::", "read_to_string", "read_dir", "metadata("] {
        assert!(
            !response.contains(forbidden),
            "response builder must not read filesystem state: {forbidden}"
        );
    }
}

#[test]
fn app_router_architecture_reports_and_adr_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/app_route_responsibilities_report.md",
        "docs/reports/foundation/app_route_business_logic_residue_report.md",
        "docs/reports/foundation/app_route_complexity_score_report.md",
        "docs/reports/foundation/app_module_dependency_graph_report.md",
        "docs/adr/20260308-app-routing-shape.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing architecture artifact: {rel}"
        );
    }
}
