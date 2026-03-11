#![forbid(unsafe_code)]
//! Binary-level memory parity tests.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-memory-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn python_cli() -> String {
    if let Ok(path) = std::env::var("BIJUX_REFERENCE_CLI") {
        if !path.trim().is_empty() {
            return path;
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root = manifest_dir.parent().and_then(|p| p.parent()).expect("workspace root");
    let legacy = root.join("bin").join("bijux");
    if legacy.exists() {
        return legacy.display().to_string();
    }

    env!("CARGO_BIN_EXE_bijux").to_string()
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

#[test]
fn memory_root_and_list_support_json_yaml_and_text() {
    let temp = make_temp_dir("formats");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "{\"beta\":2,\"alpha\":1}").expect("write memory");
    let envs = [("HOME", home.display().to_string())];

    let root_json = run_with_env(&["memory", "--format", "json", "--no-pretty"], &envs);
    assert!(root_json.status.success());
    let root_payload = parse_json(&root_json.stdout);
    assert_eq!(root_payload["status"], "ok");
    assert_eq!(root_payload["count"], 2);

    let list_json = run_with_env(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    assert!(list_json.status.success());
    let list_payload = parse_json(&list_json.stdout);
    assert_eq!(list_payload["status"], "ok");
    assert_eq!(list_payload["keys"][0], "alpha");
    assert_eq!(list_payload["keys"][1], "beta");

    let list_yaml = run_with_env(&["memory", "list", "--format", "yaml", "--pretty"], &envs);
    assert!(list_yaml.status.success());
    let yaml = String::from_utf8(list_yaml.stdout).expect("yaml utf-8");
    assert_eq!(yaml, include_str!("../../../data/golden/cli_surface/memory_list_yaml.txt"));

    let list_text = run_with_env(&["memory", "list", "--format", "text"], &envs);
    assert!(list_text.status.success());
    let text = String::from_utf8(list_text.stdout).expect("text utf-8");
    assert_eq!(text, include_str!("../../../data/golden/cli_surface/memory_list_text.txt"));
}

#[test]
fn memory_missing_state_is_empty_and_quiet_mode_suppresses_output() {
    let temp = make_temp_dir("missing");
    let home = temp.join("home");
    fs::create_dir_all(home.join(".bijux")).expect("mkdir");

    let out = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["count"], 0);

    let quiet =
        run_with_env(&["memory", "list", "--quiet"], &[("HOME", home.display().to_string())]);
    assert!(quiet.status.success());
    assert!(quiet.stdout.is_empty());
    assert!(quiet.stderr.is_empty());
}

#[test]
fn memory_wrong_type_state_and_missing_state_file_behaviors_are_stable() {
    let temp = make_temp_dir("wrong-type-and-missing");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "{\"alpha\":1,\"beta\":{\"nested\":1}}")
        .expect("write wrong-type memory");

    let out_wrong_type = run_with_env(
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert_eq!(out_wrong_type.status.code(), Some(0));
    let doctor_payload = parse_json(&out_wrong_type.stdout);
    assert_eq!(doctor_payload["doctor"]["status"], "degraded");
    assert!(doctor_payload["doctor"]["issues"].as_array().is_some_and(|rows| !rows.is_empty()));

    fs::remove_file(&memory_file).expect("remove memory");
    let out_missing = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert_eq!(out_missing.status.code(), Some(0));
    let missing_payload = parse_json(&out_missing.stdout);
    assert_eq!(missing_payload["status"], "ok");
    assert_eq!(missing_payload["count"], 0);
}

#[test]
fn memory_non_object_json_state_fails_with_error_envelope() {
    let temp = make_temp_dir("non-object");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "[]").expect("write non-object json");

    let out = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let payload = parse_json(&out.stderr);
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["code"], 1);
    assert_eq!(payload["command"], "memory list");
}

#[test]
fn memory_command_with_config_path_override_keeps_stable_json_contract() {
    let temp = make_temp_dir("config-env");
    let home = temp.join("home");
    let config_path = temp.join("custom.env");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "{\"home_only\":true}").expect("write memory");

    let args = [
        "memory",
        "list",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ];
    let envs = [
        ("HOME", home.display().to_string()),
        ("BIJUXCLI_CONFIG", config_path.display().to_string()),
    ];

    let out = run_with_env(&args, &envs);
    let repeat = run_with_env(
        &[
            "memory",
            "list",
            "--format",
            "json",
            "--no-pretty",
            "--config-path",
            config_path.to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert!(out.status.success());
    assert!(repeat.status.success());
    assert_eq!(out.stdout, repeat.stdout);
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["status"], "ok");
    assert!(payload.get("keys").and_then(Value::as_array).is_some());
}

#[test]
fn memory_root_parity_with_python_summary_command() {
    let temp = make_temp_dir("python-parity");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");
    fs::write(&memory_file, "{\"alpha\":1,\"beta\":2}").expect("write memory");

    let py = Command::new(python_cli())
        .args(["memory", "--format", "json", "--no-pretty"])
        .env("HOME", home.display().to_string())
        .output()
        .expect("python cli");

    let rs = run_with_env(
        &["memory", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );

    assert_eq!(py.status.code(), rs.status.code());
    let py_json = parse_json(&py.stdout);
    let rs_json = parse_json(&rs.stdout);
    assert_eq!(py_json["status"], rs_json["status"]);
    assert_eq!(py_json["count"], rs_json["count"]);
}
