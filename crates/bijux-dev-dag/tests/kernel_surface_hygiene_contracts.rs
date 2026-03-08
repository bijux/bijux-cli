use bijux_dag_testkit as _;
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
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("workspace crates parent")
        .parent()
        .expect("workspace root")
        .to_path_buf()
}

#[test]
fn runtime_core_does_not_reference_control_plane_or_ai_assist_surfaces() {
    let root = repo_root();
    let runtime_core = root.join("crates/bijux-dag-runtime/src/runtime_core");
    let forbidden = [
        "control_plane",
        "control-plane",
        "evidence/reports",
        "ai_operator_assist",
        "workflow_product",
        "reporting",
    ];
    let mut offenders = Vec::new();
    let mut stack = vec![runtime_core];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if path.extension().and_then(|v| v.to_str()) != Some("rs") {
                continue;
            }
            let text = fs::read_to_string(&path).expect("read source");
            for token in forbidden {
                if text.contains(token) {
                    let rel = path
                        .strip_prefix(&root)
                        .expect("strip prefix")
                        .to_string_lossy()
                        .replace('\\', "/");
                    offenders.push(format!("{rel} -> {token}"));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "runtime_core kernel surface contains non-kernel tokens: {}",
        offenders.join(" | ")
    );
}

#[test]
fn kernel_reports_exist_for_ci_consumers() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/kernel_api_surface_report.md",
        "docs/reports/foundation/kernel_public_api_audit.md",
        "docs/reports/foundation/public_api_shrink_report.md",
    ] {
        assert!(root.join(rel).exists(), "missing kernel CI report: {rel}");
    }
}
