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
fn app_boundary_and_ux_reports_exist() {
    for rel in [
        "docs/reports/foundation/app_service_boundary_report.md",
        "docs/reports/foundation/operator_command_ux_audit.md",
        "docs/spec/OUTPUT_CONCISION_CONTRACT.md",
        "docs/reports/foundation/json_consistency_sweep.md",
        "docs/reports/foundation/app_route_coverage_by_command_family.md",
        "docs/reports/foundation/app_response_schema_coverage_by_command_family.md",
        "docs/reports/foundation/app_route_to_service_mapping.md",
        "docs/reports/foundation/app_service_boundary_completion_report.md",
        "docs/reports/foundation/app_hot_path_quality_dashboard.md",
        "docs/examples/operator_command_examples.md",
        "docs/reports/foundation/app_inspect_explain_latency_baseline.md",
        "crates/bijux-dag-app/tests/route_entrypoint_no_panic_contract.rs",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}
