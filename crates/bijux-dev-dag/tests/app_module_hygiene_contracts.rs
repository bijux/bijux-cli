use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
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
fn app_hygiene_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/adr/20260308-app-router-final-end-state.md",
        "docs/spec/GRAPH_INPUT_READING_RESPONSIBILITIES.md",
        "docs/reports/foundation/app_router_remaining_top_level_responsibilities.md",
        "docs/reports/foundation/app_route_weak_modules_completion_report.md",
        "docs/reports/foundation/app_sub_ten_line_module_inventory.md",
        "docs/reports/foundation/app_no_dead_module_report.md",
        "docs/reports/foundation/app_no_unreferenced_response_module_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing report/doc: {rel}");
    }
}

#[test]
fn app_zero_coverage_allowlist_entries_are_blocked() {
    let root = repo_root();
    let payload: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/protected_zero_coverage_allowlist.json"))
            .expect("read allowlist"),
    )
    .expect("parse allowlist");

    let allow = payload["protected_zero_coverage_allowlist"]
        .as_array()
        .expect("allowlist array");
    let mut app_entries: Vec<&str> = allow
        .iter()
        .filter_map(|v| v.as_str())
        .filter(|p| p.starts_with("crates/bijux-dag-app/src/"))
        .collect();
    app_entries.sort_unstable();
    let baseline = vec![
        "crates/bijux-dag-app/src/commands/config_resolution.rs",
        "crates/bijux-dag-app/src/commands/config_surface.rs",
        "crates/bijux-dag-app/src/inspect/mod.rs",
        "crates/bijux-dag-app/src/routes/output_selection.rs",
        "crates/bijux-dag-app/src/routes/plan_routes.rs",
        "crates/bijux-dag-app/src/routes/renderer.rs",
        "crates/bijux-dag-app/src/routes/response.rs",
        "crates/bijux-dag-app/src/routes/run_lookup.rs",
    ];
    assert_eq!(
        app_entries, baseline,
        "release gate violated: app zero-coverage allowlist drifted from baseline: {app_entries:?}"
    );
}

#[test]
fn app_disallows_one_line_wrapper_modules() {
    let root = repo_root();
    let mut files = Vec::new();
    collect_rust_files(&root.join("crates/bijux-dag-app/src"), &mut files);
    let mut offenders = Vec::new();
    for file in files {
        let rel = file
            .strip_prefix(&root)
            .expect("file under repo root")
            .to_string_lossy()
            .to_string();
        if rel.ends_with("mod.rs") {
            continue;
        }
        let content = fs::read_to_string(&file).expect("read rust source");
        let meaningful = content
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("#!["))
            .count();
        if meaningful <= 1 {
            offenders.push(rel);
        }
    }

    assert!(
        offenders.is_empty(),
        "one-line wrapper modules are disallowed in app crate: {offenders:?}"
    );
}

#[test]
fn app_lib_file_size_budget_is_enforced() {
    let root = repo_root();
    let lib_path = root.join("crates/bijux-dag-app/src/lib.rs");
    let content = fs::read_to_string(&lib_path).expect("read app lib");
    let line_count = content.lines().count();
    let line_budget = 2600usize;
    assert!(
        line_count <= line_budget,
        "app lib line budget exceeded: lines={line_count} budget={line_budget}"
    );
}

#[test]
fn app_route_file_size_ceilings_are_enforced() {
    let root = repo_root();
    let inspect_path = root.join("crates/bijux-dag-app/src/routes/inspect_routes.rs");
    let plan_path = root.join("crates/bijux-dag-app/src/routes/plan_routes.rs");
    let inspect_lines = fs::read_to_string(&inspect_path)
        .expect("read inspect routes")
        .lines()
        .count();
    let plan_lines = fs::read_to_string(&plan_path)
        .expect("read plan routes")
        .lines()
        .count();
    assert!(
        inspect_lines <= 380,
        "inspect_routes.rs line budget exceeded: lines={inspect_lines} budget=380"
    );
    assert!(
        plan_lines <= 260,
        "plan_routes.rs line budget exceeded: lines={plan_lines} budget=260"
    );
}

#[test]
fn app_router_dispatch_stays_in_route_modules_for_key_families() {
    let root = repo_root();
    let lib = fs::read_to_string(root.join("crates/bijux-dag-app/src/lib.rs")).expect("read app lib");
    let required_delegations = [
        "Commands::Plan { command } => routes::plan_routes::handle_plan_command(&cli, command)",
        "Commands::Explain { run_dir, node } => {",
        "routes::inspect_routes::handle_explain_command(&cli, run_dir, node)",
        "Commands::WhyRerun { run_a, run_b } => {",
        "routes::diagnostics_routes::handle_why_rerun_command(&cli, run_a, run_b)",
        "Commands::TraceArtifact { run_dir, artifact_id } => {",
        "routes::diagnostics_routes::handle_trace_artifact_command(&cli, run_dir, artifact_id)",
        "Commands::Capabilities { backend } => {",
        "routes::surface_routes::handle_capabilities_command(&cli, backend)",
        "Commands::SemanticPortability { backend } => {",
        "routes::surface_routes::handle_semantic_portability_command(&cli, backend)",
    ];
    for snippet in required_delegations {
        assert!(
            lib.contains(snippet),
            "router dispatch missing required delegation snippet: {snippet}"
        );
    }
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).expect("read app src dir") {
        let entry = entry.expect("read dir entry");
        let path = entry.path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}
