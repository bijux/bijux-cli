#![forbid(unsafe_code)]
//! Binary-level config parity integration tests.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use libc as _;
use serde_json::Value;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-cli-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

#[test]
fn config_get_uses_env_variable_override_for_config_path() {
    let temp = make_temp_dir("env-path");
    let config_path = temp.join("custom.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(["cli", "config", "get", "alpha"])
        .env("BIJUXCLI_CONFIG", config_path.display().to_string())
        .output()
        .expect("binary should execute");

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let payload: Value = serde_json::from_str(&stdout).expect("json payload");
    assert_eq!(payload["value"], "1");
}

#[test]
fn config_get_prefers_runtime_env_key_over_file_value() {
    let temp = make_temp_dir("env-key");
    let config_path = temp.join("custom.env");
    fs::write(&config_path, "BIJUXCLI_SAMPLE=file\n").expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(["cli", "config", "get", "sample"])
        .env("BIJUXCLI_CONFIG", config_path.display().to_string())
        .env("BIJUXCLI_SAMPLE", "env")
        .output()
        .expect("binary should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let payload: Value = serde_json::from_str(&stdout).expect("json payload");
    assert_eq!(payload["value"], "env");
}

#[test]
fn config_flag_override_takes_precedence_over_env_path() {
    let temp = make_temp_dir("flag-override");
    let env_path = temp.join("env.env");
    let flag_path = temp.join("flag.env");
    fs::write(&env_path, "BIJUXCLI_ALPHA=env\n").expect("write env path");
    fs::write(&flag_path, "BIJUXCLI_ALPHA=flag\n").expect("write flag path");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args([
            "cli",
            "config",
            "get",
            "alpha",
            "--config-path",
            flag_path.to_str().expect("utf-8 path"),
        ])
        .env("BIJUXCLI_CONFIG", env_path.display().to_string())
        .output()
        .expect("binary should execute");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("stdout utf-8");
    let payload: Value = serde_json::from_str(&stdout).expect("json payload");
    assert_eq!(payload["value"], "flag");
}

#[test]
fn config_failure_routes_machine_error_to_stderr() {
    let output = run(&["cli", "config", "get", "missing_key"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(!output.stderr.is_empty());

    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    let payload: Value = serde_json::from_str(&stderr).expect("json payload");
    assert_eq!(payload["code"], 2);
}

#[test]
fn config_success_keeps_stderr_empty() {
    let temp = make_temp_dir("stderr-empty");
    let config_path = temp.join("custom.env");

    let output = run(&[
        "cli",
        "config",
        "set",
        "alpha=1",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_path.to_str().expect("utf-8 path"),
    ]);

    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    assert!(!output.stdout.is_empty());
}
