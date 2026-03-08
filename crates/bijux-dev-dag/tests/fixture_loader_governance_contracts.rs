use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use bijux_dag_testkit as _;
use clap as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;

use std::fs;
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_rs_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("entry").path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|v| v.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}

#[test]
fn duplicate_fixture_loader_report_is_present_and_zero_duplicate() {
    let report = root().join("docs/reports/foundation/duplicate_fixture_loader_helpers_report.md");
    assert!(report.exists(), "missing duplicate fixture loader report");
    let body = fs::read_to_string(report).expect("read report");
    assert!(
        body.contains("Duplicate helper names: 0"),
        "fixture loader report shows duplicate helper names"
    );
}

#[test]
fn fixture_loader_helpers_stay_centralized_in_testkit_for_app_artifacts_and_dev_dag() {
    let mut files = Vec::new();
    for rel in [
        "crates/bijux-dag-app",
        "crates/bijux-dag-artifacts",
        "crates/bijux-dev-dag",
    ] {
        collect_rs_files(&root().join(rel), &mut files);
    }

    let repo = root();
    for file in files {
        let rel = file
            .strip_prefix(&repo)
            .expect("relative file")
            .to_string_lossy()
            .to_string();

        if rel.ends_with("fixture_loader_governance_contracts.rs") {
            continue;
        }

        let content = fs::read_to_string(&file).expect("read source");
        for line in content.lines() {
            let trimmed = line.trim_start();
            assert!(
                !(trimmed.starts_with("fn load_") && trimmed.contains("fixture")),
                "forbidden local fixture loader declaration in {rel}: {trimmed}"
            );
            assert!(
                !trimmed.starts_with("fn fixture_path"),
                "forbidden local fixture_path helper in {rel}: {trimmed}"
            );
            assert!(
                !trimmed.starts_with("fn fixture_dir"),
                "forbidden local fixture_dir helper in {rel}: {trimmed}"
            );
            assert!(
                !trimmed.starts_with("fn fixtures_root"),
                "forbidden local fixtures_root helper in {rel}: {trimmed}"
            );
        }
    }
}
