#![forbid(unsafe_code)]
//! Release-tree stamping guardrails.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tempfile::{tempdir, TempDir};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn script_path() -> PathBuf {
    repo_root().join(".github/scripts/prepare_release_tree.py")
}

fn run_prepare_release_tree(
    script: &Path,
    workspace_root: &Path,
    output_dir: &Path,
    version: &str,
) -> Output {
    Command::new("python3")
        .args([
            script.to_str().expect("script path utf-8"),
            "--workspace-root",
            workspace_root.to_str().expect("workspace path utf-8"),
            "--output-dir",
            output_dir.to_str().expect("output path utf-8"),
            "--version",
            version,
        ])
        .output()
        .expect("prepare_release_tree.py should execute")
}

fn write_file(path: &Path, content: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dirs");
    }
    fs::write(path, content).expect("write file");
}

fn run_status(repo_root: &Path, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .current_dir(repo_root)
        .args(args)
        .status()
        .unwrap_or_else(|err| panic!("run {program} failed: {err}"));
    assert!(status.success(), "{program} {:?} failed", args);
}

fn init_release_tree_fixture() -> TempDir {
    let fixture = tempdir().expect("fixture tempdir");
    let root = fixture.path();

    write_file(
        &root.join("Cargo.toml"),
        r#"[workspace]
members = ["crates/bijux-demo"]
resolver = "2"

[workspace.package]
version = "0.2.0"
"#,
    );
    write_file(
        &root.join("crates/bijux-demo/Cargo.toml"),
        r#"[package]
name = "bijux-demo"
version.workspace = true
edition = "2021"

[lib]
path = "src/lib.rs"
"#,
    );
    write_file(
        &root.join("crates/bijux-demo/src/lib.rs"),
        "pub fn exported_value() -> &'static str { \"committed-head\" }\n",
    );
    write_file(
        &root.join("crates/bijux-demo/src/build/builder.rs"),
        "pub const BUILDER_SURFACE: &str = \"preserved\";\n",
    );
    write_file(
        &root.join("crates/bijux-demo/src/artifacts/mod.rs"),
        "pub const ARTIFACTS_SURFACE: &str = \"preserved\";\n",
    );
    write_file(
        &root.join("templates/plugins-py/cookiecutter.json"),
        "{\n  \"cli_min\": \"0.2.0\",\n  \"cli_max\": \"0.3.0\"\n}\n",
    );
    write_file(
        &root.join("templates/plugins-rs/cookiecutter.json"),
        "{\n  \"cli_min\": \"0.2.0\",\n  \"cli_max\": \"0.3.0\"\n}\n",
    );

    run_status(root, "cargo", &["generate-lockfile"]);
    run_status(root, "git", &["init"]);
    run_status(root, "git", &["config", "user.name", "Bijux Tests"]);
    run_status(root, "git", &["config", "user.email", "tests@bijux.local"]);
    run_status(root, "git", &["add", "."]);
    run_status(root, "git", &["commit", "-m", "seed release tree fixture"]);

    fixture
}

#[test]
fn prepare_release_tree_exports_committed_head_not_dirty_worktree() {
    let fixture = init_release_tree_fixture();
    let root = fixture.path();
    let output_root = tempdir().expect("temp output root");
    let output_dir = output_root.path().join("release-tree");
    fs::create_dir(&output_dir).expect("create output dir");

    write_file(
        &root.join("crates/bijux-demo/src/lib.rs"),
        "pub fn exported_value() -> &'static str { \"dirty-worktree\" }\n",
    );
    write_file(&root.join("untracked-note.txt"), "this file must not leak into the release tree\n");

    let out = run_prepare_release_tree(&script_path(), root, &output_dir, "0.3.0");
    assert!(
        out.status.success(),
        "prepare_release_tree.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let released_source = fs::read_to_string(output_dir.join("crates/bijux-demo/src/lib.rs"))
        .expect("release source");
    assert!(released_source.contains("committed-head"));
    assert!(!released_source.contains("dirty-worktree"));
    assert!(
        !output_dir.join("untracked-note.txt").exists(),
        "untracked worktree files must not leak into the release tree"
    );
}

#[test]
fn prepare_release_tree_preserves_nested_source_directories() {
    let fixture = init_release_tree_fixture();
    let root = fixture.path();
    let output_root = tempdir().expect("temp output root");
    let output_dir = output_root.path().join("release-tree");
    fs::create_dir(&output_dir).expect("create output dir");

    let out = run_prepare_release_tree(&script_path(), root, &output_dir, "0.3.0");
    assert!(
        out.status.success(),
        "prepare_release_tree.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    assert!(output_dir.join("crates/bijux-demo/src/build/builder.rs").is_file());
    assert!(output_dir.join("crates/bijux-demo/src/artifacts/mod.rs").is_file());
}

#[test]
fn prepare_release_tree_stamps_template_compatibility_defaults() {
    let workspace_root = repo_root();
    let output_root = tempdir().expect("temp output root");
    let output_dir = output_root.path().join("release-tree");
    fs::create_dir(&output_dir).expect("create output dir");

    let out = run_prepare_release_tree(&script_path(), &workspace_root, &output_dir, "0.3.0");

    assert!(
        out.status.success(),
        "prepare_release_tree.py failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let py_cookiecutter =
        fs::read_to_string(output_dir.join("templates/plugins-py/cookiecutter.json"))
            .expect("python template defaults");
    let rs_cookiecutter =
        fs::read_to_string(output_dir.join("templates/plugins-rs/cookiecutter.json"))
            .expect("rust template defaults");
    let workspace_manifest =
        fs::read_to_string(output_dir.join("Cargo.toml")).expect("workspace manifest");

    for rendered in [&py_cookiecutter, &rs_cookiecutter] {
        assert!(rendered.contains(r#""cli_min": "0.3.0""#));
        assert!(rendered.contains(r#""cli_max": "0.4.0""#));
    }
    assert!(workspace_manifest.contains(r#"version = "0.3.0""#));
    assert!(workspace_manifest
        .contains(r#"bijux-cli = { version = "0.3.0", path = "crates/bijux-cli" }"#));
}
