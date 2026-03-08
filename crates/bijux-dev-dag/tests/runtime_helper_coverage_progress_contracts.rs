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
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn runtime_helper_reports_and_benchmark_artifacts_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/runtime_helper_low_coverage_report.md",
        "docs/reports/foundation/runtime_scheduler_state_machine_trace_fixture_inventory_report.md",
        "docs/reports/foundation/runtime_scheduler_helper_boundary_benchmark.md",
        "docs/reports/foundation/runtime_helper_coverage_completion_report.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing runtime helper report: {rel}"
        );
    }

    let completion = fs::read_to_string(
        root.join("docs/reports/foundation/runtime_helper_coverage_completion_report.md"),
    )
    .expect("read completion");
    for required in [
        "501-520",
        "runtime_execution_helper_expansion_contracts.rs",
        "runtime_helper_invariants_fast.json",
        "runtime_helper_zero_coverage_gate_contracts.rs",
    ] {
        assert!(
            completion.contains(required),
            "runtime helper completion report missing {required}"
        );
    }
}
