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
fn app_service_boundary_contract_and_reports_exist() {
    for rel in [
        "crates/bijux-dag-app/tests/service_boundary_contract.rs",
        "docs/reports/foundation/app_route_to_service_mapping.md",
        "docs/reports/foundation/app_lib_direct_command_helpers.md",
        "docs/reports/foundation/app_modules_zero_direct_tests_report.md",
        "docs/reports/foundation/app_modules_below_50_coverage_report.md",
        "docs/reports/foundation/app_hot_path_quality_dashboard.md",
    ] {
        assert!(
            repo_root().join(rel).exists(),
            "missing app boundary artifact: {rel}"
        );
    }
}
