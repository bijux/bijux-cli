use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
struct ReleaseValidationSuite {
    format: String,
    local_entrypoint: String,
    ci_entrypoint: String,
    release_tree: ReleaseTreeContract,
    public_dag_crates: Vec<String>,
    commands: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct ReleaseTreeContract {
    script: String,
    candidate_ref: String,
    version_source: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("repo root")
        .to_path_buf()
}

fn read_suite() -> ReleaseValidationSuite {
    let path = repo_root().join("configs/dag/release/release_validation_suite.json");
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| panic!("read suite failed: {err}"));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse suite failed: {err}"))
}

#[test]
fn release_validation_suite_is_current() {
    let suite = read_suite();
    assert_eq!(suite.format, "release-validation-suite/v1");
    assert_eq!(suite.local_entrypoint, "make release-validate-rs");
    assert_eq!(suite.ci_entrypoint, "make gh-release-validate");
    assert_eq!(suite.release_tree.script, ".github/scripts/prepare_release_tree.py");
    assert_eq!(suite.release_tree.candidate_ref, "HEAD");
    assert_eq!(suite.release_tree.version_source, "workspace.package.version");
    assert_eq!(
        suite.public_dag_crates,
        vec![
            "bijux-dag-core",
            "bijux-dag-artifacts",
            "bijux-dag-runtime",
            "bijux-dag-app",
            "bijux-dag-cli",
        ]
    );
}

#[test]
fn release_validation_suite_commands_cover_required_release_checks() {
    let suite = read_suite();
    let commands = suite.commands;
    assert_eq!(commands.len(), 15, "expected exact release validation command count");

    let required = [
        "cargo fmt --all -- --check",
        "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings",
        "cargo test --workspace --all-targets --all-features --locked",
        "cargo doc --workspace --all-features --no-deps",
        "cargo package -p bijux-dag-core --list",
        "cargo package -p bijux-dag-artifacts --list",
        "cargo package -p bijux-dag-runtime --list",
        "cargo package -p bijux-dag-app --list",
        "cargo package -p bijux-dag-cli --list",
        "cargo publish -p bijux-dag-core --dry-run --locked",
        "cargo publish -p bijux-dag-artifacts --dry-run --locked",
        "cargo publish -p bijux-dag-runtime --dry-run --locked",
        "cargo publish -p bijux-dag-app --dry-run --locked",
        "cargo publish -p bijux-dag-cli --dry-run --locked",
        "cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture",
    ];

    for command in required {
        assert!(
            commands.iter().any(|entry| entry == command),
            "missing release validation command: {command}"
        );
    }

    let fmt_index = commands
        .iter()
        .position(|entry| entry == "cargo fmt --all -- --check")
        .expect("fmt command");
    let clippy_index = commands
        .iter()
        .position(|entry| entry == "cargo clippy --workspace --all-targets --all-features --locked -- -D warnings")
        .expect("clippy command");
    let test_index = commands
        .iter()
        .position(|entry| entry == "cargo test --workspace --all-targets --all-features --locked")
        .expect("test command");
    let doc_index = commands
        .iter()
        .position(|entry| entry == "cargo doc --workspace --all-features --no-deps")
        .expect("doc command");
    let smoke_index = commands
        .iter()
        .position(|entry| entry == "cargo test -p bijux-dag-cli --test smoke_pipeline --locked -- --nocapture")
        .expect("smoke command");

    assert!(fmt_index < clippy_index);
    assert!(clippy_index < test_index);
    assert!(test_index < doc_index);
    assert!(doc_index < smoke_index);
}
