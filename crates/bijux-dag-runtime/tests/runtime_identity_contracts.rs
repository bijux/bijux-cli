use bijux_dag_artifacts as _;
use bijux_dag_core as _;
use bijux_dag_runtime as _;
use ctrlc as _;
use hex as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use tempfile as _;
use thiserror as _;

#[path = "../build_support.rs"]
mod build_support;

use bijux_dag_core::parse_graph_strict;
use bijux_dag_runtime::{Runtime, RuntimeConfig};
use serde_json::Value;
use std::fs;

fn simple_const_graph() -> String {
    r#"{
      "spec": "bijux-dag/v0.1",
      "nodes": [
        {
          "id": "const1",
          "kind": "const",
          "inputs": [],
          "outputs": [
            {
              "name": "value",
              "path": "value.txt"
            }
          ],
          "params": {
            "value": "hello"
          }
        }
      ],
      "edges": []
    }"#
    .to_string()
}

#[test]
fn build_stamp_normalization_accepts_trimmed_hex_only() {
    assert_eq!(build_support::BUILD_GIT_SHA_ENV, "BIJUX_DAG_BUILD_GIT_SHA");
    assert_eq!(build_support::normalize_git_sha("  AbC1234  ").as_deref(), Some("abc1234"));
    assert!(build_support::normalize_git_sha("abc123").is_none());
    assert!(build_support::normalize_git_sha("not-a-sha").is_none());
}

#[test]
fn build_stamp_supports_gitfile_worktrees() {
    let temp = tempfile::tempdir().expect("temp dir");
    let workspace_root = temp.path().join("workspace");
    let linked_git_dir = temp.path().join("git-dir");
    fs::create_dir_all(&workspace_root).expect("workspace root");
    fs::create_dir_all(&linked_git_dir).expect("linked git dir");
    fs::write(workspace_root.join(".git"), format!("gitdir: {}\n", linked_git_dir.display()))
        .expect("write git file");

    let resolved = build_support::git_dir_from_workspace_root(&workspace_root)
        .expect("resolve linked git dir");
    assert_eq!(resolved, linked_git_dir);
}

#[test]
fn build_stamp_workspace_root_and_rerun_paths_track_git_metadata() {
    let temp = tempfile::tempdir().expect("temp dir");
    let workspace_root = temp.path().join("workspace");
    let crate_dir = workspace_root.join("crates").join("bijux-dag-runtime");
    let git_dir = workspace_root.join(".git");
    fs::create_dir_all(&crate_dir).expect("crate dir");
    fs::create_dir_all(git_dir.join("refs").join("heads")).expect("git refs");
    fs::write(git_dir.join("HEAD"), "ref: refs/heads/main\n").expect("write head");
    fs::write(git_dir.join("refs").join("heads").join("main"), "abc1234\n").expect("write branch");

    let resolved_root = build_support::workspace_root_from_manifest_dir(&crate_dir);
    assert_eq!(resolved_root, workspace_root);

    let rerun_paths = build_support::git_rerun_paths(&git_dir);
    assert!(rerun_paths.contains(&git_dir.join("HEAD")));
    assert!(rerun_paths.contains(&git_dir.join("packed-refs")));
    assert!(rerun_paths.contains(&git_dir.join("refs").join("heads").join("main")));
}

#[test]
fn runtime_outputs_use_build_stamped_tool_version() {
    let graph = parse_graph_strict(&simple_const_graph()).expect("parse graph");
    let runtime = Runtime::new();
    let temp = tempfile::tempdir().expect("temp dir");
    let run_dir = runtime.run(&graph, temp.path(), RuntimeConfig::default()).expect("runtime run");

    let expected = match option_env!("BIJUX_DAG_BUILD_GIT_SHA") {
        Some(build_git_sha) if !build_git_sha.trim().is_empty() => {
            format!("{}+{}", env!("CARGO_PKG_VERSION"), build_git_sha)
        }
        _ => env!("CARGO_PKG_VERSION").to_string(),
    };

    let manifest: Value =
        serde_json::from_str(&fs::read_to_string(run_dir.join("manifest.json")).expect("manifest"))
            .expect("parse manifest");
    assert_eq!(manifest["tool_version"], expected);

    let provenance: Value = serde_json::from_str(
        &fs::read_to_string(run_dir.join("provenance.json")).expect("provenance"),
    )
    .expect("parse provenance");
    assert_eq!(provenance["tool_version"], expected);
}
