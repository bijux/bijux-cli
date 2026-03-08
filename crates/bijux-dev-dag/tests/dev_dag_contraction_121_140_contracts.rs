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
fn dev_dag_121_140_status_report_exists_and_covers_required_sections() {
    let report =
        root().join("docs/reports/foundation/dev_dag_contraction_121_140_status_report.md");
    assert!(report.exists(), "missing report: {}", report.display());
    let raw = fs::read_to_string(report).expect("read report");
    for token in [
        "121 decomposition of command surfaces",
        "122-133 direct test anchors for commands/repo/report/tooling",
        "134 dev-dag 0%-coverage report grouped by command family",
        "135 dev-dag command-size report grouped by command family",
        "137-138 release and advisory signal dashboards",
        "139 fast helper suite for repo/tooling/report modules",
        "140 end-state ADR for command decomposition",
    ] {
        assert!(raw.contains(token), "missing report token: {token}");
    }
}

#[test]
fn dev_dag_121_140_governance_artifacts_exist() {
    for rel in [
        "crates/bijux-dev-dag/src/commands/perf_evidence.rs",
        "crates/bijux-dev-dag/src/commands/suite_catalog.rs",
        "crates/bijux-dev-dag/src/commands/evidence_registry.rs",
        "crates/bijux-dev-dag/src/commands/reporting.rs",
        "crates/bijux-dev-dag/src/commands/command_runtime.rs",
        "crates/bijux-dev-dag/src/commands/shared_io.rs",
        "crates/bijux-dev-dag/src/repo/layout.rs",
        "crates/bijux-dev-dag/src/repo/root.rs",
        "crates/bijux-dev-dag/src/report/write.rs",
        "crates/bijux-dev-dag/src/tooling/cargo.rs",
        "crates/bijux-dev-dag/src/tooling/git.rs",
        "docs/reports/foundation/dev_dag_zero_coverage_report_by_command_family.md",
        "docs/reports/foundation/dev_dag_command_size_report_by_family.md",
        "docs/reports/foundation/evidence_dashboard.md",
        "docs/reports/foundation/advisory_evidence_dashboard.md",
        "configs/suites/dev_dag_helpers_fast.json",
        "crates/bijux-dev-dag/tests/dev_dag_helpers_fast_suite_contracts.rs",
        "docs/adr/20260308-dev-dag-command-decomposition-shape.md",
    ] {
        assert!(
            root().join(rel).exists(),
            "missing required artifact: {rel}"
        );
    }
}
