#![forbid(unsafe_code)]
//! Python-vs-Rust compatibility tests for config command outputs.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-compat-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run_with_env(binary: &str, args: &[&str], envs: &HashMap<String, String>) -> Output {
    let mut command = Command::new(binary);
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("process should execute")
}

fn python_cli() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().and_then(|p| p.parent()).expect("workspace root");
    root.join("bin").join("bijux").display().to_string()
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

#[test]
fn config_set_and_get_match_python_on_exit_and_core_fields() {
    let temp = make_temp_dir("set-get");
    let config_path = temp.join("config.env");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py_set = run_with_env(
        &python_cli(),
        &["config", "set", "alpha=1", "--format", "json", "--no-pretty"],
        &envs,
    );
    let rs_set = run_with_env(
        env!("CARGO_BIN_EXE_bijux-rs"),
        &[
            "cli",
            "config",
            "set",
            "alpha=1",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8 path"),
        ],
        &envs,
    );

    assert_eq!(py_set.status.code(), rs_set.status.code());
    assert!(py_set.stderr.is_empty());
    assert!(rs_set.stderr.is_empty());

    let py_set_json = parse_json(&py_set.stdout);
    let rs_set_json = parse_json(&rs_set.stdout);
    assert_eq!(py_set_json["status"], rs_set_json["status"]);
    assert_eq!(py_set_json["key"], rs_set_json["key"]);
    assert_eq!(py_set_json["value"], rs_set_json["value"]);

    let py_get = run_with_env(
        &python_cli(),
        &["config", "get", "alpha", "--format", "json", "--no-pretty"],
        &envs,
    );
    let rs_get = run_with_env(
        env!("CARGO_BIN_EXE_bijux-rs"),
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8 path"),
        ],
        &envs,
    );

    assert_eq!(py_get.status.code(), rs_get.status.code());
    assert!(py_get.stderr.is_empty());
    assert!(rs_get.stderr.is_empty());

    let py_get_json = parse_json(&py_get.stdout);
    let rs_get_json = parse_json(&rs_get.stdout);
    assert_eq!(py_get_json["value"], rs_get_json["value"]);
}

#[test]
fn config_get_missing_key_matches_python_failure_routing() {
    let temp = make_temp_dir("missing");
    let config_path = temp.join("config.env");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py = run_with_env(
        &python_cli(),
        &["config", "get", "missing", "--format", "json", "--no-pretty"],
        &envs,
    );
    let rs = run_with_env(
        env!("CARGO_BIN_EXE_bijux-rs"),
        &[
            "cli",
            "config",
            "get",
            "missing",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8 path"),
        ],
        &envs,
    );

    assert_eq!(py.status.code(), rs.status.code());
    assert!(py.stdout.is_empty());
    assert!(rs.stdout.is_empty());
    assert!(!py.stderr.is_empty());
    assert!(!rs.stderr.is_empty());
}
