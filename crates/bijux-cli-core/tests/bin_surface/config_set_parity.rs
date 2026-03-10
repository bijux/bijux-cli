#![forbid(unsafe_code)]
//! Config set command parity and snapshot coverage.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-set-bin-{name}-{nanos}"));
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

fn run_with_stdin(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn");

    use std::io::Write;
    child.stdin.as_mut().expect("stdin").write_all(input.as_bytes()).expect("write stdin");

    child.wait_with_output().expect("wait")
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
fn config_set_output_snapshots_text_json_yaml() {
    let temp = make_temp_dir("snapshots");
    let config_path = temp.join("set.env");
    let path = config_path.to_str().expect("utf-8");

    let text = run(&["cli", "config", "set", "alpha=1", "--format", "text", "--config-path", path]);
    assert!(text.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(text.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_set_text.txt")
    );

    let pretty = run(&[
        "cli",
        "config",
        "set",
        "alpha=1",
        "--format",
        "json",
        "--pretty",
        "--config-path",
        path,
    ]);
    assert!(pretty.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(pretty.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_set_json_pretty.txt")
    );

    let compact = run(&[
        "cli",
        "config",
        "set",
        "alpha=1",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        path,
    ]);
    assert!(compact.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(compact.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_set_json_compact.txt")
    );

    let yaml = run(&[
        "cli",
        "config",
        "set",
        "alpha=1",
        "--format",
        "yaml",
        "--pretty",
        "--config-path",
        path,
    ]);
    assert!(yaml.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(yaml.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_set_yaml_pretty.txt")
    );
}

#[test]
fn config_set_accepts_direct_and_stdin_key_value_pairs() {
    let temp = make_temp_dir("stdin");
    let config_path = temp.join("set.env");
    let path = config_path.to_str().expect("utf-8");

    let direct = run(&["cli", "config", "set", "alpha=1", "--config-path", path]);
    assert_eq!(direct.status.code(), Some(0));

    let stdin = run_with_stdin(&["cli", "config", "set", "--config-path", path], "beta=2\n");
    assert_eq!(stdin.status.code(), Some(0));

    let content = fs::read_to_string(config_path).expect("config file");
    assert!(content.contains("BIJUXCLI_ALPHA=1"));
    assert!(content.contains("BIJUXCLI_BETA=2"));
}

#[test]
fn config_set_rejects_missing_separator_and_empty_key() {
    let temp = make_temp_dir("invalid");
    let config_path = temp.join("set.env");
    let path = config_path.to_str().expect("utf-8");

    let missing_separator = run(&["cli", "config", "set", "invalid", "--config-path", path]);
    assert_eq!(missing_separator.status.code(), Some(2));
    assert!(missing_separator.stdout.is_empty());
    assert!(!missing_separator.stderr.is_empty());

    let empty_key = run(&["cli", "config", "set", "=1", "--config-path", path]);
    assert_eq!(empty_key.status.code(), Some(2));
    assert!(empty_key.stdout.is_empty());
    assert!(!empty_key.stderr.is_empty());
}

#[test]
fn config_set_preserves_existing_entries_and_stable_ordering() {
    let temp = make_temp_dir("ordering");
    let config_path = temp.join("set.env");
    fs::write(&config_path, "BIJUXCLI_ZETA=9\n").expect("seed");
    let path = config_path.to_str().expect("utf-8");

    let out_a = run(&["cli", "config", "set", "alpha=1", "--config-path", path]);
    assert_eq!(out_a.status.code(), Some(0));

    let out_b = run(&["cli", "config", "set", "zeta=10", "--config-path", path]);
    assert_eq!(out_b.status.code(), Some(0));

    let content = fs::read_to_string(config_path).expect("config file");
    let lines: Vec<&str> = content.lines().collect();
    assert_eq!(lines, vec!["BIJUXCLI_ALPHA=1", "BIJUXCLI_ZETA=10"]);
}

#[test]
#[cfg(unix)]
fn config_set_keeps_original_file_on_write_failure() {
    use std::os::unix::fs::PermissionsExt;

    let temp = make_temp_dir("rollback");
    let read_only_dir = temp.join("readonly");
    fs::create_dir_all(&read_only_dir).expect("mkdir");

    let config_path = read_only_dir.join("set.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("seed");

    fs::set_permissions(&read_only_dir, fs::Permissions::from_mode(0o555)).expect("chmod");

    let out = run(&[
        "cli",
        "config",
        "set",
        "beta=2",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    fs::set_permissions(&read_only_dir, fs::Permissions::from_mode(0o755)).expect("restore chmod");
    let content = fs::read_to_string(&config_path).expect("config file");
    assert_eq!(content, "BIJUXCLI_ALPHA=1\n");
}

#[test]
fn config_set_stream_routing_and_python_parity() {
    let temp = make_temp_dir("python-parity");
    let config_path = temp.join("set.env");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py_ok = run_python(&["config", "set", "alpha=1", "--format", "json", "--no-pretty"], &envs);
    let rs_ok = run_with_env(
        &[
            "cli",
            "config",
            "set",
            "alpha=1",
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

    assert_eq!(py_ok.status.code(), rs_ok.status.code());
    assert!(py_ok.stderr.is_empty());
    assert!(rs_ok.stderr.is_empty());

    let py_ok_json: Value = serde_json::from_slice(&py_ok.stdout).expect("py json");
    let rs_ok_json: Value = serde_json::from_slice(&rs_ok.stdout).expect("rs json");
    assert_eq!(py_ok_json["status"], rs_ok_json["status"]);
    assert_eq!(py_ok_json["key"], rs_ok_json["key"]);

    let py_bad =
        run_python(&["config", "set", "invalid", "--format", "json", "--no-pretty"], &envs);
    let rs_bad = run_with_env(
        &[
            "cli",
            "config",
            "set",
            "invalid",
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

    assert_eq!(py_bad.status.code(), rs_bad.status.code());
    assert!(py_bad.stdout.is_empty());
    assert!(rs_bad.stdout.is_empty());
    assert!(!py_bad.stderr.is_empty());
    assert!(!rs_bad.stderr.is_empty());
}
