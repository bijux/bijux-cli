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
fn runtime_helper_81_100_status_report_exists_and_covers_required_items() {
    let report = root()
        .join("docs/reports/foundation/runtime_helper_direct_coverage_81_100_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "engine_dispatch.rs",
        "engine_observe.rs",
        "engine_finalize.rs",
        "engine_record.rs",
        "engine_metrics.rs",
        "scheduler_workload.rs",
        "equal-priority deterministic ordering",
        "partial-rerun closure semantics",
        "runtime_helper_low_coverage_report.md",
        "runtime_helper_invariants_fast.json",
        "runtime_architecture_health_dashboard.md",
    ] {
        assert!(
            raw.contains(token),
            "missing token in status report: {token}"
        );
    }
}

#[test]
fn runtime_helper_81_100_governance_artifacts_exist() {
    for rel in [
        "configs/suites/runtime_helper_invariants_fast.json",
        "docs/reports/foundation/runtime_helper_low_coverage_report.md",
        "docs/reports/foundation/runtime_engine_scheduler_hotpath_benchmark.md",
        "docs/reports/foundation/runtime_architecture_health_dashboard.md",
        "docs/reports/foundation/runtime_engine_scheduler_coverage_completion_report.md",
        "crates/bijux-dag-runtime/tests/runtime_execution_helper_expansion_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_determinism_contracts.rs",
        "crates/bijux-dag-runtime/tests/runtime_scheduler_state_machine_invariants_contracts.rs",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
