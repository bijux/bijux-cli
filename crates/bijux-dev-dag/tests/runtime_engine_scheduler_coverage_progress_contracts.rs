use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn runtime_engine_scheduler_coverage_report_and_evidence_are_present() {
    let root = repo_root();

    let required_paths = [
        "docs/reports/foundation/runtime_engine_scheduler_coverage_completion_report.md",
        "docs/reports/foundation/runtime_engine_scheduler_hotpath_benchmark.md",
        "docs/reports/foundation/runtime_scheduler_contract_drift_report.md",
        "docs/reports/foundation/scheduler_profile_report.json",
        "configs/suites/runtime_engine_scheduler_fast.json",
        "crates/bijux-dev-dag/tests/runtime_engine_scheduler_fast_suite_contracts.rs",
    ];

    for rel in required_paths {
        assert!(root.join(rel).exists(), "missing runtime coverage artifact {rel}");
    }

    let report = fs::read_to_string(
        root.join("docs/reports/foundation/runtime_engine_scheduler_coverage_completion_report.md"),
    )
    .expect("read runtime coverage completion report");

    for required in [
        "(301-320)",
        "engine_dispatch.rs",
        "scheduler_workload.rs",
        "runtime_scheduler_state_machine_invariants_contracts.rs",
        "runtime_engine_scheduler_hotpath_benchmark.md",
        "runtime_scheduler_contract_drift_report.md",
        "runtime_engine_scheduler_fast.json",
    ] {
        assert!(
            report.contains(required),
            "runtime coverage completion report missing {required}"
        );
    }
}
