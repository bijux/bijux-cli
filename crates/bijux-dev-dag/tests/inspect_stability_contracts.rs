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

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
}

#[test]
fn inspect_321_340_status_report_exists_and_covers_required_sections() {
    let report = root().join("docs/reports/foundation/inspect_321_340_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "321-334 inspect determinism, corrupted/missing/partial/replay/imported behavior, and lineage checks",
        "335-336 diagnostics and stability reports",
        "337 inspect verification suite",
        "338 inspect operator smoke tests",
        "339 inspect diagnostics dashboard",
        "340 ADR",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn inspect_321_340_governance_artifacts_exist() {
    for rel in [
        "docs/reports/foundation/inspect_321_340_status_report.md",
        "docs/reports/foundation/inspect_diagnostics_report.md",
        "docs/reports/foundation/inspect_stability_report.md",
        "docs/reports/foundation/inspect_diagnostics_dashboard.md",
        "docs/reports/foundation/artifact_lineage_visualization_report.md",
        "docs/reports/foundation/app_inspect_explain_latency_baseline.md",
        "docs/adr/20260308-inspect-guarantees.md",
        "configs/suites/inspect_verification.json",
        "configs/schema/operator/run_inspect.schema.json",
        "configs/schema/operator/artifact_inspect.schema.json",
        "configs/schema/operator/run_timeline.schema.json",
        "crates/bijux-dag-app/tests/plan_explain_inspect_output_contract.rs",
        "crates/bijux-dag-app/tests/operator_ux_contract.rs",
        "crates/bijux-dag-app/tests/artifact_inspect_storage_contracts.rs",
        "crates/bijux-dag-app/tests/operator_input_no_panic_contracts.rs",
        "crates/bijux-dag-app/tests/route_entrypoint_no_panic_contract.rs",
        "crates/bijux-dag-app/tests/app_smoke_routed_workflows_contract.rs",
        "crates/bijux-dev-dag/tests/artifact_lineage_completion_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
