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

#[test]
fn reporting_helpers_do_not_target_authoritative_evidence_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let reporting = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/reporting.rs"))
        .expect("read reporting source");
    let write = fs::read_to_string(root.join("crates/bijux-dev-dag/src/report/write.rs"))
        .expect("read write source");

    for forbidden in [
        "evidence/_meta/registries/evidence_registry.json",
        "evidence/ownership/evidence_ledger.json",
    ] {
        assert!(
            !reporting.contains(forbidden),
            "reporting helper must not write/read authoritative artifact path: {forbidden}"
        );
        assert!(
            !write.contains(forbidden),
            "write helper must not write/read authoritative artifact path: {forbidden}"
        );
    }
}

#[test]
fn report_determinism_dashboard_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let path = root.join("docs/reports/foundation/dev_dag_report_writing_determinism_dashboard.md");
    assert!(path.exists(), "missing determinism dashboard report");
    let body = fs::read_to_string(path).expect("read dashboard");
    assert!(body.contains("Determinism checks"));
    assert!(body.contains("Integrity checks"));
}
