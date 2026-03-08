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

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn app_route_support_fast_suite_is_defined() {
    let root = repo_root();
    let suite = root.join("configs/suites/app_route_support_fast.json");
    assert!(suite.exists(), "missing app route-support fast suite");

    let payload: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&suite).expect("read suite"))
            .expect("parse suite");
    assert_eq!(payload["id"], "app-route-support-fast");
    let commands = payload["commands"].as_array().expect("commands array");
    for required in [
        "routes::output_selection::tests",
        "routes::response::tests",
        "routes::run_lookup::tests",
    ] {
        assert!(
            commands
                .iter()
                .filter_map(|v| v.as_str())
                .any(|cmd| cmd.contains(required)),
            "app route-support fast suite missing {required}"
        );
    }
}
