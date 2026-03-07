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
        .join("..")
        .join("..")
        .canonicalize()
        .expect("repo root")
}

#[test]
fn bundle_format_specs_exist_and_define_v1_identifiers() {
    let root = repo_root();
    let cases = [
        ("docs/spec/GRAPH_BUNDLE_FORMAT_v1.md", "graph-bundle/v1"),
        ("docs/spec/RUN_BUNDLE_FORMAT_v1.md", "run-bundle/v1"),
        ("docs/spec/ARTIFACT_BUNDLE_FORMAT_v1.md", "artifact-bundle/v1"),
        (
            "docs/spec/BUNDLE_MANIFEST_VERSIONING_POLICY.md",
            "export-bundle/v0.1",
        ),
    ];

    for (rel, token) in cases {
        let path = root.join(rel);
        assert!(path.exists(), "missing bundle format policy doc: {rel}");
        let text = fs::read_to_string(path).expect("read bundle format doc");
        assert!(text.contains(token), "bundle spec missing required token: {token}");
    }
}

#[test]
fn import_export_contract_and_cli_contract_cover_new_bundle_flags() {
    let root = repo_root();
    let import_export =
        fs::read_to_string(root.join("docs/spec/IMPORT_EXPORT_CONTRACT.md")).expect("read import export contract");
    for token in [
        "--without-artifacts",
        "--from-run",
        "--verify-only",
        "without-artifacts",
    ] {
        assert!(
            import_export.contains(token),
            "import/export contract missing token: {token}"
        );
    }

    let cli = fs::read_to_string(root.join("docs/spec/CLI_CONTRACT.md")).expect("read cli contract");
    for token in [
        "dag export --from-run",
        "dag export --without-artifacts",
        "dag import --verify-only",
    ] {
        assert!(cli.contains(token), "cli contract missing token: {token}");
    }
}
