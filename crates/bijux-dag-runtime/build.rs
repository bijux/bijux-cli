#![forbid(unsafe_code)]

mod build_support;

use std::env;
use std::process::Command;

use build_support::{
    git_dir_from_workspace_root, git_rerun_paths, normalize_git_sha,
    workspace_root_from_manifest_dir, BUILD_GIT_SHA_ENV,
};

fn main() {
    let workspace_root = workspace_root();
    emit_git_rerun_hints(&workspace_root);
    println!("cargo:rerun-if-env-changed={BUILD_GIT_SHA_ENV}");

    if let Some(git_sha) = resolved_build_git_sha(&workspace_root) {
        println!("cargo:rustc-env=BIJUX_DAG_BUILD_GIT_SHA={git_sha}");
    }
}

fn workspace_root() -> std::path::PathBuf {
    let manifest_dir = std::path::PathBuf::from(
        env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );
    workspace_root_from_manifest_dir(&manifest_dir)
}

fn emit_git_rerun_hints(workspace_root: &std::path::Path) {
    let Some(git_dir) = git_dir_from_workspace_root(workspace_root) else {
        return;
    };
    for rerun_path in git_rerun_paths(&git_dir) {
        println!("cargo:rerun-if-changed={}", rerun_path.display());
    }
}

fn git_commit_abbrev(workspace_root: &std::path::Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    normalize_git_sha(&String::from_utf8_lossy(&output.stdout))
}

fn resolved_build_git_sha(workspace_root: &std::path::Path) -> Option<String> {
    if let Ok(explicit_sha) = env::var(BUILD_GIT_SHA_ENV) {
        return Some(normalize_git_sha(&explicit_sha).unwrap_or_else(|| {
            panic!(
                "{BUILD_GIT_SHA_ENV} must be a 7-40 character hexadecimal Git revision, got `{}`",
                explicit_sha.trim()
            )
        }));
    }
    git_commit_abbrev(workspace_root)
}
