#![forbid(unsafe_code)]
//! Build script that resolves runtime version metadata from git tags.

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

use semver::Version;

const VERSION_SOURCE_OVERRIDE: &str = "override";
const VERSION_SOURCE_GIT_TAG: &str = "git-tag";
const VERSION_SOURCE_GIT_TAG_DERIVED: &str = "git-tag-derived";
const VERSION_SOURCE_PACKAGE_FALLBACK: &str = "package-fallback";

#[derive(Debug, Clone, PartialEq, Eq)]
struct RuntimeBuildVersion {
    semver_version: String,
    display_version: String,
    source: &'static str,
    git_commit: Option<String>,
    git_dirty: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct GitDerivedVersion {
    base_semver: String,
    commits_since_tag: u64,
    commit_abbrev: String,
    dirty: bool,
}

fn main() {
    println!("cargo:rerun-if-env-changed=BIJUX_VERSION_OVERRIDE");

    let workspace_root = workspace_root();
    emit_git_rerun_hints(&workspace_root);

    let package_version = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".to_string());
    let metadata = resolve_runtime_versions(&workspace_root, &package_version);

    println!("cargo:rustc-env=BIJUX_BUILD_SEMVER_VERSION={}", metadata.semver_version);
    println!("cargo:rustc-env=BIJUX_BUILD_DISPLAY_VERSION={}", metadata.display_version);
    println!("cargo:rustc-env=BIJUX_BUILD_VERSION_SOURCE={}", metadata.source);
    emit_optional_env("BIJUX_BUILD_GIT_COMMIT", metadata.git_commit.as_deref());
    emit_optional_env(
        "BIJUX_BUILD_GIT_DIRTY",
        metadata.git_dirty.map(|dirty| if dirty { "1" } else { "0" }),
    );
}

fn workspace_root() -> PathBuf {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf)
}

fn resolve_runtime_versions(workspace_root: &Path, package_version: &str) -> RuntimeBuildVersion {
    let git_commit = git_commit_abbrev(workspace_root);
    let git_dirty = git_dirty_state(workspace_root);

    if let Some(override_version) =
        env::var("BIJUX_VERSION_OVERRIDE").ok().and_then(|value| normalize_version_string(&value))
    {
        return RuntimeBuildVersion {
            semver_version: override_version.clone(),
            display_version: override_version,
            source: VERSION_SOURCE_OVERRIDE,
            git_commit,
            git_dirty,
        };
    }

    if git_dirty != Some(true) {
        if let Some(version) = describe_exact_tag_version(workspace_root) {
            return RuntimeBuildVersion {
                semver_version: version.clone(),
                display_version: tagged_display_version(&version),
                source: VERSION_SOURCE_GIT_TAG,
                git_commit,
                git_dirty,
            };
        }
    }

    if let Some(derived) =
        describe_git_version(workspace_root).or_else(|| latest_tag_baseline_version(workspace_root))
    {
        return RuntimeBuildVersion {
            semver_version: derived_semver_version(
                &derived.base_semver,
                derived.commits_since_tag,
                &derived.commit_abbrev,
                derived.dirty,
            ),
            display_version: derived_display_version(
                &derived.base_semver,
                derived.commits_since_tag,
                &derived.commit_abbrev,
                derived.dirty,
            ),
            source: VERSION_SOURCE_GIT_TAG_DERIVED,
            git_commit: Some(derived.commit_abbrev),
            git_dirty: Some(derived.dirty),
        };
    }

    let fallback = fallback_package_version(package_version);
    RuntimeBuildVersion {
        semver_version: fallback.clone(),
        display_version: tagged_display_version(&fallback),
        source: VERSION_SOURCE_PACKAGE_FALLBACK,
        git_commit,
        git_dirty,
    }
}

fn describe_exact_tag_version(workspace_root: &Path) -> Option<String> {
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

fn describe_git_version(workspace_root: &Path) -> Option<GitDerivedVersion> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["describe", "--tags", "--match", "v[0-9]*", "--long", "--dirty", "--abbrev=12"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let describe = String::from_utf8_lossy(&output.stdout);
    parse_git_describe(describe.trim())
}

fn parse_git_describe(raw: &str) -> Option<GitDerivedVersion> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let (without_dirty, dirty) = match trimmed.strip_suffix("-dirty") {
        Some(value) => (value, true),
        None => (trimmed, false),
    };

    let (tag_and_count, commit_abbrev) = without_dirty.rsplit_once("-g")?;
    if commit_abbrev.trim().is_empty() {
        return None;
    }

    let (tag, count_raw) = tag_and_count.rsplit_once('-')?;
    let commits_since_tag = count_raw.parse::<u64>().ok()?;
    let base_semver = normalize_version_string(tag)?;
    Some(GitDerivedVersion {
        base_semver,
        commits_since_tag,
        commit_abbrev: commit_abbrev.trim().to_string(),
        dirty,
    })
}

fn latest_tag_baseline_version(workspace_root: &Path) -> Option<GitDerivedVersion> {
    let tag = latest_version_tag(workspace_root)?;
    let base_semver = normalize_version_string(&tag)?;
    let commit_abbrev = git_commit_abbrev(workspace_root)?;
    let dirty = git_dirty_state(workspace_root)?;
    let commits_since_tag = commits_since_tag(workspace_root, &tag)?;
    Some(GitDerivedVersion { base_semver, commits_since_tag, commit_abbrev, dirty })
}

fn latest_version_tag(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["tag", "--list", "v[0-9]*", "--sort=-version:refname"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToOwned::to_owned)
}

fn commits_since_tag(workspace_root: &Path, tag: &str) -> Option<u64> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-list", "--count", &format!("{tag}..HEAD")])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8_lossy(&output.stdout).trim().parse::<u64>().ok()
}

fn tagged_display_version(version: &str) -> String {
    format!("v{version}")
}

fn derived_display_version(
    base_semver: &str,
    commits_since_tag: u64,
    commit_abbrev: &str,
    dirty: bool,
) -> String {
    let mut version = format!(
        "{}+dev.{}.g{}",
        tagged_display_version(base_semver),
        commits_since_tag,
        commit_abbrev
    );
    if dirty {
        version.push_str(".dirty");
    }
    version
}

fn derived_semver_version(
    base_semver: &str,
    commits_since_tag: u64,
    commit_abbrev: &str,
    dirty: bool,
) -> String {
    let mut next = Version::parse(base_semver).expect("base git tag semver should be valid");
    next.patch += 1;

    let mut version = format!("{next}-dev.{commits_since_tag}.g{commit_abbrev}");
    if dirty {
        version.push_str(".dirty");
    }
    version
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

fn git_commit_abbrev(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--short=12", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if commit.is_empty() {
        return None;
    }
    Some(commit)
}

fn git_dirty_state(workspace_root: &Path) -> Option<bool> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["status", "--porcelain"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(!String::from_utf8_lossy(&output.stdout).trim().is_empty())
}

fn emit_optional_env(key: &str, value: Option<&str>) {
    if let Some(value) = value {
        println!("cargo:rustc-env={key}={value}");
    }
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
    let resolved =
        if git_dir_path.is_absolute() { git_dir_path } else { workspace_root.join(git_dir_path) };

    for relative in ["HEAD", "packed-refs", "refs/tags", "refs/heads"] {
        let candidate = resolved.join(relative);
        println!("cargo:rerun-if-changed={}", candidate.display());
    }
}
