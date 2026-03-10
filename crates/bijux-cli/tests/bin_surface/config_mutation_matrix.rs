#![forbid(unsafe_code)]
//! Config mutation behavior matrix coverage.
//! test_type: config-mutation-rollback

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

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bijux-config-mutation-matrix-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

#[test]
fn config_set_create_replace_preserve_quoted_spaces_and_invalid_key() {
    let root = temp_dir("set-cases");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ZETA=9\n").expect("seed");
    let path = file.to_str().expect("utf-8");

    let create = run(&["cli", "config", "set", "alpha=1", "--config-path", path]);
    assert_eq!(create.status.code(), Some(0));

    let replace = run(&["cli", "config", "set", "alpha=2", "--config-path", path]);
    assert_eq!(replace.status.code(), Some(0));

    let quoted = run(&[
        "cli",
        "config",
        "set",
        "quoted=\"hello\"",
        "--config-path",
        path,
    ]);
    assert_eq!(quoted.status.code(), Some(0));

    let spaces = run(&[
        "cli",
        "config",
        "set",
        "message=hello world",
        "--config-path",
        path,
    ]);
    assert_eq!(spaces.status.code(), Some(0));

    let invalid = run(&["cli", "config", "set", "bad-key=1", "--config-path", path]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(!invalid.stderr.is_empty());

    let body = fs::read_to_string(&file).expect("config body");
    assert!(body.contains("BIJUXCLI_ALPHA=2"));
    assert!(body.contains("BIJUXCLI_ZETA=9"));
    assert!(body.contains("BIJUXCLI_QUOTED=hello") || body.contains("BIJUXCLI_QUOTED=\"hello\""));
    assert!(body.contains("BIJUXCLI_MESSAGE=hello world"));
}

#[test]
fn config_unset_existing_and_missing_keys() {
    let root = temp_dir("unset-cases");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed");
    let path = file.to_str().expect("utf-8");

    let existing = run(&[
        "cli",
        "config",
        "unset",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(existing.status.code(), Some(0));
    let existing_json: Value = serde_json::from_slice(&existing.stdout).expect("json");
    assert_eq!(existing_json["removed"], true);

    let missing = run(&[
        "cli",
        "config",
        "unset",
        "missing",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(missing.status.code(), Some(0));
    let missing_json: Value = serde_json::from_slice(&missing.stdout).expect("json");
    assert_eq!(missing_json["removed"], false);
}

#[test]
fn config_clear_populated_and_empty_and_reload_after_external_change() {
    let root = temp_dir("clear-reload");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed");
    let path = file.to_str().expect("utf-8");

    let clear_populated = run(&[
        "cli",
        "config",
        "clear",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(clear_populated.status.code(), Some(0));
    let clear_json: Value = serde_json::from_slice(&clear_populated.stdout).expect("json");
    assert_eq!(clear_json["removed_keys"], 2);

    let clear_empty = run(&[
        "cli",
        "config",
        "clear",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(clear_empty.status.code(), Some(0));
    let clear_empty_json: Value = serde_json::from_slice(&clear_empty.stdout).expect("json");
    assert_eq!(clear_empty_json["removed_keys"], 0);

    fs::write(&file, "BIJUXCLI_GAMMA=3\n").expect("external update");
    let reload = run(&[
        "cli",
        "config",
        "reload",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert_eq!(reload.status.code(), Some(0));
    let reload_json: Value = serde_json::from_slice(&reload.stdout).expect("json");
    assert_eq!(reload_json["entry_count"], 1);
}

#[test]
fn config_export_text_json_yaml_and_load_valid_malformed() {
    let root = temp_dir("export-load");
    let active = root.join("active.env");
    let exported = root.join("exported.env");
    let valid = root.join("valid.env");
    let malformed = root.join("malformed.env");

    fs::write(&active, "BIJUXCLI_ALPHA=1\n").expect("seed active");

    let text = run(&[
        "cli",
        "config",
        "export",
        exported.to_str().expect("utf-8"),
        "--format",
        "text",
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(text.status.code(), Some(2));
    assert!(text.stdout.is_empty());
    assert!(!text.stderr.is_empty());

    let json = run(&[
        "cli",
        "config",
        "export",
        exported.to_str().expect("utf-8"),
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(json.status.code(), Some(0));

    let yaml = run(&[
        "cli",
        "config",
        "export",
        exported.to_str().expect("utf-8"),
        "--format",
        "yaml",
        "--pretty",
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(yaml.status.code(), Some(0));

    fs::write(&valid, "BIJUXCLI_BETA=2\n").expect("seed valid");
    let load_valid = run(&[
        "cli",
        "config",
        "load",
        valid.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(load_valid.status.code(), Some(0));

    fs::write(&malformed, "BROKEN\n").expect("seed malformed");
    let load_malformed = run(&[
        "cli",
        "config",
        "load",
        malformed.to_str().expect("utf-8"),
        "--config-path",
        active.to_str().expect("utf-8"),
    ]);
    assert_eq!(load_malformed.status.code(), Some(2));
    assert!(load_malformed.stdout.is_empty());
    assert!(!load_malformed.stderr.is_empty());
}

#[test]
fn config_mutation_rollback_and_retry_idempotency_proof() {
    let root = temp_dir("rollback");
    let file = root.join("cfg.env");
    fs::write(&file, "BIJUXCLI_ALPHA=1\n").expect("seed");
    let path = file.to_str().expect("utf-8");

    let malformed = root.join("malformed.env");
    fs::write(&malformed, "BROKEN\n").expect("seed malformed");

    let failed = run(&[
        "cli",
        "config",
        "load",
        malformed.to_str().expect("utf-8"),
        "--config-path",
        path,
    ]);
    assert_eq!(failed.status.code(), Some(2));

    let after_failed = fs::read_to_string(&file).expect("read after failed load");
    assert_eq!(after_failed, "BIJUXCLI_ALPHA=1\n");

    let valid = root.join("valid.env");
    fs::write(&valid, "BIJUXCLI_ALPHA=2\n").expect("seed valid");

    let retry_one = run(&[
        "cli",
        "config",
        "load",
        valid.to_str().expect("utf-8"),
        "--config-path",
        path,
    ]);
    assert_eq!(retry_one.status.code(), Some(0));

    let retry_two = run(&[
        "cli",
        "config",
        "load",
        valid.to_str().expect("utf-8"),
        "--config-path",
        path,
    ]);
    assert_eq!(retry_two.status.code(), Some(0));

    let final_body = fs::read_to_string(&file).expect("read final");
    assert_eq!(final_body, "BIJUXCLI_ALPHA=2\n");
}
