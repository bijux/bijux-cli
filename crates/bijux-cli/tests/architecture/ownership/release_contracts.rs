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
fn workspace_manifest_declares_shared_project_links() {
    let manifest = read_repo_file("Cargo.toml");
    for required in [
        "homepage = ",
        "documentation = ",
        "authors = [\"Bijan Mousavi <mousavi.bijan@gmail.com>\"]",
    ] {
        assert!(
            manifest.contains(required),
            "workspace manifest is missing shared project metadata: {required}"
        );
    }
}

#[test]
fn crate_manifests_declare_clear_publish_metadata() {
    for (path, required) in [
        (
            "crates/bijux-cli/Cargo.toml",
            vec![
                "description = ",
                "homepage",
                "documentation = ",
                "readme = ",
                "keywords = [",
                "categories = [",
                "automation",
                "plugins",
            ],
        ),
        (
            "crates/bijux-cli-python/Cargo.toml",
            vec![
                "description = ",
                "homepage",
                "documentation = ",
                "readme = ",
                "keywords = [",
                "categories = [",
                "installing and launching the Bijux command runtime",
            ],
        ),
        (
            "crates/bijux-dev-cli/Cargo.toml",
            vec![
                "description = ",
                "homepage",
                "documentation = ",
                "readme = ",
                "keywords = [",
                "categories = [",
                "ownership contracts",
            ],
        ),
    ] {
        let manifest = read_repo_file(path);
        for field in required {
            assert!(manifest.contains(field), "{path} is missing crate metadata field: {field}");
        }
    }
}

#[test]
fn crate_documentation_links_match_current_public_docs() {
    for (path, expected) in [
        (
            "crates/bijux-cli/Cargo.toml",
            "https://bijux.github.io/bijux-cli/04-architecture/runtime-and-distribution/",
        ),
        (
            "crates/bijux-cli-python/Cargo.toml",
            "https://bijux.github.io/bijux-cli/06-reference/integrations-and-routed-runtimes/",
        ),
        (
            "crates/bijux-dev-cli/Cargo.toml",
            "https://bijux.github.io/bijux-cli/04-architecture/maintainer-control-plane/",
        ),
    ] {
        let manifest = read_repo_file(path);
        assert!(
            manifest.contains(expected),
            "{path} should point to the current public documentation surface"
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
