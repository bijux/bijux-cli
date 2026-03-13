#![forbid(unsafe_code)]
//! Build script that resolves runtime version metadata from git tags.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;

fn main() {
    println!("cargo:rerun-if-env-changed=BIJUX_VERSION_OVERRIDE");

    let workspace_root = workspace_root();
    emit_git_rerun_hints(&workspace_root);

    let package_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let (semver_version, display_version) =
        resolve_runtime_versions(&workspace_root, &package_version);

    println!("cargo:rustc-env=BIJUX_BUILD_SEMVER_VERSION={semver_version}");
    println!("cargo:rustc-env=BIJUX_BUILD_DISPLAY_VERSION={display_version}");
}

fn workspace_root() -> PathBuf {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf)
}

fn resolve_runtime_versions(workspace_root: &Path, package_version: &str) -> (String, String) {
    if let Some(override_version) = env::var("BIJUX_VERSION_OVERRIDE")
        .ok()
        .and_then(|value| normalize_version_string(&value))
    {
        return (override_version.clone(), override_version);
    }

    if let Some(version) = describe_tag_version(workspace_root) {
        return (version.clone(), version);
    }

    let fallback = fallback_package_version(package_version);
    (fallback.clone(), fallback)
}

fn describe_tag_version(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["describe", "--tags", "--match", "v[0-9]*", "--exact-match"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let tag = String::from_utf8_lossy(&output.stdout);
    normalize_version_string(tag.trim())
}

fn fallback_package_version(package_version: &str) -> String {
    normalize_version_string(package_version).unwrap_or_else(|| package_version.to_string())
}

fn normalize_version_string(raw: &str) -> Option<String> {
    let trimmed = raw.trim();
    let without_prefix = trimmed.strip_prefix('v').unwrap_or(trimmed);
    let parsed = Version::parse(without_prefix).ok()?;
    Some(parsed.to_string())
}

fn emit_git_rerun_hints(workspace_root: &Path) {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--git-dir"])
        .output();
    let Ok(output) = output else {
        return;
    };
    if !output.status.success() {
        return;
    }
    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if git_dir.is_empty() {
        return;
    }

    let git_dir_path = PathBuf::from(&git_dir);
    let resolved = if git_dir_path.is_absolute() {
        git_dir_path
    } else {
        workspace_root.join(git_dir_path)
    };

    for relative in ["HEAD", "packed-refs", "refs/tags", "refs/heads"] {
        let candidate = resolved.join(relative);
        println!("cargo:rerun-if-changed={}", candidate.display());
    }
}
