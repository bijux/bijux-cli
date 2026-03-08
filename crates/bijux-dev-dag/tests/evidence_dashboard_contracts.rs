use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn release_and_advisory_dashboards_exist_in_human_and_machine_forms() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/release_evidence_dashboard.md",
        "docs/reports/foundation/advisory_evidence_dashboard.md",
        "docs/reports/foundation/release_evidence_dashboard.json",
        "docs/reports/foundation/advisory_evidence_dashboard.json",
    ] {
        assert!(root.join(rel).exists(), "missing dashboard artifact: {rel}");
    }
}

#[test]
fn release_dashboard_is_blocking_and_advisory_dashboard_is_non_blocking() {
    let root = repo_root();
    let release: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("docs/reports/foundation/release_evidence_dashboard.json"))
            .expect("read release dashboard"),
    )
    .expect("parse release dashboard");
    let advisory: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("docs/reports/foundation/advisory_evidence_dashboard.json"))
            .expect("read advisory dashboard"),
    )
    .expect("parse advisory dashboard");

    assert_eq!(release["blocking"], true);
    assert_eq!(advisory["blocking"], false);
    assert!(release["families"]
        .as_array()
        .expect("release families")
        .iter()
        .any(|entry| entry == "battle"));
    assert_eq!(
        advisory["families"]
            .as_array()
            .expect("advisory families")
            .as_slice(),
        [serde_json::json!("compare")]
    );
}
