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

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

#[test]
fn command_modules_keep_uniform_error_surface_signature() {
    let root = repo_root();
    let mut offenders = Vec::new();
    for rel in [
        "crates/bijux-dev-dag/src/commands/perf_evidence.rs",
        "crates/bijux-dev-dag/src/commands/suite_catalog.rs",
        "crates/bijux-dev-dag/src/commands/command_runtime.rs",
    ] {
        let src = fs::read_to_string(root.join(rel)).expect("read command module");
        if src.contains("Result<(), anyhow::Error>") || src.contains("Result<(), Box<dyn") {
            offenders.push(rel.to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "dev-dag command modules must use Result<_, String> style error surface: {offenders:?}"
    );
}

#[test]
fn commands_do_not_write_runtime_source_tree() {
    let root = repo_root();
    let src = fs::read_to_string(root.join("crates/bijux-dev-dag/src/commands/mod.rs"))
        .expect("read commands mod");
    for line in src.lines() {
        let references_truth_tree = line.contains("crates/bijux-dag-runtime/src")
            || line.contains("crates/bijux-dag-core/src")
            || line.contains("crates/bijux-dag-artifacts/src");
        let looks_like_mutation = line.contains("fs::write")
            || line.contains("fs::remove")
            || line.contains("create_dir_all")
            || line.contains("std::fs::write")
            || line.contains("std::fs::remove");
        assert!(
            !(references_truth_tree && looks_like_mutation),
            "dev-dag command orchestration must not mutate runtime truth source trees: {line}"
        );
    }
}

#[test]
fn governance_runtime_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/reports/foundation/governance_tool_hotspots.md",
        "docs/reports/foundation/governance_command_runtime.md",
        "docs/reports/foundation/dev_dag_command_module_boundaries.md",
    ] {
        assert!(root.join(rel).exists(), "missing governance report: {rel}");
    }
}
