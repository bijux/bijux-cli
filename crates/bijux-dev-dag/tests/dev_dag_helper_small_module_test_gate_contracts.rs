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

#[test]
fn helper_modules_under_25_lines_must_have_direct_tests() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let helper_files = [
        "crates/bijux-dev-dag/src/repo/layout.rs",
        "crates/bijux-dev-dag/src/repo/root.rs",
        "crates/bijux-dev-dag/src/report/write.rs",
        "crates/bijux-dev-dag/src/tooling/cargo.rs",
        "crates/bijux-dev-dag/src/tooling/git.rs",
        "crates/bijux-dev-dag/src/tooling/mod.rs",
    ];

    for rel in helper_files {
        let src = fs::read_to_string(root.join(rel)).expect("read helper source");
        let lines = src.lines().count();
        if lines <= 25 {
            assert!(
                src.contains("#[cfg(test)]") && src.contains("#[test]"),
                "small helper module must include direct tests: {rel}"
            );
        }
    }
}

#[test]
fn helper_untested_report_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let report = root.join("docs/reports/foundation/dev_dag_helpers_still_untested_report.md");
    assert!(report.exists(), "missing helper untested report");
    let body = fs::read_to_string(report).expect("read helper report");
    assert!(body.contains("No helper module in this scope is missing direct in-file tests."));
}
