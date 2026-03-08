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
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn hotspot_and_repo_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/file_size_hotspot_report.md",
        "docs/reports/foundation/long_function_hotspot_report.md",
        "docs/reports/foundation/public_api_hotspot_report.md",
        "docs/reports/foundation/dependency_cycle_report.md",
        "docs/reports/foundation/doc_drift_report.md",
        "docs/architecture/module_ownership_map.md",
        "docs/TEST_TAXONOMY.md",
        "docs/REPO_CONSTITUTION.md",
        "docs/tracking/STRUCTURAL_DEBT_REGISTER.md",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing repo-health surface: {rel}"
        );
    }
}

#[test]
fn app_and_dev_crates_do_not_depend_on_runtime_core_internals() {
    let root = repo_root();
    for rel in ["crates/bijux-dag-app/src", "crates/bijux-dev-dag/src"] {
        let base = root.join(rel);
        let mut stack = vec![base];
        while let Some(path) = stack.pop() {
            for entry in fs::read_dir(&path).expect("read dir") {
                let entry = entry.expect("entry");
                let p = entry.path();
                if p.is_dir() {
                    stack.push(p);
                    continue;
                }
                if p.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }
                let body = fs::read_to_string(&p).expect("read file");
                assert!(
                    !body.contains("runtime_core::")
                        && !body.contains("bijux_dag_runtime::runtime_core"),
                    "app/dev crate must not import runtime_core internals: {}",
                    p.display()
                );
            }
        }
    }
}

#[test]
fn root_scripts_directory_is_absent() {
    let root = repo_root();
    assert!(
        !root.join("scripts").exists(),
        "root scripts directory should not exist; use bijux-dev-dag repo commands instead"
    );
}
