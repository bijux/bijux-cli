#![forbid(unsafe_code)]
//! Config unset/clear/reload parity and snapshot coverage.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-mutation-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn python_cli() -> String {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().and_then(|p| p.parent()).expect("workspace root");
    root.join("bin").join("bijux").display().to_string()
}

fn run_python(args: &[&str], envs: &HashMap<String, String>) -> Output {
    let mut cmd = Command::new(python_cli());
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("python cli")
}

fn normalize_snapshot(stdout: String, config_path: &str) -> String {
    stdout.replace(config_path, "<CONFIG_PATH>")
}

#[test]
fn config_unset_clear_reload_text_snapshots() {
    let temp = make_temp_dir("snapshots");
    let config_path = temp.join("config.env");
    let path = config_path.to_str().expect("utf-8");

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed unset");
    let unset =
        run(&["cli", "config", "unset", "alpha", "--format", "text", "--config-path", path]);
    assert!(unset.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(unset.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_unset_text.txt")
    );

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed clear");
    let clear = run(&["cli", "config", "clear", "--format", "text", "--config-path", path]);
    assert!(clear.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(clear.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_clear_text.txt")
    );

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("seed reload");
    let reload = run(&["cli", "config", "reload", "--format", "text", "--config-path", path]);
    assert!(reload.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(reload.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_reload_text.txt")
    );
}

#[test]
fn config_unset_behaves_for_existing_missing_and_invalid_keys() {
    let temp = make_temp_dir("unset");
    let config_path = temp.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed");
    let path = config_path.to_str().expect("utf-8");

    let existing = run(&["cli", "config", "unset", "alpha", "--config-path", path]);
    assert_eq!(existing.status.code(), Some(0));
    let existing_json: Value = serde_json::from_slice(&existing.stdout).expect("json");
    assert_eq!(existing_json["status"], "deleted");
    assert_eq!(existing_json["removed"], true);

    let missing = run(&["cli", "config", "unset", "missing", "--config-path", path]);
    assert_eq!(missing.status.code(), Some(0));
    let missing_json: Value = serde_json::from_slice(&missing.stdout).expect("json");
    assert_eq!(missing_json["removed"], false);

    let invalid = run(&["cli", "config", "unset", "bad-key", "--config-path", path]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(!invalid.stderr.is_empty());
}

#[test]
fn config_clear_and_reload_handle_missing_and_malformed_files() {
    let temp = make_temp_dir("clear-reload");
    let missing = temp.join("missing.env");
    let malformed = temp.join("malformed.env");
    fs::write(&malformed, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("seed malformed");

    let clear_missing =
        run(&["cli", "config", "clear", "--config-path", missing.to_str().expect("utf-8")]);
    assert_eq!(clear_missing.status.code(), Some(0));
    let clear_missing_json: Value = serde_json::from_slice(&clear_missing.stdout).expect("json");
    assert_eq!(clear_missing_json["removed_keys"], 0);

    let reload_missing =
        run(&["cli", "config", "reload", "--config-path", missing.to_str().expect("utf-8")]);
    assert_eq!(reload_missing.status.code(), Some(0));
    let reload_missing_json: Value = serde_json::from_slice(&reload_missing.stdout).expect("json");
    assert_eq!(reload_missing_json["entry_count"], 0);

    let reload_malformed =
        run(&["cli", "config", "reload", "--config-path", malformed.to_str().expect("utf-8")]);
    assert_eq!(reload_malformed.status.code(), Some(1));
    assert!(reload_malformed.stdout.is_empty());
    assert!(!reload_malformed.stderr.is_empty());
}

#[test]
#[cfg(unix)]
fn config_clear_reports_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = make_temp_dir("clear-write-failure");
    let dir = temp.join("readonly");
    fs::create_dir_all(&dir).expect("mkdir");
    let config_path = dir.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed");

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o555)).expect("chmod");

    let out =
        run(&["cli", "config", "clear", "--config-path", config_path.to_str().expect("utf-8")]);

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    fs::set_permissions(&dir, fs::Permissions::from_mode(0o755)).expect("restore");
}

#[test]
fn config_mutation_python_parity_for_exit_and_streams() {
    let temp = make_temp_dir("python-parity");
    let config_path = temp.join("config.env");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed unset parity");
    let py_unset =
        run_python(&["config", "unset", "alpha", "--format", "json", "--no-pretty"], &envs);
    let rs_unset = run_with_env(
        &[
            "cli",
            "config",
            "unset",
            "alpha",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_CONFIG", config_path.display().to_string()),
            ("HOME", temp.display().to_string()),
            ("NO_COLOR", "1".to_string()),
        ],
    );
    assert_eq!(py_unset.status.code(), rs_unset.status.code());
    assert!(py_unset.stderr.is_empty());
    assert!(rs_unset.stderr.is_empty());

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed clear parity");
    let py_clear = run_python(&["config", "clear", "--format", "json", "--no-pretty"], &envs);
    let rs_clear = run_with_env(
        &[
            "cli",
            "config",
            "clear",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_CONFIG", config_path.display().to_string()),
            ("HOME", temp.display().to_string()),
            ("NO_COLOR", "1".to_string()),
        ],
    );
    assert_eq!(py_clear.status.code(), rs_clear.status.code());
    assert!(py_clear.stderr.is_empty());
    assert!(rs_clear.stderr.is_empty());

    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed reload parity");
    let py_reload = run_python(&["config", "reload", "--format", "json", "--no-pretty"], &envs);
    let rs_reload = run_with_env(
        &[
            "cli",
            "config",
            "reload",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &[
            ("BIJUXCLI_CONFIG", config_path.display().to_string()),
            ("HOME", temp.display().to_string()),
            ("NO_COLOR", "1".to_string()),
        ],
    );
    assert_eq!(py_reload.status.code(), rs_reload.status.code());
    assert!(py_reload.stderr.is_empty());
    assert!(rs_reload.stderr.is_empty());
}
