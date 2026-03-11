#![forbid(unsafe_code)]
//! Plugin scaffold minimal layout and lifecycle checks.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str], plugins_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env("BIJUXCLI_PLUGINS_DIR", plugins_dir)
        .output()
        .expect("binary should execute")
}

fn run_ok_json(args: &[&str], plugins_dir: &Path) -> Value {
    let out = run(args, plugins_dir);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn tmp_dir(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "bijux-plugin-minimal-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir temp");
    base
}

fn manifest_file(scaffold_dir: &Path) -> PathBuf {
    scaffold_dir.join("plugin.manifest.json")
}

fn file_set(base: &Path) -> BTreeSet<String> {
    let mut files = BTreeSet::new();
    let mut stack = vec![base.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let rel = path
                    .strip_prefix(base)
                    .expect("relative path")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(rel);
            }
        }
    }
    files
}

fn expected_snapshot(path: &str) -> BTreeSet<String> {
    let content = match path {
        "python" => include_str!(
            "../../../data/golden/cli_surface/plugin_scaffold_python_minimal_files.txt"
        ),
        "rust" => {
            include_str!("../../../data/golden/cli_surface/plugin_scaffold_rust_minimal_files.txt")
        }
        _ => panic!("unknown snapshot"),
    };
    content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

#[test]
fn scaffold_minimal_layout_is_stable_and_runnable_for_python_and_rust() {
    let root = tmp_dir("scaffold");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    for (kind, namespace) in [("python", "minpy"), ("rust", "minrust")] {
        let scaffold_dir = root.join(format!("{}_plugin", kind));
        run_ok_json(
            &[
                "cli",
                "plugins",
                "scaffold",
                kind,
                namespace,
                "--path",
                scaffold_dir.to_str().expect("utf-8"),
            ],
            &plugins_dir,
        );

        let produced = file_set(&scaffold_dir);
        let expected = expected_snapshot(kind);
        assert_eq!(produced, expected, "{kind} scaffold file set drifted");

        for forbidden in ["README.md", "pyproject.toml", "Cargo.toml", ".gitignore"] {
            assert!(
                !scaffold_dir.join(forbidden).exists(),
                "{kind} scaffold should not include decorative file {forbidden}"
            );
        }

        run_ok_json(
            &[
                "cli",
                "plugins",
                "install",
                manifest_file(&scaffold_dir).to_str().expect("utf-8"),
            ],
            &plugins_dir,
        );

        let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
        assert!(listed["plugins"]
            .as_array()
            .expect("plugins array")
            .iter()
            .any(|item| item["manifest"]["namespace"] == namespace));

        let check = run_ok_json(&["cli", "plugins", "check", namespace], &plugins_dir);
        assert_eq!(check["status"], "healthy");

        let uninstall = run_ok_json(&["cli", "plugins", "uninstall", namespace], &plugins_dir);
        assert_eq!(uninstall["status"], "uninstalled");
    }
}
