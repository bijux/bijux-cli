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
fn kernel_contract_docs_and_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/KERNEL_BOUNDARY_CONTRACT.md",
        "docs/spec/KERNEL_DEPENDENCY_POLICY.md",
        "docs/reports/foundation/kernel_api_surface_report.md",
        "docs/reports/foundation/public_api_shrink_report.md",
        "docs/architecture/SACRED_EXECUTION_FLOW_KERNEL_NOTE.md",
        "configs/policy/kernel_dependency_policy.json",
        "configs/suites/kernel_smoke.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing kernel boundary surface: {rel}"
        );
    }
}

#[test]
fn kernel_crates_forbid_cli_and_dev_governance_dependencies() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/kernel_dependency_policy.json"))
            .expect("read kernel dependency policy"),
    )
    .expect("parse kernel dependency policy");

    let crates = policy["kernel_crates"]
        .as_array()
        .expect("kernel_crates array");
    let forbidden = policy["forbidden_dependencies"]
        .as_array()
        .expect("forbidden_dependencies array");

    for crate_path in crates {
        let rel = crate_path.as_str().expect("kernel crate path");
        let content = fs::read_to_string(root.join(rel)).expect("read kernel Cargo.toml");
        for dependency in forbidden {
            let dependency = dependency.as_str().expect("forbidden dependency");
            assert!(
                !content.contains(dependency),
                "kernel crate dependency policy violation: {rel} contains `{dependency}`"
            );
        }
    }
}

#[test]
fn kernel_sources_do_not_reference_evidence_report_surfaces() {
    let root = repo_root();
    let policy: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(root.join("configs/policy/kernel_dependency_policy.json"))
            .expect("read kernel dependency policy"),
    )
    .expect("parse kernel dependency policy");
    let forbidden_tokens = policy["forbidden_source_tokens"]
        .as_array()
        .expect("forbidden_source_tokens");

    let mut offenders = Vec::new();
    for src_root in ["crates/bijux-dag-core/src", "crates/bijux-dag-runtime/src"] {
        let mut stack = vec![root.join(src_root)];
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
                let content = fs::read_to_string(&path).expect("read source");
                for token in forbidden_tokens {
                    let token = token.as_str().expect("forbidden token");
                    if content.contains(token) {
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
    }

    assert!(
        offenders.is_empty(),
        "kernel source references evidence/report surfaces: {}",
        offenders.join(" | ")
    );
}
