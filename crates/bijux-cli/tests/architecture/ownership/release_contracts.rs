#![forbid(unsafe_code)]
//! Publishing metadata and automation contract guardrails.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_repo_file(path: &str) -> String {
    fs::read_to_string(repo_root().join(path)).expect("repo file should exist")
}

fn quoted_value_after(text: &str, prefix: &str) -> Option<String> {
    text.lines().find_map(|line| {
        let trimmed = line.trim();
        if !trimmed.starts_with(prefix) {
            return None;
        }
        let value = trimmed.strip_prefix(prefix)?.trim();
        Some(value.trim_matches('"').to_string())
    })
}

fn is_hex_sha(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[test]
fn runtime_crate_manifest_declares_publish_metadata() {
    let manifest = read_repo_file("crates/bijux-cli/Cargo.toml");
    for required in
        ["homepage = ", "documentation = ", "readme = ", "keywords = [", "categories = ["]
    {
        assert!(
            manifest.contains(required),
            "runtime crate manifest is missing publish metadata: {required}"
        );
    }
}

#[test]
fn pinned_rust_toolchain_matches_workspace_rust_version() {
    let workspace_manifest = read_repo_file("Cargo.toml");
    let rust_toolchain = read_repo_file("rust-toolchain.toml");
    let workspace_rust_version =
        quoted_value_after(&workspace_manifest, "rust-version = ").expect("workspace rust-version");
    let channel = quoted_value_after(&rust_toolchain, "channel = ").expect("toolchain channel");
    assert_eq!(
        channel,
        format!("{workspace_rust_version}.0"),
        "rust-toolchain.toml must pin the exact patch release derived from workspace rust-version"
    );
}

#[test]
fn github_workflows_pin_external_actions_to_commits() {
    for path in [
        ".github/workflows/ci.yml",
        ".github/workflows/deploy-docs.yml",
        ".github/workflows/release-crates.yml",
        ".github/workflows/release-pypi.yml",
    ] {
        let content = read_repo_file(path);
        for line in content.lines() {
            let trimmed = line.trim();
            let Some(spec) = trimmed.strip_prefix("uses: ") else {
                continue;
            };
            if spec.starts_with("./") {
                continue;
            }
            let (_, revision) = spec.split_once('@').expect("action spec must contain @");
            let revision = revision.split_whitespace().next().expect("action revision");
            assert!(
                is_hex_sha(revision),
                "{path} must pin actions to a full commit SHA, found: {spec}"
            );
        }
        assert!(
            content.contains("toolchain: ${{ env.RUST_TOOLCHAIN_VERSION }}")
                || !content.contains("dtolnay/rust-toolchain@"),
            "{path} must set the pinned Rust toolchain input when using dtolnay/rust-toolchain"
        );
    }
}
