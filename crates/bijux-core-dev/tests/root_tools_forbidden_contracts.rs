use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use std::path::{Path, PathBuf};
use tempfile as _;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn root_tools_directory_is_not_present() {
    assert!(
        !root().join("tools").exists(),
        "root tools directory is forbidden; use bijux-dev-dag binaries"
    );
}

#[test]
fn governance_report_generators_live_in_bijux_dev_dag_bin_only() {
    let bin_dir = root().join("crates/bijux-core-dev/src/bin");
    for required in [
        "generate_duplicate_fixture_loader_report.rs",
        "generate_fixture_governance_reports.rs",
        "generate_human_output_governance_reports.rs",
        "generate_json_output_governance_reports.rs",
    ] {
        assert!(
            bin_dir.join(required).exists(),
            "missing bijux-dev-dag report generator: {required}"
        );
    }
}
