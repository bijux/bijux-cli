use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn app_router_post_extraction_reports_and_adr_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/app_route_responsibility_report.md",
        "docs/reports/foundation/app_lib_residual_responsibility_report.md",
        "docs/reports/foundation/app_route_coupling_report.md",
        "docs/reports/foundation/app_route_import_graph.md",
        "docs/reports/foundation/app_router_post_extraction_completion_report.md",
        "docs/adr/20260308-app-routing-post-extraction-end-state.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing app router extraction artifact {rel}"
        );
    }
}

#[test]
fn app_lib_avoids_remaining_capability_branching_logic() {
    let root = repo_root();
    let lib = fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs")).expect("read lib");
    for forbidden in [
        "backend_capability_payload(&backend_a)",
        "equivalence proof downgraded due to unsupported backend",
        "run_tree_for_id(",
        "run_timeline_for_id(",
    ] {
        assert!(
            !lib.contains(forbidden),
            "lib.rs still contains route business logic token: {forbidden}"
        );
    }
}

#[test]
fn route_modules_own_key_dispatch_and_payload_paths() {
    let root = repo_root();
    let surface =
        fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/surface_routes.rs"))
            .expect("read surface routes");
    let runs = fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/runs_routes.rs"))
        .expect("read runs routes");
    let response = fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/response.rs"))
        .expect("read response routes");
    let renderer = fs::read_to_string(root.join("crates/bijux-dag-app/src/routes/renderer.rs"))
        .expect("read renderer routes");

    assert!(
        surface.contains("handle_equivalence_proof_command"),
        "surface routes must own equivalence proof routing behavior"
    );
    assert!(
        runs.contains("RunCommands::Tree") && runs.contains("RunCommands::Timeline"),
        "runs routes must own tree/timeline command dispatch"
    );
    assert!(
        response.contains("simple_failure_payload"),
        "response module must own JSON envelope helpers"
    );
    assert!(
        renderer.contains("render") || renderer.contains("format"),
        "renderer module must own text formatting helpers"
    );
}
