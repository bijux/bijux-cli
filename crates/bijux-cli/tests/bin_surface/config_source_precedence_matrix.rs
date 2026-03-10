#![forbid(unsafe_code)]
//! Config precedence and source reporting coverage matrix.
//! test_type: config-source-precedence

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use bijux_cli_python as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bijux-config-source-precedence-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[test]
fn cli_flags_override_env_backed_values_and_config_path() {
    let root = temp_dir("todo-301-304");
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
    assert_eq!(payload["source_path"], arg_path.to_str().expect("utf-8"));
}

#[test]
fn env_overrides_file_and_file_overrides_default_with_missing_fallback() {
    let root = temp_dir("todo-302-303-305");
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
    let root = temp_dir("todo-306-307");
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
    assert_eq!(duplicate_get.status.code(), Some(0));
    let duplicate_payload: Value = serde_json::from_slice(&duplicate_get.stdout).expect("json");
    assert_eq!(duplicate_payload["value"], "2");
}

#[test]
fn source_metadata_and_dev_cli_env_precedence_are_reported() {
    let root = temp_dir("todo-308-310");
    let file = root.join("config.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("seed");

    let get = run(&[
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
    assert_eq!(get.status.code(), Some(0));
    let get_payload: Value = serde_json::from_slice(&get.stdout).expect("json");
    assert!(get_payload.get("source_path").is_some());

    let env = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );
    assert_eq!(env.status.code(), Some(0));
    let env_payload: Value = serde_json::from_slice(&env.stdout).expect("json");
    assert_eq!(
        env_payload["source_precedence"],
        serde_json::json!(["flags", "env", "config", "defaults"])
    );
    assert_eq!(
        env_payload["active"]["config_file"],
        file.to_str().expect("utf-8")
    );
}

#[test]
fn source_reports_json_text_are_deterministic_ignore_noise_and_env_order() {
    let root = temp_dir("todo-311-315");
    let file = root.join("config.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("seed");

    let json_one = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );
    let json_two = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );
    assert_eq!(json_one.status.code(), Some(0));
    assert_eq!(json_two.status.code(), Some(0));
    assert_eq!(json_one.stdout, json_two.stdout);

    let text = run_with_env(
        &["dev", "cli", "env", "--format", "text"],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );
    assert_eq!(text.status.code(), Some(0));
    let text_body = String::from_utf8(text.stdout).expect("utf-8");
    assert!(text_body.contains("source_precedence"));

    let with_noise = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[
            ("BIJUXCLI_CONFIG", file.to_str().expect("utf-8")),
            ("UNRELATED_NOISE_A", "x"),
            ("UNRELATED_NOISE_B", "y"),
        ],
    );
    let reversed_noise = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[
            ("UNRELATED_NOISE_B", "y"),
            ("UNRELATED_NOISE_A", "x"),
            ("BIJUXCLI_CONFIG", file.to_str().expect("utf-8")),
        ],
    );
    assert_eq!(with_noise.status.code(), Some(0));
    assert_eq!(reversed_noise.status.code(), Some(0));
    assert_eq!(with_noise.stdout, reversed_noise.stdout);
}

#[test]
fn cross_command_source_precedence_consistency() {
    let root = temp_dir("todo-316");
    let file = root.join("config.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("seed");

    let get = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
        ],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );
    let env = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_CONFIG", file.to_str().expect("utf-8"))],
    );

    assert_eq!(get.status.code(), Some(0));
    assert_eq!(env.status.code(), Some(0));

    let get_payload: Value = serde_json::from_slice(&get.stdout).expect("json");
    let env_payload: Value = serde_json::from_slice(&env.stdout).expect("json");

    assert_eq!(
        get_payload["source_path"],
        env_payload["active"]["config_file"]
    );
}
