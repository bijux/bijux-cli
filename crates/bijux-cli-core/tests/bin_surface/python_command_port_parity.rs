#![forbid(unsafe_code)]
//! Parity-style coverage for commands that were previously Python-only.

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn temp_home(label: &str) -> PathBuf {
    let path = env::temp_dir().join(format!("bijux-parity-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temp home");
    path
}

fn run_with_home(args: &[&str], home: &PathBuf) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env("HOME", home)
        .output()
        .expect("binary should execute")
}

#[test]
fn plugins_root_and_info_are_routed_and_return_success() {
    let home = temp_home("plugins-root");

    let root = run_with_home(&["plugins"], &home);
    assert_eq!(root.status.code(), Some(0));
    let root_stdout = String::from_utf8(root.stdout).expect("utf8 stdout");
    let root_json: serde_json::Value = serde_json::from_str(&root_stdout).expect("json payload");
    assert!(root_json.get("plugins").is_some());

    let info = run_with_home(&["plugins", "info"], &home);
    assert_eq!(info.status.code(), Some(0));
    let info_stdout = String::from_utf8(info.stdout).expect("utf8 stdout");
    let info_json: serde_json::Value = serde_json::from_str(&info_stdout).expect("json payload");
    assert!(info_json.get("compatibility_warnings").is_some());

    let _ = fs::remove_dir_all(home);
}

#[test]
fn history_clear_has_help_and_success_exit_code() {
    let help = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(["history", "clear", "--help"])
        .output()
        .expect("binary should execute");
    assert_eq!(help.status.code(), Some(0));
    let help_stdout = String::from_utf8(help.stdout).expect("utf8 stdout");
    assert!(help_stdout.contains("history"));
    assert!(help_stdout.contains("clear"));

    let home = temp_home("history-clear");
    let output = run_with_home(&["history", "clear"], &home);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("json payload");
    assert_eq!(payload["status"], "cleared");

    let _ = fs::remove_dir_all(home);
}

#[test]
fn memory_set_get_delete_clear_have_expected_exit_and_streams() {
    let home = temp_home("memory-crud");

    let set = run_with_home(&["memory", "set", "alpha=1"], &home);
    assert_eq!(set.status.code(), Some(0));
    assert!(set.stderr.is_empty());

    let get = run_with_home(&["memory", "get", "alpha"], &home);
    assert_eq!(get.status.code(), Some(0));
    let get_stdout = String::from_utf8(get.stdout).expect("utf8 stdout");
    let get_payload: serde_json::Value = serde_json::from_str(&get_stdout).expect("json payload");
    assert_eq!(get_payload["key"], "alpha");

    let delete = run_with_home(&["memory", "delete", "alpha"], &home);
    assert_eq!(delete.status.code(), Some(0));

    let clear = run_with_home(&["memory", "clear"], &home);
    assert_eq!(clear.status.code(), Some(0));
    let clear_stdout = String::from_utf8(clear.stdout).expect("utf8 stdout");
    let clear_payload: serde_json::Value =
        serde_json::from_str(&clear_stdout).expect("json payload");
    assert_eq!(clear_payload["status"], "cleared");

    let _ = fs::remove_dir_all(home);
}

#[test]
fn config_list_alias_path_returns_object_payload() {
    let home = temp_home("config-list");
    let set = run_with_home(&["cli", "config", "set", "alpha=1"], &home);
    assert_eq!(set.status.code(), Some(0));

    let output = run_with_home(&["config", "list"], &home);
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("json payload");
    assert!(payload.is_object());

    let _ = fs::remove_dir_all(home);
}
