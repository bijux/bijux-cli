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
fn run_identity_docs_schemas_and_reports_exist() {
    let root = repo_root();
    for rel in [
        "docs/spec/RUN_IDENTITY_CONTRACT.md",
        "docs/spec/RUN_HISTORY_CONTRACT.md",
        "docs/spec/RUN_SUMMARY_SCHEMA_v0.1.md",
        "docs/spec/RUN_MANIFEST_EVOLUTION_MATRIX.md",
        "docs/reports/foundation/run_identity_fields_report.md",
        "configs/schema/operator/run_history.schema.json",
        "configs/schema/operator/run_id_explain.schema.json",
    ] {
        assert!(
            root.join(rel).exists(),
            "missing run identity surface: {rel}"
        );
    }
}

#[test]
fn app_cli_model_exposes_history_and_id_explain_subcommands() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .expect("read app command model");
    for token in ["History {", "IdExplain {"] {
        assert!(
            source.contains(token),
            "run identity command not wired in CLI model: {token}"
        );
    }
}

#[test]
fn run_inspection_commands_require_explicit_root_argument() {
    let root = repo_root();
    let source = fs::read_to_string(root.join("crates/bijux-dag-app/src/commands/mod.rs"))
        .expect("read app command model");
    let runs_enum_start = source
        .find("pub(crate) enum RunsCommands")
        .expect("runs enum start");
    let runs_source = &source[runs_enum_start..];
    for block in [
        "History {",
        "IdExplain {",
        "Show {",
        "Inspect {",
        "Timeline {",
    ] {
        let pos = runs_source
            .find(block)
            .unwrap_or_else(|| panic!("missing runs command block: {block}"));
        let tail = &runs_source[pos..runs_source.len().min(pos + 220)];
        assert!(
            tail.contains("root: PathBuf"),
            "runs command must require explicit --root to avoid ambient state: {block}"
        );
    }
}
