#![forbid(unsafe_code)]
//! Packaging and install ambiguity hardening coverage.

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn tmp_dir(name: &str) -> PathBuf {
    let dir =
        env::temp_dir().join(format!("bijux-install-ambiguity-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("run")
}

fn install_channel_path(root: &PathBuf, marker: &str) -> PathBuf {
    match marker {
        "cargo" => root.join(".cargo").join("bin"),
        "pip" => root.join("site-packages").join("bin"),
        other => panic!("unsupported marker: {other}"),
    }
}

fn executable_name() -> String {
    let extension = std::env::consts::EXE_EXTENSION;
    if extension.is_empty() {
        "bijux".to_string()
    } else {
        format!("bijux.{extension}")
    }
}

fn write_executable(path: &Path, body: &str) {
    fs::write(path, body).expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(path).expect("metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("chmod +x");
    }
}

fn write_channel_binary(dir: &PathBuf) {
    fs::create_dir_all(dir).expect("mkdir channel dir");
    write_executable(&dir.join(executable_name()), "#!/bin/sh\n");
}

fn assert_command_runs_in_channel(marker: &str, args: &[&str], envs: &[(&str, String)]) {
    let root = tmp_dir(&format!("{marker}-{:?}", args).replace([' ', '"', ',', '[', ']'], "-"));
    let dir = install_channel_path(&root, marker);
    write_channel_binary(&dir);
    let path = env::join_paths([&dir]).expect("join path").to_string_lossy().to_string();

    let mut with_path = vec![("PATH", path)];
    with_path.extend_from_slice(envs);

    let out = run_with_env(args, &with_path);
    assert_eq!(
        out.status.code(),
        Some(0),
        "command failed for {marker} {:?}: stderr={}",
        args,
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn cargo_installed_invocation_version_is_green() {
    assert_command_runs_in_channel("cargo", &["version"], &[]);
}

#[test]
fn pip_installed_invocation_version_is_green() {
    assert_command_runs_in_channel("pip", &["version"], &[]);
}

#[test]
fn cargo_installed_invocation_status_is_green() {
    assert_command_runs_in_channel("cargo", &["status", "--format", "json", "--no-pretty"], &[]);
}

#[test]
fn pip_installed_invocation_status_is_green() {
    assert_command_runs_in_channel("pip", &["status", "--format", "json", "--no-pretty"], &[]);
}

#[test]
fn cargo_installed_invocation_plugins_list_is_green() {
    assert_command_runs_in_channel(
        "cargo",
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[],
    );
}

#[test]
fn pip_installed_invocation_plugins_list_is_green() {
    assert_command_runs_in_channel(
        "pip",
        &["plugins", "list", "--format", "json", "--no-pretty"],
        &[],
    );
}

#[test]
fn cargo_installed_invocation_config_get_is_green() {
    let root = tmp_dir("cargo-config-get");
    let config_path = root.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");
    assert_command_runs_in_channel(
        "cargo",
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
    );
}

#[test]
fn pip_installed_invocation_config_get_is_green() {
    let root = tmp_dir("pip-config-get");
    let config_path = root.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");
    assert_command_runs_in_channel(
        "pip",
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[],
    );
}

#[test]
fn cli_paths_under_overridden_home_are_consistent() {
    let root = tmp_dir("paths-home-override");
    let out = run_with_env(
        &["cli", "paths", "--format", "json", "--no-pretty"],
        &[("HOME", root.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");

    let expected_prefix = root.join(".bijux");
    assert!(payload["config"]
        .as_str()
        .expect("config")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["history"]
        .as_str()
        .expect("history")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["plugins"]
        .as_str()
        .expect("plugins")
        .contains(expected_prefix.to_str().expect("utf-8")));
}

#[test]
fn cli_paths_under_xdg_style_home_root_are_consistent() {
    let root = tmp_dir("paths-xdg-style-home").join(".local").join("share");
    fs::create_dir_all(&root).expect("mkdir xdg-style home");

    let out = run_with_env(
        &["cli", "paths", "--format", "json", "--no-pretty"],
        &[("HOME", root.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");

    let expected_prefix = root.join(".bijux");
    assert!(payload["config"]
        .as_str()
        .expect("config")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["history"]
        .as_str()
        .expect("history")
        .contains(expected_prefix.to_str().expect("utf-8")));
    assert!(payload["plugins"]
        .as_str()
        .expect("plugins")
        .contains(expected_prefix.to_str().expect("utf-8")));
}
