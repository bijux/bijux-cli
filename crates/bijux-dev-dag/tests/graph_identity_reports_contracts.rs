use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::Path;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn graph_identity_docs_and_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/GRAPH_IDENTITY_FIELD_IMPACT.md",
        "docs/spec/NODE_FINGERPRINT_FIELD_IMPACT.md",
        "docs/reports/foundation/graph_identity_decomposition_report.json",
        "docs/reports/foundation/graph_identity_field_impact_report.json",
        "docs/reports/foundation/canonical_diff_fixture_inventory_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing graph identity artifact: {rel}");
    }
}

#[test]
fn graph_identity_reports_are_generated_from_dev_dag_bin() {
    let root = repo_root();
    let generator = fs::read_to_string(
        root.join("crates/bijux-dev-dag/src/bin/generate_graph_identity_reports.rs"),
    )
    .expect("read generator source");
    assert!(
        generator.contains("graph_identity_field_impact_report.json"),
        "generator must emit graph identity field impact report"
    );
    assert!(
        generator.contains("canonical_diff_fixture_inventory_report.md"),
        "generator must emit canonical diff fixture inventory report"
    );
}
