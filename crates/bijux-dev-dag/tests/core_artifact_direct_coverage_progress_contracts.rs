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
fn core_artifact_coverage_reports_exist_and_stay_scoped() {
    let root = repo_root();
    let completion =
        root.join("docs/reports/foundation/core_artifact_direct_coverage_completion_report.md");
    let uncovered =
        root.join("docs/reports/foundation/core_artifact_still_uncovered_product_paths_report.md");
    let suite = root.join("configs/suites/core_artifact_direct_coverage_fast.json");

    assert!(completion.exists(), "missing completion report for 281-300");
    assert!(uncovered.exists(), "missing uncovered product paths report");
    assert!(
        suite.exists(),
        "missing core/artifact direct coverage fast suite"
    );

    let completion_body = fs::read_to_string(completion).expect("read completion report");
    for required in [
        "281-300",
        "graph_identity_property_contracts.rs",
        "artifact_io_expansion_contracts.rs",
        "core_artifact_direct_coverage_fast.json",
    ] {
        assert!(
            completion_body.contains(required),
            "completion report missing {required}"
        );
    }

    let uncovered_body = fs::read_to_string(uncovered).expect("read uncovered report");
    for required in [
        "Remaining Uncovered Product Paths",
        "none in this scoped set",
        "crates/bijux-dag-core/src/graph/canonical.rs",
        "crates/bijux-dag-artifacts/src/io/fs.rs",
    ] {
        assert!(
            uncovered_body.contains(required),
            "uncovered report missing {required}"
        );
    }
}
