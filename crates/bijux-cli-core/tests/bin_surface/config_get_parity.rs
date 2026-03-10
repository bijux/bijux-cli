#![forbid(unsafe_code)]
//! Config get command parity and snapshot coverage.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
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
    let path = std::env::temp_dir().join(format!("bijux-config-get-bin-{name}-{nanos}"));
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
fn config_get_output_snapshots_text_json_yaml() {
    let temp = make_temp_dir("snapshots");
    let config_path = temp.join("get.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");
    let path = config_path.to_str().expect("utf-8");

    let text = run(&["cli", "config", "get", "alpha", "--format", "text", "--config-path", path]);
    assert!(text.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(text.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_get_text.txt")
    );

    let pretty = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--pretty",
        "--config-path",
        path,
    ]);
    assert!(pretty.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(pretty.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_get_json_pretty.txt")
    );

    let compact = run(&[
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
    assert!(compact.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(compact.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_get_json_compact.txt")
    );

    let yaml = run(&[
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
    assert!(yaml.status.success());
    assert_eq!(
        normalize_snapshot(String::from_utf8(yaml.stdout).expect("utf-8"), path),
        include_str!("../snapshots/config_get_yaml_pretty.txt")
    );
}

#[test]
fn config_get_found_missing_invalid_and_normalized_keys() {
    let temp = make_temp_dir("matrix");
    let config_path = temp.join("get.env");
    fs::write(&config_path, "BIJUXCLI_MIXED=1\n").expect("write config");
    let path = config_path.to_str().expect("utf-8");

    let found = run(&["cli", "config", "get", "mixed", "--config-path", path]);
    assert_eq!(found.status.code(), Some(0));
    let found_json: Value = serde_json::from_slice(&found.stdout).expect("json");
    assert_eq!(found_json["value"], "1");

    let normalized = run(&["cli", "config", "get", "BIJUXCLI_MIXED", "--config-path", path]);
    assert_eq!(normalized.status.code(), Some(0));
    let normalized_json: Value = serde_json::from_slice(&normalized.stdout).expect("json");
    assert_eq!(normalized_json["key"], "mixed");

    let missing = run(&["cli", "config", "get", "missing", "--config-path", path]);
    assert_eq!(missing.status.code(), Some(2));
    assert!(missing.stdout.is_empty());
    assert!(!missing.stderr.is_empty());

    let invalid = run(&["cli", "config", "get", "bad-key", "--config-path", path]);
    assert_eq!(invalid.status.code(), Some(2));
    assert!(invalid.stdout.is_empty());
    assert!(!invalid.stderr.is_empty());
}

#[test]
fn config_get_path_override_malformed_quiet_no_color_and_trace() {
    let temp = make_temp_dir("behavior");
    let env_path = temp.join("env.env");
    let flag_path = temp.join("flag.env");
    let bad_path = temp.join("bad.env");
    fs::write(&env_path, "BIJUXCLI_ALPHA=env\n").expect("write env");
    fs::write(&flag_path, "BIJUXCLI_ALPHA=flag\n").expect("write flag");
    fs::write(&bad_path, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write bad");

    let override_out = run_with_env(
        &["cli", "config", "get", "alpha", "--config-path", flag_path.to_str().expect("utf-8")],
        &[("BIJUXCLI_CONFIG", env_path.display().to_string())],
    );
    assert_eq!(override_out.status.code(), Some(0));
    let override_json: Value = serde_json::from_slice(&override_out.stdout).expect("json");
    assert_eq!(override_json["value"], "flag");

    let malformed =
        run(&["cli", "config", "get", "alpha", "--config-path", bad_path.to_str().expect("utf-8")]);
    assert_eq!(malformed.status.code(), Some(1));
    assert!(malformed.stdout.is_empty());
    assert!(!malformed.stderr.is_empty());

    let quiet = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--quiet",
        "--config-path",
        flag_path.to_str().expect("utf-8"),
    ]);
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let no_color = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "alpha",
            "--format",
            "text",
            "--config-path",
            flag_path.to_str().expect("utf-8"),
        ],
        &[("NO_COLOR", "1".to_string())],
    );
    assert_eq!(no_color.status.code(), Some(0));
    let no_color_stdout = String::from_utf8(no_color.stdout).expect("utf-8");
    assert!(!no_color_stdout.contains("\u{1b}["));

    let base = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        flag_path.to_str().expect("utf-8"),
    ]);
    let traced = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--log-level",
        "trace",
        "--config-path",
        flag_path.to_str().expect("utf-8"),
    ]);
    assert_eq!(base.status.code(), Some(0));
    assert_eq!(traced.status.code(), Some(0));
    assert_eq!(base.stdout, traced.stdout);
}

#[test]
fn config_get_python_parity_for_success_and_missing() {
    let temp = make_temp_dir("python-parity");
    let config_path = temp.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py_ok = run_python(&["config", "get", "alpha", "--format", "json", "--no-pretty"], &envs);
    let rs_ok = run_with_env(
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
    assert_eq!(py_ok_json["value"], rs_ok_json["value"]);

    let py_missing =
        run_python(&["config", "get", "missing", "--format", "json", "--no-pretty"], &envs);
    let rs_missing = run_with_env(
        &[
            "cli",
            "config",
            "get",
            "missing",
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
    assert_eq!(py_missing.status.code(), rs_missing.status.code());
    assert!(py_missing.stdout.is_empty());
    assert!(rs_missing.stdout.is_empty());
    assert!(!py_missing.stderr.is_empty());
    assert!(!rs_missing.stderr.is_empty());
}
