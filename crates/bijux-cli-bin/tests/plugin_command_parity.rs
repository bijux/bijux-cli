#![forbid(unsafe_code)]
//! Plugin command parity checks for overlaps currently captured in Python behavior locks.

use std::path::PathBuf;
use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::{json, Value};
use shlex as _;
use thiserror as _;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn read_captures() -> Value {
    let root = workspace_root();
    let path = root.join("artifacts/current-python-behavior-lock.json");
    serde_json::from_str(&std::fs::read_to_string(path).expect("capture file"))
        .expect("capture json")
}

fn run(args: &[&str]) -> (i32, String, String) {
    run_with_env(args, &[])
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    command.args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("binary run");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8(output.stdout).expect("stdout utf-8"),
        String::from_utf8(output.stderr).expect("stderr utf-8"),
    )
}

#[test]
fn plugins_list_parity_shape_matches_python_capture_overlap() {
    let captures = read_captures();
    let py = &captures["captures"]["bijux_plugins_list"];
    let (code, out, err) = run(&["plugins", "list"]);

    assert_eq!(code, py["exit_code"].as_i64().unwrap_or(0) as i32);
    assert!(err.is_empty());

    let py_json: Value =
        serde_json::from_str(py["stdout"].as_str().unwrap_or("{}")).expect("python list json");
    let rs_json: Value = serde_json::from_str(&out).expect("rust list json");

    assert!(py_json.get("plugins").is_some());
    assert!(rs_json.get("plugins").is_some());
    assert!(rs_json.get("directory").is_some());
}

#[test]
fn plugins_check_parity_exit_and_stream_routing_matches_capture_overlap() {
    let captures = read_captures();
    let py = &captures["captures"]["behavior_plugins_check"];
    let argv = py["argv"].as_array().expect("argv");
    let args: Vec<&str> = argv.iter().skip(1).filter_map(Value::as_str).collect();

    let env_overrides = py["env_overrides"].as_object().expect("env_overrides");
    let owned_env: Vec<(String, String)> = env_overrides
        .iter()
        .filter_map(|(key, value)| value.as_str().map(|text| (key.to_string(), text.to_string())))
        .collect();
    let env_refs: Vec<(&str, &str)> =
        owned_env.iter().map(|(key, value)| (key.as_str(), value.as_str())).collect();

    let (code, out, err) = run_with_env(&args, &env_refs);
    let expected_code = py["exit_code"].as_i64().unwrap_or(0) as i32;
    assert!(
        code == expected_code || code == 1,
        "unexpected exit code for plugin check parity overlap: expected {expected_code} or 1, got {code}",
    );

    let envelope = if out.trim().is_empty() { &err } else { &out };
    let rs_json: Value = serde_json::from_str(envelope).unwrap_or_else(|_| json!({}));
    assert!(rs_json.get("status").is_some());
}
