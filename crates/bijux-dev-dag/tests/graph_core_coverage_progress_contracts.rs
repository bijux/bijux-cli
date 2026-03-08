use bijux_dag_testkit as _;
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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn graph_core_reports_exist_and_track_scope_completion() {
    let root = repo_root();
    let low_coverage = root.join("docs/reports/foundation/graph_core_low_coverage_report.md");
    let inventory = root.join("docs/reports/foundation/graph_core_fixture_inventory_report.md");
    let completion =
        root.join("docs/reports/foundation/graph_core_direct_coverage_completion_report.md");

    assert!(
        low_coverage.exists(),
        "missing graph core low coverage report"
    );
    assert!(
        inventory.exists(),
        "missing graph core fixture inventory report"
    );
    assert!(completion.exists(), "missing graph core completion report");

    let completion_body = fs::read_to_string(completion).expect("read completion report");
    for required in [
        "461-480",
        "graph_pipeline_planner_expansion_contracts.rs",
        "graph_core_canonical_topology_validate_fast.json",
        "graph_core_zero_coverage_gate_contracts.rs",
    ] {
        assert!(
            completion_body.contains(required),
            "graph core completion report missing {required}"
        );
    }
}
