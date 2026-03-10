#![forbid(unsafe_code)]
//! Root config command parity and snapshot coverage.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_routing as _;
use shlex as _;
use thiserror as _;
use bijux_cli_repl as _;
use libc as _;
use serde_json::Value;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-root-bin-{name}-{nanos}"));
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

#[test]
fn root_config_output_snapshots_text_json_yaml() {
    let temp = make_temp_dir("snapshots");
    let config_path = temp.join("root.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("write config");

    let text =
        run(&["config", "--format", "text", "--config-path", config_path.to_str().expect("utf-8")]);
    assert!(text.status.success());
    assert_eq!(
        String::from_utf8(text.stdout).expect("utf-8"),
        include_str!("snapshots/config_root_text.txt")
    );

    let pretty_json = run(&[
        "config",
        "--format",
        "json",
        "--pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert!(pretty_json.status.success());
    assert_eq!(
        String::from_utf8(pretty_json.stdout).expect("utf-8"),
        include_str!("snapshots/config_root_json_pretty.txt")
    );

    let compact_json = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert!(compact_json.status.success());
    assert_eq!(
        String::from_utf8(compact_json.stdout).expect("utf-8"),
        include_str!("snapshots/config_root_json_compact.txt")
    );

    let pretty_yaml = run(&[
        "config",
        "--format",
        "yaml",
        "--pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ]);
    assert!(pretty_yaml.status.success());
    assert_eq!(
        String::from_utf8(pretty_yaml.stdout).expect("utf-8"),
        include_str!("snapshots/config_root_yaml_pretty.txt")
    );
}

#[test]
fn root_config_quiet_and_no_color_modes() {
    let temp = make_temp_dir("quiet-no-color");
    let config_path = temp.join("root.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\n").expect("write config");

    let quiet = run(&["config", "--quiet", "--config-path", config_path.to_str().expect("utf-8")]);
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());

    let no_color = run_with_env(
        &["config", "--format", "text", "--config-path", config_path.to_str().expect("utf-8")],
        &[("NO_COLOR", "1".to_string())],
    );
    assert!(no_color.status.success());
    let stdout = String::from_utf8(no_color.stdout).expect("utf-8");
    assert!(!stdout.contains("\u{1b}["));
}

#[test]
fn root_config_stream_and_exit_code_behavior() {
    let temp = make_temp_dir("streams");
    let good = temp.join("good.env");
    let bad = temp.join("bad.env");
    fs::write(&good, "BIJUXCLI_ALPHA=1\n").expect("write good");
    fs::write(&bad, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write bad");

    let success = run(&["config", "--config-path", good.to_str().expect("utf-8")]);
    assert_eq!(success.status.code(), Some(0));
    assert!(!success.stdout.is_empty());
    assert!(success.stderr.is_empty());

    let failure = run(&["config", "--config-path", bad.to_str().expect("utf-8")]);
    assert_eq!(failure.status.code(), Some(1));
    assert!(failure.stdout.is_empty());
    assert!(!failure.stderr.is_empty());
}

#[test]
fn root_config_python_parity_output_and_exit() {
    let temp = make_temp_dir("python-parity");
    let config_path = temp.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("write config");

    let mut envs = HashMap::new();
    envs.insert("BIJUXCLI_CONFIG".to_string(), config_path.display().to_string());
    envs.insert("HOME".to_string(), temp.display().to_string());
    envs.insert("NO_COLOR".to_string(), "1".to_string());

    let py = run_python(&["config", "--format", "json", "--no-pretty"], &envs);
    let rs = run_with_env(
        &[
            "config",
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

    assert_eq!(py.status.code(), rs.status.code());
    assert!(py.stderr.is_empty());
    assert!(rs.stderr.is_empty());

    let py_json: Value = serde_json::from_slice(&py.stdout).expect("py json");
    let rs_json: Value = serde_json::from_slice(&rs.stdout).expect("rs json");
    assert_eq!(py_json, rs_json);
}

#[test]
fn root_config_empty_malformed_duplicate_override_and_trace() {
    let temp = make_temp_dir("matrix");
    let empty = temp.join("empty.env");
    let malformed = temp.join("malformed.env");
    let duplicate = temp.join("duplicate.env");
    let env_path = temp.join("env.env");
    let flag_path = temp.join("flag.env");

    fs::write(&empty, "").expect("write empty");
    fs::write(&malformed, "BIJUXCLI_ALPHA=1\nBAD\n").expect("write malformed");
    fs::write(&duplicate, "BIJUXCLI_ALPHA=old\nBIJUXCLI_ALPHA=new\n").expect("write dupes");
    fs::write(&env_path, "BIJUXCLI_ALPHA=env\n").expect("write env path");
    fs::write(&flag_path, "BIJUXCLI_ALPHA=flag\n").expect("write flag path");

    let out_empty = run(&["config", "--config-path", empty.to_str().expect("utf-8")]);
    assert_eq!(out_empty.status.code(), Some(0));
    let empty_json: Value = serde_json::from_slice(&out_empty.stdout).expect("json");
    assert_eq!(empty_json, serde_json::json!({}));

    let out_malformed = run(&["config", "--config-path", malformed.to_str().expect("utf-8")]);
    assert_eq!(out_malformed.status.code(), Some(1));
    assert!(out_malformed.stdout.is_empty());
    assert!(!out_malformed.stderr.is_empty());

    let out_duplicate = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        duplicate.to_str().expect("utf-8"),
    ]);
    assert_eq!(out_duplicate.status.code(), Some(0));
    let duplicate_json: Value = serde_json::from_slice(&out_duplicate.stdout).expect("json");
    assert_eq!(duplicate_json["alpha"], "new");

    let out_override = run_with_env(
        &["config", "--config-path", flag_path.to_str().expect("utf-8")],
        &[("BIJUXCLI_CONFIG", env_path.display().to_string())],
    );
    assert_eq!(out_override.status.code(), Some(0));
    let override_json: Value = serde_json::from_slice(&out_override.stdout).expect("json");
    assert_eq!(override_json["alpha"], "flag");

    let base = run(&[
        "config",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        flag_path.to_str().expect("utf-8"),
    ]);
    let traced = run(&[
        "config",
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
