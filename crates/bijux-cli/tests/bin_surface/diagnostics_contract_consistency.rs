#![forbid(unsafe_code)]
//! Diagnostics contract consistency checks for JSON shape, text output, and exit codes.

use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn parse_json_stdout(output: &std::process::Output) -> Value {
    serde_json::from_slice::<Value>(&output.stdout).expect("valid json")
}

#[test]
fn diagnostics_json_shape_is_consistent_across_dev_commands() {
    let cases = [
        ["dev", "cli", "doctor", "--format", "json", "--no-pretty"].as_slice(),
        ["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"].as_slice(),
        ["dev", "cli", "state-audit", "--format", "json", "--no-pretty"].as_slice(),
        ["dev", "cli", "plugin-health", "--format", "json", "--no-pretty"].as_slice(),
        ["dev", "cli", "package-health", "--format", "json", "--no-pretty"].as_slice(),
        ["dev", "cli", "route-audit", "--format", "json", "--no-pretty"].as_slice(),
    ];

    for args in cases {
        let output = run(args);
        assert_eq!(output.status.code(), Some(0), "non-zero exit for {args:?}");
        let payload = parse_json_stdout(&output);
        assert!(payload.is_object(), "payload is not object for {args:?}");
        assert!(
            payload.as_object().is_some_and(|obj| !obj.is_empty()),
            "diagnostics payload is empty for {args:?}"
        );
        assert!(
            payload.get("status").is_some()
                || payload.get("doctor").is_some()
                || payload.get("summary").is_some()
                || payload.get("paths").is_some()
                || payload.get("corruption_health").is_some()
                || payload.get("machine_report").is_some()
                || payload.get("text_report").is_some()
                || payload.get("install_state_assumptions").is_some(),
            "missing diagnostics contract keys for {args:?}"
        );
    }
}

#[test]
fn diagnostics_text_output_is_skimmable_and_non_empty() {
    let cases = [
        ["dev", "cli", "doctor", "--format", "text"].as_slice(),
        ["dev", "cli", "state-doctor", "--format", "text"].as_slice(),
        ["dev", "cli", "plugin-health", "--format", "text"].as_slice(),
    ];

    for args in cases {
        let output = run(args);
        assert_eq!(output.status.code(), Some(0), "non-zero exit for {args:?}");
        let text = String::from_utf8(output.stdout).expect("utf-8 text");
        assert!(!text.trim().is_empty(), "empty text output for {args:?}");
        assert!(text.contains('\n'), "expected multiline text for {args:?}");
    }
}

#[test]
fn diagnostics_exit_codes_follow_usage_runtime_success_contracts() {
    let ok = run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]);
    assert_eq!(ok.status.code(), Some(0));

    let usage = run(&["dev", "cli", "not-a-command", "--format", "json", "--no-pretty"]);
    assert_eq!(usage.status.code(), Some(2));
}
