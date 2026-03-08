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

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn app_weak_routes_fast_suite_is_defined_and_reported() {
    let root = repo_root();
    let suite = root.join("configs/suites/app_weak_routes_fast.json");
    assert!(suite.exists(), "missing app weak routes fast suite");
    let report = root.join("docs/reports/foundation/app_weak_routes_fast_suite.md");
    assert!(report.exists(), "missing app weak routes fast suite report");

    let suite_raw = fs::read_to_string(&suite).expect("read suite");
    let payload: serde_json::Value = serde_json::from_str(&suite_raw).expect("parse suite");
    assert_eq!(payload["id"], "app-weak-routes-fast");
    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "routes::inspect_routes::tests",
        "routes::plan_routes::tests",
        "routes::diagnostics_routes::tests",
        "routes::output_selection::tests",
        "routes::surface_routes::tests",
    ] {
        assert!(
            commands
                .iter()
                .filter_map(|v| v.as_str())
                .any(|cmd| cmd.contains(required)),
            "suite missing required command filter: {required}"
        );
    }
}
