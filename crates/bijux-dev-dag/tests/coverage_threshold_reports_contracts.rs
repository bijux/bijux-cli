use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::path::Path;

fn workspace_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("workspace root")
}

#[test]
fn coverage_threshold_reports_and_allowlist_are_tracked() {
    let root = workspace_root();
    for rel in [
        "docs/reports/foundation/line_coverage_under_50_report.md",
        "docs/reports/foundation/line_coverage_under_25_report.md",
        "docs/reports/foundation/line_coverage_zero_direct_report.md",
        "configs/policy/protected_zero_coverage_allowlist.json",
        "crates/bijux-dev-dag/src/bin/generate_line_coverage_reports.rs",
    ] {
        assert!(root.join(rel).exists(), "missing required file: {rel}");
    }
}

#[test]
fn protected_zero_coverage_allowlist_has_entries() {
    let root = workspace_root();
    let payload: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(root.join("configs/policy/protected_zero_coverage_allowlist.json"))
            .expect("read allowlist"),
    )
    .expect("parse allowlist");

    let items = payload["protected_zero_coverage_allowlist"]
        .as_array()
        .expect("allowlist entries");
    assert!(!items.is_empty(), "allowlist must not be empty");
}
