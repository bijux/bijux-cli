#![forbid(unsafe_code)]
//! Config read behavior matrix coverage.
//! test_type: config-read-determinism

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
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
    let root = std::env::temp_dir()
        .join(format!("bijux-config-read-matrix-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[test]
fn root_config_list_empty_one_multiple_duplicate_comments_and_malformed_behavior() {
    let root = temp_dir("root-listing");
    let empty = root.join("empty.env");
    let one = root.join("one.env");
    let multi = root.join("multi.env");
    let duplicate = root.join("duplicate.env");
    let malformed = root.join("malformed.env");

    fs::write(&empty, "").expect("write empty");
    fs::write(&one, "BIJUXCLI_ALPHA=1\n").expect("write one");
    fs::write(&multi, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("write multi");
    fs::write(&duplicate, "BIJUXCLI_ALPHA=1\nBIJUXCLI_ALPHA=3\n# c\n\nBIJUXCLI_BETA=2\n")
        .expect("write dup");
    fs::write(&malformed, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write malformed");

    let empty_out = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        empty.to_str().expect("utf-8"),
    ]);
    assert_eq!(empty_out.status.code(), Some(0));
    let empty_json: Value = serde_json::from_slice(&empty_out.stdout).expect("json");
    assert_eq!(empty_json, serde_json::json!({}));

    let one_out = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        one.to_str().expect("utf-8"),
    ]);
    assert_eq!(one_out.status.code(), Some(0));
    let one_json: Value = serde_json::from_slice(&one_out.stdout).expect("json");
    assert_eq!(one_json["alpha"], "1");

    let multi_out = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        multi.to_str().expect("utf-8"),
    ]);
    assert_eq!(multi_out.status.code(), Some(0));
    let multi_json: Value = serde_json::from_slice(&multi_out.stdout).expect("json");
    assert_eq!(multi_json["alpha"], "1");
    assert_eq!(multi_json["beta"], "2");

    let dup_out = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        duplicate.to_str().expect("utf-8"),
    ]);
    assert_eq!(dup_out.status.code(), Some(0));
    let dup_json: Value = serde_json::from_slice(&dup_out.stdout).expect("json");
    assert_eq!(dup_json["alpha"], "3");
    assert_eq!(dup_json["beta"], "2");

    let malformed_out = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        malformed.to_str().expect("utf-8"),
    ]);
    assert_eq!(malformed_out.status.code(), Some(1));
    assert!(malformed_out.stdout.is_empty());
    assert!(!malformed_out.stderr.is_empty());
}

#[test]
fn config_get_existing_missing_invalid_with_path_and_env_override() {
    let root = temp_dir("config-get");
    let file = root.join("file.env");
    let env_file = root.join("env.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("write file");
    fs::write(&env_file, "BIJUXCLI_ALPHA=from-env\n").expect("write env file");

    let found = run(&[
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
    assert_eq!(found.status.code(), Some(0));
    let found_json: Value = serde_json::from_slice(&found.stdout).expect("json");
    assert_eq!(found_json["value"], "from-file");

    let missing = run(&[
        "cli",
        "config",
        "get",
        "missing",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        file.to_str().expect("utf-8"),
    ]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    assert!(!missing.stderr.is_empty());

    let invalid = run(&[
        "cli",
        "config",
        "get",
        "bad-key",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        file.to_str().expect("utf-8"),
    ]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(!invalid.stderr.is_empty());

    let path_override = run_with_env(
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
        &[("BIJUXCLI_CONFIG", env_file.to_str().expect("utf-8"))],
    );
    assert_eq!(path_override.status.code(), Some(0));
    let path_override_json: Value = serde_json::from_slice(&path_override.stdout).expect("json");
    assert_eq!(path_override_json["value"], "from-file");
}

#[test]
fn config_get_json_yaml_text_quiet_and_no_color_behavior() {
    let root = temp_dir("formats");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ALPHA=from-file\n").expect("write file");
    let path = file.to_str().expect("utf-8");

    let json_out = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(json_out.status.code(), Some(0));
    let _: Value = serde_json::from_slice(&json_out.stdout).expect("json");

    let yaml_out = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "yaml",
        "--pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(yaml_out.status.code(), Some(0));
    let yaml_text = String::from_utf8(yaml_out.stdout).expect("utf-8");
    assert!(yaml_text.contains("value:"));

    let text_out =
        run(&["cli", "config", "get", "alpha", "--format", "text", "--config-path", path]);
    assert_eq!(text_out.status.code(), Some(0));
    let text = String::from_utf8(text_out.stdout).expect("utf-8");
    assert!(text.contains("alpha"));

    let quiet_out = run(&["cli", "config", "get", "alpha", "--quiet", "--config-path", path]);
    assert_eq!(quiet_out.status.code(), Some(0));
    assert!(quiet_out.stdout.is_empty());
    assert!(quiet_out.stderr.is_empty());

    let no_color_out = run_with_env(
        &["cli", "config", "get", "alpha", "--format", "text", "--config-path", path],
        &[("NO_COLOR", "1")],
    );
    assert_eq!(no_color_out.status.code(), Some(0));
    let no_color_text = String::from_utf8(no_color_out.stdout).expect("utf-8");
    assert!(!no_color_text.contains("\u{1b}["));
}

#[test]
fn config_listing_repeated_run_determinism_and_field_order_stability() {
    let root = temp_dir("determinism");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("write file");
    let path = file.to_str().expect("utf-8");

    let args = ["config", "--format", "json", "--no-pretty", "--config-path", path];
    let first = run(&args);
    let second = run(&args);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);

    let body = String::from_utf8(first.stdout).expect("utf-8");
    let alpha = body.find("\"alpha\"").expect("alpha key");
    let beta = body.find("\"beta\"").expect("beta key");
    assert!(alpha < beta, "stable field order should keep alpha before beta");
}
