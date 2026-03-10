#![forbid(unsafe_code)]
//! Output and route coverage contracts for dev cli command surfaces.

use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use shlex as _;
use thiserror as _;

fn run_raw(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

#[test]
fn dev_cli_commands_support_json_and_text_flags() {
    let cases = [
        ("parity", "rust_python"),
        ("crate-health", "crate_metrics"),
        ("route-audit", "summary"),
        ("script-audit", "scripts"),
        ("docs-audit", "docs_audit"),
        ("state-audit", "paths"),
        ("runtime-identity", "entrypoints"),
    ];

    for (command, key) in cases {
        let json_out = run_raw(&["dev", "cli", command, "--json"]);
        assert!(json_out.status.success(), "json run failed for {command}");
        let json_text = String::from_utf8(json_out.stdout).expect("json utf-8");
        let payload: serde_json::Value = serde_json::from_str(&json_text).expect("valid json");
        assert!(payload.get(key).is_some(), "missing key `{key}` for command `{command}`");

        let text_out = run_raw(&["dev", "cli", command, "--text"]);
        assert!(text_out.status.success(), "text run failed for {command}");
        let text = String::from_utf8(text_out.stdout).expect("text utf-8");
        assert!(
            text.contains(key),
            "text output missing key marker `{key}` for command `{command}`"
        );
        assert!(!text.trim().is_empty(), "text output should not be empty for `{command}`");
    }
}

#[test]
fn dev_cli_failure_snapshots_are_stable_for_json_and_text() {
    let json_out = run_raw(&["dev", "cli", "does-not-exist", "--json"]);
    assert_eq!(json_out.status.code(), Some(2));
    assert!(json_out.stdout.is_empty());
    let json_err = String::from_utf8(json_out.stderr).expect("stderr utf-8");
    assert_eq!(json_err, include_str!("../snapshots/dev_cli_unknown_json.txt"));

    let text_out = run_raw(&["dev", "cli", "does-not-exist", "--text"]);
    assert_eq!(text_out.status.code(), Some(2));
    assert!(text_out.stdout.is_empty());
    let text_err = String::from_utf8(text_out.stderr).expect("stderr utf-8");
    assert_eq!(text_err, include_str!("../snapshots/dev_cli_unknown_text.txt"));
}
