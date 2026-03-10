#![forbid(unsafe_code)]
//! Binary-level history parity tests and snapshots.

use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::Instant;
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
    let path = std::env::temp_dir().join(format!("bijux-history-bin-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn run_with_env(args: &[&str], envs: &[(&str, String)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json output")
}

#[test]
fn history_json_yaml_text_outputs_are_emitted() {
    let temp = make_temp_dir("formats");
    let history_path = temp.join("history.json");
    fs::write(
        &history_path,
        serde_json::to_string(&vec![
            serde_json::json!({"command":"status","timestamp":1.0}),
            serde_json::json!({"command":"doctor","timestamp":2.0}),
        ])
        .expect("json"),
    )
    .expect("write");
    let envs = [("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())];

    let out_json = run_with_env(&["history", "--format", "json", "--no-pretty"], &envs);
    assert!(out_json.status.success());
    let payload = parse_json(&out_json.stdout);
    assert_eq!(payload["entries"].as_array().expect("array").len(), 2);
    let json_text = String::from_utf8(out_json.stdout).expect("json utf-8");
    assert_eq!(json_text, include_str!("../snapshots/history_root_json.txt"));

    let out_yaml = run_with_env(&["history", "--format", "yaml", "--pretty"], &envs);
    assert!(out_yaml.status.success());
    let yaml = String::from_utf8(out_yaml.stdout).expect("yaml utf-8");
    assert_eq!(yaml, include_str!("../snapshots/history_root_yaml.txt"));

    let out_text = run_with_env(&["history", "--format", "text"], &envs);
    assert!(out_text.status.success());
    let text = String::from_utf8(out_text.stdout).expect("text utf-8");
    assert_eq!(text, include_str!("../snapshots/history_root_text.txt"));
}

#[test]
fn history_missing_and_malformed_behaviors_are_stable() {
    let temp = make_temp_dir("errors");
    let missing_path = temp.join("missing.history");
    let envs_missing = [("BIJUXCLI_HISTORY_FILE", missing_path.display().to_string())];

    let out_missing = run_with_env(&["history", "--format", "json", "--no-pretty"], &envs_missing);
    assert_eq!(out_missing.status.code(), Some(0));
    let payload = parse_json(&out_missing.stdout);
    assert_eq!(payload["entries"], Value::Array(Vec::new()));

    let malformed_path = temp.join("malformed.history");
    fs::write(&malformed_path, "{\"oops\":true}").expect("write malformed");
    let envs_malformed = [("BIJUXCLI_HISTORY_FILE", malformed_path.display().to_string())];
    let out_malformed =
        run_with_env(&["history", "--format", "json", "--no-pretty"], &envs_malformed);
    assert_eq!(out_malformed.status.code(), Some(1));
    assert!(out_malformed.stdout.is_empty());
    assert!(!out_malformed.stderr.is_empty());

    let truncated_path = temp.join("truncated.history");
    fs::write(&truncated_path, "[{\"command\":\"status\"").expect("write truncated");
    let envs_truncated = [("BIJUXCLI_HISTORY_FILE", truncated_path.display().to_string())];
    let out_truncated =
        run_with_env(&["history", "--format", "json", "--no-pretty"], &envs_truncated);
    assert_eq!(out_truncated.status.code(), Some(0));
    let truncated_payload = parse_json(&out_truncated.stdout);
    assert!(truncated_payload["entries"].is_array());
}

#[test]
fn history_root_parity_with_python_for_read_only_listing() {
    let temp = make_temp_dir("python-parity");
    let history_path = temp.join("history.json");
    fs::write(
        &history_path,
        serde_json::to_string(&vec![
            serde_json::json!({"command":"status","timestamp":1.0,"success":true,"params":[],"return_code":0,"duration_ms":0.0,"raw":{}}),
            serde_json::json!({"command":"doctor","timestamp":2.0,"success":true,"params":[],"return_code":0,"duration_ms":0.0,"raw":{}}),
        ])
        .expect("json"),
    )
    .expect("write");

    let python_cli = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("bin")
        .join("bijux");

    let py = Command::new(python_cli)
        .args(["history", "--format", "json", "--no-pretty"])
        .env("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())
        .output()
        .expect("python cli");

    let rs = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );

    assert_eq!(py.status.code(), rs.status.code());
    let py_json = parse_json(&py.stdout);
    let rs_json = parse_json(&rs.stdout);

    let py_entries = py_json["entries"].as_array().expect("py entries");
    let rs_entries = rs_json["entries"].as_array().expect("rs entries");
    assert_eq!(py_entries.len(), rs_entries.len());
    assert_eq!(py_entries[0]["command"], rs_entries[0]["command"]);
    assert_eq!(py_entries[1]["command"], rs_entries[1]["command"]);
}

#[test]
fn history_handles_huge_files_with_stable_tail_limit() {
    let temp = make_temp_dir("huge");
    let history_path = temp.join("huge.json");
    let entries: Vec<Value> = (0..2_000)
        .map(|i| serde_json::json!({"command": format!("cmd-{i}"), "timestamp": i as f64}))
        .collect();
    fs::write(&history_path, serde_json::to_string(&entries).expect("json")).expect("write");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    let loaded = payload["entries"].as_array().expect("entries");
    assert_eq!(loaded.len(), 20);
    assert_eq!(
        loaded.first().and_then(|v| v.get("command")).and_then(Value::as_str),
        Some("cmd-1980")
    );
    assert_eq!(
        loaded.last().and_then(|v| v.get("command")).and_then(Value::as_str),
        Some("cmd-1999")
    );
}

#[test]
fn history_resilience_keeps_valid_entries_when_malformed_rows_are_interleaved() {
    let temp = make_temp_dir("malformed-interleaved");
    let history_path = temp.join("interleaved.json");
    fs::write(
        &history_path,
        "[{\"command\":\"status\",\"timestamp\":1},{\"command\":\"status\",\"timestamp\":2},\"bad\",null,{\"command\":\"doctor\",\"timestamp\":3}]",
    )
    .expect("write");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    let loaded = payload["entries"].as_array().expect("entries");
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0]["command"], "status");
    assert_eq!(loaded[1]["command"], "status");
    assert_eq!(loaded[2]["command"], "doctor");
}

#[test]
fn history_skips_malformed_entries_inside_json_array() {
    let temp = make_temp_dir("malformed-array");
    let history_path = temp.join("malformed-array.json");
    fs::write(
        &history_path,
        "[{\"command\":\"status\",\"timestamp\":1},\"bad\",123,{\"command\":\"doctor\",\"timestamp\":2}]",
    )
    .expect("write");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    let loaded = payload["entries"].as_array().expect("entries");
    assert_eq!(loaded.len(), 2);
}

#[test]
fn history_malformed_array_with_nested_noise_keeps_only_object_entries() {
    let temp = make_temp_dir("malformed-array-nested");
    let history_path = temp.join("malformed-array-nested.json");
    fs::write(
        &history_path,
        "[{\"command\":\"status\",\"timestamp\":1},[1,2],{\"not_command\":true},null,{\"command\":\"doctor\",\"timestamp\":2}]",
    )
    .expect("write");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    let loaded = payload["entries"].as_array().expect("entries");
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0]["command"], "status");
    assert_eq!(loaded[2]["command"], "doctor");
}

#[test]
fn history_reads_repl_line_layout_for_cli_interop() {
    let temp = make_temp_dir("repl-interop");
    let history_path = temp.join("repl.history");
    fs::write(&history_path, "status\nplugins list\nhistory\n").expect("write");

    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    assert!(out.status.success());
    let payload = parse_json(&out.stdout);
    let loaded = payload["entries"].as_array().expect("entries");
    assert_eq!(loaded.len(), 3);
    assert_eq!(loaded[0]["command"], "status");
    assert_eq!(loaded[1]["command"], "plugins list");
    assert_eq!(loaded[2]["command"], "history");
}

#[test]
fn history_oversized_file_stays_within_budget() {
    let temp = make_temp_dir("budget");
    let history_path = temp.join("budget.json");
    let entries: Vec<Value> = (0..10_000)
        .map(|i| serde_json::json!({"command": format!("cmd-{i}"), "timestamp": i as f64}))
        .collect();
    fs::write(&history_path, serde_json::to_string(&entries).expect("json")).expect("write");

    let start = Instant::now();
    let out = run_with_env(
        &["history", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_HISTORY_FILE", history_path.display().to_string())],
    );
    let elapsed = start.elapsed();
    assert!(out.status.success());
    assert!(elapsed.as_millis() < 1500, "oversized history budget exceeded: {elapsed:?}");
}
