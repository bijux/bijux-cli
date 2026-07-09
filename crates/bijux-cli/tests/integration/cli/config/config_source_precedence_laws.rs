#![forbid(unsafe_code)]
//! Config precedence and source reporting laws.
//! test_type: config-source-precedence

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!(
            "bijux-config-source-precedence-laws-{name}-{}-{counter}",
            std::process::id(),
        ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[test]
fn cli_flags_override_env_backed_values_and_config_path() {
    let root = temp_dir("config-source-precedence-laws");
    let env_path = root.join("env.env");
    let arg_path = root.join("arg.env");
    fs::write(&env_path, "BIJUXCLI_ALPHA=from-env-path\n").expect("env cfg");
    fs::write(&arg_path, "BIJUXCLI_ALPHA=from-arg-path\n").expect("arg cfg");

    let out = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            arg_path.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_ALPHA", "from-env-value"),
            ("BIJUXCLI_CONFIG", env_path.to_str().expect("utf-8")),
        ],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["value"], "from-env-value");
    assert_eq!(payload["source"], "env");
    assert_eq!(payload["source_env"], "BIJUXCLI_ALPHA");
    assert!(payload["source_path"].is_null());
}

#[test]
fn env_overrides_file_and_file_overrides_default_with_missing_fallback() {
    let root = temp_dir("config-source-precedence-laws");
    let file = root.join("config.env");
    let missing = root.join("missing.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("file cfg");

    let env_over_file = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            file.to_str().expect("utf-8"),
        ],
        &[("BIJUXCLI_ALPHA", "from-env")],
    );
    assert_eq!(env_over_file.status.code(), Some(0));
    let env_payload: Value = serde_json::from_slice(&env_over_file.stdout).expect("json");
    assert_eq!(env_payload["value"], "from-env");
    assert_eq!(env_payload["source"], "env");
    assert!(env_payload["source_path"].is_null());

    let file_over_default = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        file.to_str().expect("utf-8"),
    ]);
    assert_eq!(file_over_default.status.code(), Some(0));
    let file_payload: Value = serde_json::from_slice(&file_over_default.stdout).expect("json");
    assert_eq!(file_payload["value"], "from-file");
    assert_eq!(file_payload["source"], "file");

    let missing_fallback = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        missing.to_str().expect("utf-8"),
    ]);
    assert_eq!(missing_fallback.status.code(), Some(0));
    let missing_payload: Value = serde_json::from_slice(&missing_fallback.stdout).expect("json");
    assert_eq!(missing_payload, serde_json::json!({}));
}

#[test]
fn malformed_and_duplicate_config_source_behavior_is_stable() {
    let root = temp_dir("config-source-precedence-laws");
    let malformed = root.join("malformed.env");
    let duplicate = root.join("duplicate.env");
    fs::write(&malformed, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("malformed");
    fs::write(&duplicate, "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=2\n").expect("duplicate");

    let malformed_one = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        malformed.to_str().expect("utf-8"),
    ]);
    let malformed_two = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        malformed.to_str().expect("utf-8"),
    ]);
    assert_eq!(malformed_one.status.code(), Some(1));
    assert_eq!(malformed_two.status.code(), Some(1));
    assert_eq!(malformed_one.stderr, malformed_two.stderr);

    let duplicate_get = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        duplicate.to_str().expect("utf-8"),
    ]);
    assert_eq!(duplicate_get.status.code(), Some(1));
    assert!(duplicate_get.stdout.is_empty());
    assert!(String::from_utf8_lossy(&duplicate_get.stderr).contains("Duplicate key"));
}
