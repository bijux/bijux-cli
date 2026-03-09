#![forbid(unsafe_code)]
//! Config key/value parity coverage for Python baseline rules.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core::app::run_app;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_routing as _;
use clap as _;
use futures as _;
use serde_json::Value;

fn make_temp_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let path = std::env::temp_dir().join(format!("bijux-cli-key-value-{name}-{nanos}"));
    fs::create_dir_all(&path).expect("mkdir");
    path
}

fn parse_json(stdout: &str) -> Value {
    serde_json::from_str(stdout).expect("valid json")
}

fn run_set(config_path: &PathBuf, pair: &str) -> (i32, String, String) {
    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "set".to_string(),
        pair.to_string(),
        "--config-path".to_string(),
        config_path.display().to_string(),
    ])
    .expect("run_app");
    (out.exit_code, out.stdout, out.stderr)
}

fn run_get(config_path: &PathBuf, key: &str) -> Value {
    let out = run_app(&[
        "bijux".to_string(),
        "cli".to_string(),
        "config".to_string(),
        "get".to_string(),
        key.to_string(),
        "--config-path".to_string(),
        config_path.display().to_string(),
    ])
    .expect("run_app");
    assert_eq!(out.exit_code, 0);
    parse_json(&out.stdout)
}

#[test]
fn key_validation_matrix_matches_baseline() {
    let temp = make_temp_dir("keys");
    let path = temp.join("config.env");

    let accepted = ["alpha=1", "MixedCase=1", "_=1", "a1b2=1", "BIJUXCLI_PREF=1"];
    for case in accepted {
        let (code, _, stderr) = run_set(&path, case);
        assert_eq!(code, 0, "expected success for {case}, got stderr={stderr}");
    }

    let rejected_usage = ["=1", "   =1", "bad-key=1", "group.key=1", "bad!key=1"];
    for case in rejected_usage {
        let (code, _, _) = run_set(&path, case);
        assert_eq!(code, 2, "expected usage failure for {case}");
    }

    let (code, _, _) = run_set(&path, "näme=1");
    assert_eq!(code, 3);
}

#[test]
fn value_acceptance_for_ascii_quotes_escaped_empty_and_spaces() {
    let temp = make_temp_dir("values");
    let path = temp.join("config.env");

    let (code_ascii, _, _) = run_set(&path, "plain=ascii");
    assert_eq!(code_ascii, 0);

    let (code_quoted, _, _) = run_set(&path, "quoted=\"quoted value\"");
    assert_eq!(code_quoted, 0);

    let (code_escaped_quote, _, _) = run_set(&path, "escaped=\"a\\\"b\"");
    assert_eq!(code_escaped_quote, 0);

    let (code_empty, _, _) = run_set(&path, "empty=");
    assert_eq!(code_empty, 0);

    let (code_spaces, _, _) = run_set(&path, "spaces=value with spaces");
    assert_eq!(code_spaces, 0);

    assert_eq!(run_get(&path, "plain")["value"], "ascii");
    assert_eq!(run_get(&path, "quoted")["value"], "quoted value");
    assert_eq!(run_get(&path, "escaped")["value"], "a\"b");
    assert_eq!(run_get(&path, "empty")["value"], "");
    assert_eq!(run_get(&path, "spaces")["value"], "value with spaces");
}

#[test]
fn value_rejects_newline_tab_and_control_sequences() {
    let temp = make_temp_dir("value-reject");
    let path = temp.join("config.env");

    let (newline_code, _, _) = run_set(&path, "n=line\nbreak");
    assert_eq!(newline_code, 3);

    let (tab_code, _, _) = run_set(&path, "t=tab\tvalue");
    assert_eq!(tab_code, 3);

    let (control_code, _, _) = run_set(&path, &format!("c=bad{}x", '\u{000B}'));
    assert_eq!(control_code, 3);
}
