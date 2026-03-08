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
fn dev_dag_contraction_artifacts_cover_361_380() {
    let root = repo_root();

    let required = [
        "docs/reports/foundation/dev_dag_contraction_completion_report.md",
        "docs/reports/foundation/dev_dag_hot_files_report.md",
        "docs/reports/foundation/dev_dag_low_coverage_files_report.md",
        "docs/reports/foundation/release_critical_evidence_commands_only_report.md",
        "docs/reports/foundation/advisory_only_evidence_commands_report.md",
        "crates/bijux-dev-dag/tests/file_size_guardrails.rs",
        "crates/bijux-dev-dag/tests/evidence_lane_classification_contracts.rs",
        "docs/adr/20260308-dev-dag-cleanup-end-state.md",
    ];

    for rel in required {
        assert!(
            root.join(rel).exists(),
            "missing dev-dag contraction artifact {rel}"
        );
    }

    let completion = fs::read_to_string(
        root.join("docs/reports/foundation/dev_dag_contraction_completion_report.md"),
    )
    .expect("read dev-dag contraction completion report");

    for required in [
        "(361-380)",
        "commands/perf_evidence.rs",
        "file_size_guardrails.rs",
        "dev_dag_hot_files_report.md",
        "release_critical_evidence_commands_only_report.md",
        "evidence_lane_classification_contracts.rs",
        "20260308-dev-dag-cleanup-end-state.md",
    ] {
        assert!(
            completion.contains(required),
            "dev-dag contraction completion report missing {required}"
        );
    }
}
