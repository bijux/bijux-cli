#![forbid(unsafe_code)]
//! Release-tree stamping guardrails.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::tempdir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
fn prepare_release_tree_stamps_template_compatibility_defaults() {
    let workspace_root = repo_root();
    let output_root = tempdir().expect("temp output root");
    let output_dir = output_root.path().join("release-tree");
    fs::create_dir(&output_dir).expect("create output dir");

    let script = workspace_root.join(".github/scripts/prepare_release_tree.py");
    let out = Command::new("python3")
        .args([
            script.to_str().expect("script path utf-8"),
            "--workspace-root",
            workspace_root.to_str().expect("workspace path utf-8"),
            "--output-dir",
            output_dir.to_str().expect("output path utf-8"),
            "--version",
            "0.2.0",
        ])
        .output()
        .expect("prepare_release_tree.py should execute");

    assert!(
        out.status.success(),
        "prepare_release_tree.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let py_cookiecutter = fs::read_to_string(output_dir.join("templates/plugins-py/cookiecutter.json"))
        .expect("python template defaults");
    let rs_cookiecutter = fs::read_to_string(output_dir.join("templates/plugins-rs/cookiecutter.json"))
        .expect("rust template defaults");

    for rendered in [&py_cookiecutter, &rs_cookiecutter] {
        assert!(rendered.contains(r#""cli_min": "0.2.0""#));
        assert!(rendered.contains(r#""cli_max": "0.3.0""#));
    }
}
