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
    for required in
        ["homepage = ", "documentation = ", "authors = [\"Bijan Mousavi <bijan@bijux.io>\"]"]
    {
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
                "publish = false",
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
            "crates/bijux-dev/Cargo.toml",
            vec!["description = ", "homepage", "readme = ", "Unified maintainer control plane"],
        ),
    ] {
        let manifest = read_repo_file(path);
        for field in required {
            assert!(manifest.contains(field), "{path} is missing crate metadata field: {field}");
        }
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
        ".github/workflows/release-github.yml",
        ".github/workflows/release-ghcr.yml",
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
                || content.contains("toolchain: ${{ steps.config.outputs.rust_toolchain }}")
                || !content.contains("dtolnay/rust-toolchain@"),
            "{path} must set the pinned Rust toolchain input when using dtolnay/rust-toolchain"
        );
    }
}

#[test]
fn github_release_workflow_publishes_release_assets_from_the_stamped_release_tree() {
    let workflow = read_repo_file(".github/workflows/release-github.yml");
    let release_env = read_repo_file(".github/release.env");
    let release_files = read_repo_file(".github/release-github.files");
    let prepare_script = read_repo_file(".github/scripts/prepare_release_github.sh");
    for required in [
        "softprops/action-gh-release@",
        "source \".github/release.env\"",
        "eval \"${{ steps.config.outputs.plan_command }}\"",
        "eval \"${{ steps.config.outputs.wait_for_ci_command }}\"",
        "eval \"${{ steps.config.outputs.prepare_command }}\"",
        "gh run download",
        "release_files_manifest",
    ] {
        assert!(
            workflow.contains(required),
            "release-github.yml must keep reusable release workflow guardrails: {required}"
        );
    }
    for required in [
        "BIJUX_RELEASE_PLAN_COMMAND=make gh-release-plan-github",
        "BIJUX_RELEASE_WAIT_FOR_CI_COMMAND=make gh-release-wait-for-ci",
        "BIJUX_RELEASE_PREPARE_COMMAND=.github/scripts/prepare_release_github.sh",
        "BIJUX_RELEASE_SETUP_PYTHON=true",
        "BIJUX_RELEASE_SETUP_RUST=true",
        "BIJUX_GHCR_RELEASE_ENABLED=true",
        "BIJUX_GHCR_RELEASE_ALLOWED_PACKAGES=bijux-cli",
        "BIJUX_CRATES_RELEASE_ALLOWED_PACKAGES=bijux-cli",
    ] {
        assert!(
            release_env.contains(required),
            ".github/release.env must keep core release guardrails: {required}"
        );
    }
    for required in [
        "artifacts/github-release/*.whl",
        "artifacts/github-release/*.tar.gz",
        "artifacts/github-release/sha256sums.txt",
    ] {
        assert!(
            release_files.contains(required),
            ".github/release-github.files must keep required release uploads: {required}"
        );
    }
    for required in [
        "python3 .github/scripts/prepare_release_tree.py",
        "release_tree=\"${GITHUB_WORKSPACE}/artifacts/release-tree\"",
        "maturin build",
        "--compatibility pypi",
        "oras push",
        "sha256sums.txt",
        "Repository releases mirror the stamped tag artifacts for this version.",
    ] {
        assert!(
            prepare_script.contains(required),
            ".github/scripts/prepare_release_github.sh must keep GitHub Release asset guardrails: {required}"
        );
    }
}

#[test]
fn pypi_release_workflow_builds_pypi_compatible_distributions() {
    let workflow = read_repo_file(".github/workflows/release-pypi.yml");
    for required in [
        "PyO3/maturin-action@",
        "maturin-version: ${{ needs.resolve.outputs.maturin_version }}",
        "manylinux: \"2014\"",
        "--compatibility pypi",
        "release_tree=\"${GITHUB_WORKSPACE}/artifacts/release-tree\"",
        "make publish-py PUBLISH_BUILD=0",
    ] {
        assert!(
            workflow.contains(required),
            "release-pypi.yml must keep PyPI-safe build and upload guardrails: {required}"
        );
    }
}

#[test]
fn crates_release_automation_only_targets_public_rust_runtime_crate() {
    let workflow_support = read_repo_file("makes/gh.mk");
    let publish_support = read_repo_file("makes/rust.mk");

    assert!(
        workflow_support.contains("GH_CRATES_RELEASE_PACKAGES ?= bijux-cli"),
        "release planning should only consider the public Rust runtime crate for crates.io publication"
    );
    assert!(
        workflow_support.contains("gh-release-plan-github"),
        "release planning support should include a dedicated GitHub Release lane"
    );
    assert!(
        publish_support.contains("RUST_PUBLISH_PACKAGES ?= bijux-cli"),
        "cargo publish automation should only target the public Rust runtime crate by default"
    );
    assert!(
        !workflow_support.contains("GH_CRATES_RELEASE_PACKAGES ?= bijux-cli bijux-cli-python"),
        "release planning must not treat the Python bridge crate as a crates.io package"
    );
    assert!(
        publish_support.contains("RUST_PUBLISH_SKIP_EXISTING ?= 1"),
        "cargo publish automation should skip already-published crate versions by default"
    );
    assert!(
        publish_support.contains("already present on crates.io"),
        "cargo publish automation should emit an explicit skip path when a crate version already exists"
    );
    assert!(
        publish_support.contains("registry already has this release"),
        "cargo publish automation should recover cleanly when cargo publish reports a duplicate release"
    );
}

#[test]
fn generated_ported_snapshots_are_not_checked_in() {
    let ported_dir = repo_root().join("crates/bijux-cli/tests/data/golden/ported");
    if !ported_dir.exists() {
        return;
    }

    let entries = fs::read_dir(&ported_dir).expect("ported snapshot dir");
    let checked_in = entries
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect::<Vec<_>>();
    assert!(
        checked_in.is_empty(),
        "generated ported snapshots must stay out of the repo: {checked_in:?}"
    );
}

#[test]
fn vendored_runtime_registry_matches_root_contract() {
    let root_contract = read_repo_file("contracts/official_product_namespace_registry.json");
    let vendored_contract =
        read_repo_file("crates/bijux-cli/contracts/official_product_namespace_registry.json");
    assert_eq!(
        vendored_contract, root_contract,
        "published runtime crate must vendor the same official product registry contract as the workspace root"
    );
}
