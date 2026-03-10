#![forbid(unsafe_code)]
//! History and memory resilience hardening coverage.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn temp_dir(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "bijux-history-memory-hardening-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

#[test]
fn history_truncated_mixed_invalid_and_duplicate_records_remain_recoverable() {
    let temp = temp_dir("history-mixed");
    let history = temp.join("mixed.history");
    fs::write(
        &history,
        "[{\"command\":\"status\",\"timestamp\":1},\"bad\",{\"command\":\"status\",\"timestamp\":2},{\"command\":\"doctor\",\"timestamp\":\"nan\"}]",
    )
    .expect("write history");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload = parse_json(&out.stdout);
    let entries = payload["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["command"], "status");
    assert_eq!(entries[1]["command"], "status");
    assert_eq!(entries[2]["command"], "doctor");

    fs::write(&history, "[{\"command\":\"status\"").expect("write partial history");
    let partial = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
    );
    assert_eq!(partial.status.code(), Some(0));
    let partial_payload = parse_json(&partial.stdout);
    assert!(partial_payload["entries"].is_array());
}

#[test]
fn history_enormous_line_layout_is_tolerated_with_tail_limit() {
    let temp = temp_dir("history-long-line");
    let history = temp.join("lines.history");
    let long = "x".repeat(64 * 1024);
    let text = format!("status\n{long}\ndoctor\n");
    fs::write(&history, text).expect("write long-line history");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history.display().to_string())],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload = parse_json(&out.stdout);
    let entries = payload["entries"].as_array().expect("entries");
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0]["command"], "status");
    assert_eq!(entries[2]["command"], "doctor");
}

#[test]
fn memory_truncated_wrong_type_missing_fields_and_extra_fields_are_handled_safely() {
    let temp = temp_dir("memory-shapes");
    let home = temp.join("home");
    let memory_file = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory_file.parent().expect("parent")).expect("mkdir");

    fs::write(&memory_file, "{\"alpha\":").expect("write truncated object");
    let truncated = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert_eq!(truncated.status.code(), Some(0));
    let truncated_payload = parse_json(&truncated.stdout);
    assert_eq!(truncated_payload["count"], 0);

    fs::write(&memory_file, r#"{"alpha":1,"beta":{},"gamma":{"unexpected":true}}"#)
        .expect("write mixed memory");

    let list = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert_eq!(list.status.code(), Some(0));
    let list_payload = parse_json(&list.stdout);
    assert_eq!(list_payload["count"], 3);

    let doctor = run_with_env(
        &["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );
    assert!(matches!(doctor.status.code(), Some(0) | Some(1)));
    let doctor_payload = parse_json(&doctor.stdout);
    let issues = doctor_payload["doctor"]["issues"].as_array().expect("issues");
    assert!(issues.iter().any(|item| item["area"] == "memory"));
}

#[test]
#[cfg(unix)]
fn memory_commands_are_read_only_even_when_home_storage_is_unwritable() {
    use std::os::unix::fs::PermissionsExt;

    let temp = temp_dir("memory-read-only");
    let home = temp.join("home");
    let bijux_dir = home.join(".bijux");
    fs::create_dir_all(&bijux_dir).expect("mkdir");
    let memory_file = bijux_dir.join(".memory.json");
    fs::write(&memory_file, r#"{"alpha":{}}"#).expect("seed memory");

    fs::set_permissions(&bijux_dir, fs::Permissions::from_mode(0o555)).expect("chmod");

    let out = run_with_env(
        &["memory", "list", "--format", "json", "--no-pretty"],
        &[("HOME", home.display().to_string())],
    );

    fs::set_permissions(&bijux_dir, fs::Permissions::from_mode(0o755)).expect("restore");

    assert_eq!(out.status.code(), Some(0));
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["count"], 1);
}
