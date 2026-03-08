use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::Path;
use tempfile as _;

fn repo_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn app_hot_path_fast_suite_report_exists_and_lists_members() {
    let raw = std::fs::read_to_string(
        repo_root().join("docs/reports/foundation/app_hot_path_fast_suite.md"),
    )
    .expect("read fast suite report");
    assert!(raw.contains("generated_from:"));
    for token in [
        "help_surface_contracts.rs",
        "command_surface_routing_contracts.rs",
        "operator_malformed_input_no_panic_contracts.rs",
    ] {
        assert!(raw.contains(token), "missing suite member token {token}");
    }
}
