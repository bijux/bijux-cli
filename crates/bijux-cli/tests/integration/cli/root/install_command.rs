#![forbid(unsafe_code)]
//! Root install command contracts.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str], envs: &[(&str, &Path)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root = env::temp_dir().join(format!("bijux-install-command-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[cfg(unix)]
fn write_stub_cargo(bin_dir: &Path, log_path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/bin/sh\nprintf '%s\\n' \"$@\" > \"{}\"\nprintf 'stub cargo stderr\\n' >&2\n",
        log_path.display()
    );
    let path = bin_dir.join("cargo");
    fs::write(&path, script).expect("write cargo stub");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod");
}

#[cfg(windows)]
fn write_stub_cargo(bin_dir: &Path, log_path: &Path) {
    let script = format!(
        "@echo off\r\n(echo %*) > \"{}\"\r\necho stub cargo stderr 1>&2\r\nexit /b 0\r\n",
        log_path.display()
    );
    fs::write(bin_dir.join("cargo.bat"), script).expect("write cargo stub");
}

#[test]
fn dry_run_uses_short_aliases_without_exposing_package_names_as_commands() {
    let out = run(&["install", "dev-cli", "--dry-run", "--format", "text"], &[]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert_eq!(stdout.trim(), "cargo install --locked bijux-dev-cli");
}

#[test]
fn json_dry_run_reports_registry_backed_product_aliases() {
    let out = run(&["install", "dev-atlas", "--dry-run", "--format", "json", "--no-pretty"], &[]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["target"], "dev-atlas");
    assert_eq!(payload["package"], "bijux-dev-atlas");
    assert_eq!(payload["executable"], "bijux-dev-atlas");
    assert_eq!(payload["dry_run"], true);
}

#[test]
fn dag_dry_run_uses_the_published_cargo_package_name() {
    let out = run(&["install", "dag", "--dry-run", "--format", "json", "--no-pretty"], &[]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["target"], "dag");
    assert_eq!(payload["package"], "bijux-dag-cli");
    assert_eq!(payload["executable"], "bijux-dag");
    assert_eq!(
        payload["command"],
        serde_json::json!(["cargo", "install", "--locked", "bijux-dag-cli"])
    );
}

#[test]
fn install_executes_cargo_for_runtime_aliases() {
    let root = temp_dir("stub-cargo");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    let log_path = root.join("cargo-args.txt");
    write_stub_cargo(&bin_dir, &log_path);

    let out = run(&["install", "dna", "--format", "text"], &[("PATH", &bin_dir)]);
    assert_eq!(out.status.code(), Some(0));
    let stderr = String::from_utf8(out.stderr).expect("utf-8");
    assert!(stderr.contains("stub cargo stderr"));

    let logged = fs::read_to_string(&log_path).expect("read cargo log");
    assert!(logged.contains("install"));
    assert!(logged.contains("--locked"));
    assert!(logged.contains("bijux-dna"));
}

#[test]
fn install_rejects_unknown_targets_with_structured_error() {
    let out = run(&["install", "unknown-tool", "--format", "json", "--no-pretty"], &[]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let payload: Value = serde_json::from_slice(&out.stderr).expect("json error");
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["requested_target"], "unknown-tool");
    assert!(payload["supported_targets"]
        .as_array()
        .is_some_and(|targets| targets.iter().any(|target| target == "dev-cli")));
    assert!(payload["supported_targets"].as_array().is_some_and(|targets| targets
        .iter()
        .all(|target| { !target.as_str().unwrap_or_default().starts_with("bijux-") })));
}
