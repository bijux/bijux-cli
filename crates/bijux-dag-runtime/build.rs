#![forbid(unsafe_code)]

use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    let workspace_root = workspace_root();
    emit_git_rerun_hints(&workspace_root);

    if let Some(git_sha) = git_commit_abbrev(&workspace_root) {
        println!("cargo:rustc-env=BIJUX_DAG_BUILD_GIT_SHA={git_sha}");
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .map_or_else(|| manifest_dir.clone(), Path::to_path_buf)
}

fn emit_git_rerun_hints(workspace_root: &Path) {
    let git_dir = workspace_root.join(".git");
    let head_path = git_dir.join("HEAD");
    println!("cargo:rerun-if-changed={}", head_path.display());

    let Ok(head_contents) = std::fs::read_to_string(&head_path) else {
        return;
    };
    let Some(reference) = head_contents.trim().strip_prefix("ref: ") else {
        return;
    };
    let ref_path = git_dir.join(reference);
    println!("cargo:rerun-if-changed={}", ref_path.display());
}

fn git_commit_abbrev(workspace_root: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", workspace_root.to_string_lossy().as_ref()])
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let commit = String::from_utf8_lossy(&output.stdout).trim().to_string();
    (!commit.is_empty()).then_some(commit)
}
