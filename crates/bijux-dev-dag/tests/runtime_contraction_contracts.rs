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
use std::path::{Path, PathBuf};
use tempfile as _;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("read dir") {
        let path = entry.expect("dir entry").path();
        if path.is_dir() {
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|value| value.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

#[test]
fn runtime_dependency_audit_blocks_cli_and_repo_governance_crates() {
    let root = repo_root();
    let cargo_toml = fs::read_to_string(root.join("crates/bijux-dag-runtime/Cargo.toml"))
        .expect("read runtime cargo");
    for banned in ["clap", "git2", "octocrab", "reqwest", "axum", "warp"] {
        assert!(
            !cargo_toml.contains(banned),
            "runtime dependency audit violated by banned crate token `{banned}`"
        );
    }
}

#[test]
fn runtime_hot_path_has_no_docs_or_configs_literal_dependencies() {
    let root = repo_root();
    let runtime_src = root.join("crates/bijux-dag-runtime/src");
    let mut files = Vec::new();
    collect_rs_files(&runtime_src, &mut files);

    for path in files {
        let rel = path
            .strip_prefix(&root)
            .expect("strip prefix")
            .to_string_lossy()
            .replace('\\', "/");
        if rel.contains("/internal/testing/") {
            continue;
        }
        let src = fs::read_to_string(&path).expect("read runtime source");
        assert!(
            !src.contains("docs/"),
            "runtime source contains docs path dependency token: {rel}"
        );
        assert!(
            !src.contains("configs/"),
            "runtime source contains configs path dependency token: {rel}"
        );
    }
}

#[test]
fn runtime_core_duplicate_wrapper_modules_remain_removed() {
    let root = repo_root();
    for rel in [
        "crates/bijux-dag-runtime/src/runtime_core/engine.rs",
        "crates/bijux-dag-runtime/src/runtime_core/execution_context.rs",
        "crates/bijux-dag-runtime/src/runtime_core/invariants.rs",
        "crates/bijux-dag-runtime/src/runtime_core/planner_bridge.rs",
        "crates/bijux-dag-runtime/src/runtime_core/scheduler.rs",
    ] {
        assert!(
            !root.join(rel).exists(),
            "duplicate wrapper module should stay removed: {rel}"
        );
    }
}
