#![forbid(unsafe_code)]
//! Plugin command parity checks for the current Rust binary and current Python facade.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn python_interpreter() -> String {
    if let Ok(python) = std::env::var("PYTHON") {
        if !python.trim().is_empty() {
            return python;
        }
    }
    for candidate in ["python3.11", "python3", "python"] {
        if Command::new(candidate).arg("--version").output().is_ok() {
            return candidate.to_string();
        }
    }
    panic!("python interpreter not found");
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> (i32, String, String) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
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

fn run_python(args: &[&str], envs: &[(&str, &str)]) -> (i32, String, String) {
    let python = python_interpreter();
    let python_root = workspace_root().join("crates/bijux-cli-python/python");
    let mut command = Command::new(python);
    command.arg("-m").arg("bijux_cli_py").args(args);
    command.env("PYTHONPATH", python_root);
    command.env("BIJUX_BIN", env!("CARGO_BIN_EXE_bijux"));
    for (key, value) in envs {
        command.env(key, value);
    }
    let output = command.output().expect("python facade run");
    (
        output.status.code().unwrap_or(1),
        String::from_utf8(output.stdout).expect("stdout utf-8"),
        String::from_utf8(output.stderr).expect("stderr utf-8"),
    )
}

fn parse_json_envelope(stdout: &str, stderr: &str, label: &str) -> Value {
    let envelope = if !stdout.trim().is_empty() {
        stdout
    } else if !stderr.trim().is_empty() {
        stderr
    } else {
        panic!("{label} produced no output");
    };
    serde_json::from_str(envelope).unwrap_or_else(|error| panic!("{label}: {error}"))
}

fn temp_plugins_dir() -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "bijux-plugin-parity-{}",
        SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos()
    ));
    fs::create_dir_all(&root).expect("create temp plugins dir");
    root
}

#[test]
fn plugins_list_parity_matches_current_python_facade() {
    let plugins_dir = temp_plugins_dir();
    let plugins_dir_text = plugins_dir.to_string_lossy().to_string();
    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir_text.as_str())];

    let args = ["plugins", "list", "--format", "json", "--no-pretty"];
    let (rust_code, rust_out, rust_err) = run_with_env(&args, &envs);
    let (python_code, python_out, python_err) = run_python(&args, &envs);

    assert_eq!(rust_code, python_code);
    assert_eq!(rust_err, python_err);

    let rust_json = parse_json_envelope(&rust_out, &rust_err, "rust list json");
    let python_json = parse_json_envelope(&python_out, &python_err, "python list json");

    assert_eq!(rust_json.get("plugins"), python_json.get("plugins"));
    assert_eq!(rust_json.get("directory"), python_json.get("directory"));
}

#[test]
fn plugins_check_parity_matches_current_python_facade() {
    let plugins_dir = temp_plugins_dir();
    let plugins_dir_text = plugins_dir.to_string_lossy().to_string();
    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir_text.as_str())];

    let args = ["plugins", "check", "--format", "json", "--no-pretty"];
    let (rust_code, rust_out, rust_err) = run_with_env(&args, &envs);
    let (python_code, python_out, python_err) = run_python(&args, &envs);

    assert_eq!(rust_code, python_code);
    assert_eq!(rust_err, python_err);

    let rust_json = parse_json_envelope(&rust_out, &rust_err, "rust check json");
    let python_json = parse_json_envelope(&python_out, &python_err, "python check json");

    assert_eq!(rust_json.get("status"), python_json.get("status"));
    assert_eq!(rust_json.get("plugins"), python_json.get("plugins"));
}
