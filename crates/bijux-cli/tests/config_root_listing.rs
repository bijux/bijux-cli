#![forbid(unsafe_code)]
//! Root config listing behavior and parity checks.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow as _;
use bijux_cli::app::run_app;
use clap as _;
use futures as _;
use serde_json::Value;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-config-root-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("valid json")
}

#[test]
fn root_config_lists_all_active_file_backed_entries() {
    let temp = make_temp_dir("list");
    let path = temp.join("config.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\nBIJUXCLI_BETA=2\n").expect("write config");

    let out = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 0);
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["alpha"], "1");
    assert_eq!(payload["beta"], "2");
}

#[test]
fn root_config_empty_file_returns_empty_object() {
    let temp = make_temp_dir("empty");
    let path = temp.join("empty.env");
    fs::write(&path, "").expect("write empty");

    let out = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 0);
    let payload = parse_json(&out.stdout);
    assert_eq!(payload, serde_json::json!({}));
}

#[test]
fn root_config_malformed_file_is_error() {
    let temp = make_temp_dir("malformed");
    let path = temp.join("bad.env");
    fs::write(&path, "BIJUXCLI_OK=1\nMALFORMED\n").expect("write malformed");

    let out = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 1);
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn root_config_duplicate_keys_keep_latest_value() {
    let temp = make_temp_dir("duplicate");
    let path = temp.join("dupe.env");
    fs::write(&path, "BIJUXCLI_ALPHA=old\nBIJUXCLI_ALPHA=new\n").expect("write dupes");

    let out = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("run_app");

    assert_eq!(out.exit_code, 0);
    let payload = parse_json(&out.stdout);
    assert_eq!(payload["alpha"], "new");
}

#[test]
fn root_config_trace_mode_does_not_mutate_payload() {
    let temp = make_temp_dir("trace");
    let path = temp.join("trace.env");
    fs::write(&path, "BIJUXCLI_ALPHA=1\n").expect("write");

    let baseline = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("baseline");
    let traced = run_app(&[
        "bijux".to_string(),
        "config".to_string(),
        "--log-level".to_string(),
        "trace".to_string(),
        "--config-path".to_string(),
        path.display().to_string(),
    ])
    .expect("trace");

    assert_eq!(baseline.exit_code, 0);
    assert_eq!(traced.exit_code, 0);
    assert_eq!(parse_json(&baseline.stdout), parse_json(&traced.stdout));
}
