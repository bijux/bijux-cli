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
fn engine_split_support_modules_and_scheduler_reports_exist() {
    for rel in [
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine_dispatch.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine_observe.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine_finalize.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution/engine_record.rs",
        "docs/reports/foundation/scheduler_profile_report.json",
        "docs/spec/SCHEDULER_FAIRNESS_DETERMINISM.md",
        "docs/spec/REFERENCE_RUNTIME.md",
        "docs/spec/CPU_MEMORY_BUDGET_MODEL.md",
        "docs/reports/foundation/scheduler_overhead_baseline.md",
        "docs/reports/foundation/runtime_execution_conformance_suite.md",
        "docs/reports/foundation/runtime_error_classification_report.md",
    ] {
        assert!(repo_root().join(rel).exists(), "missing {rel}");
    }
}

#[test]
fn runtime_execution_tests_are_module_based_not_include_based() {
    let test_root = repo_root().join("crates/bijux-dag-runtime/tests");
    let mut include_hits = Vec::new();
    for entry in std::fs::read_dir(&test_root).expect("read tests dir") {
        let entry = entry.expect("entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let raw = std::fs::read_to_string(&path).expect("read test file");
        if raw.contains("include!(") {
            include_hits.push(path);
        }
    }
    assert!(
        include_hits.is_empty(),
        "runtime tests should avoid include!-based aggregation: {include_hits:?}"
    );
}
