#![forbid(unsafe_code)]
//! Integration coverage for maintainer control-plane commands.

use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(
        out.status.success(),
        "command failed: {:?}\nstdout={}\nstderr={}",
        args,
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).expect("valid json")
}

#[test]
fn required_maintainer_commands_have_stable_json_shapes() {
    let commands = [
        (["dev", "cli", "status"], "status_report"),
        (["dev", "cli", "parity"], "parity_dashboard"),
        (["dev", "cli", "route-audit"], "summary"),
        (["dev", "cli", "state-audit"], "paths"),
        (["dev", "cli", "script-audit"], "scripts"),
        (["dev", "cli", "crate-health"], "crate_metrics"),
        (["dev", "cli", "package-health"], "install_state_assumptions"),
        (["dev", "cli", "docs-audit"], "docs_count"),
    ];

    for (command, required_key) in commands {
        let first = run_ok_json(&command);
        let second = run_ok_json(&command);
        assert!(
            first.get(required_key).is_some(),
            "missing required key {required_key} for {:?}",
            command
        );

        let first_keys = first
            .as_object()
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        let second_keys = second
            .as_object()
            .map(|obj| obj.keys().cloned().collect::<Vec<_>>())
            .unwrap_or_default();
        assert_eq!(first_keys, second_keys, "top-level key drift for {:?}", command);
    }
}

#[test]
fn required_maintainer_commands_emit_skimmable_text() {
    let commands = [
        "dev cli status",
        "dev cli parity",
        "dev cli route-audit",
        "dev cli state-audit",
        "dev cli script-audit",
        "dev cli crate-health",
        "dev cli package-health",
        "dev cli docs-audit",
    ];
    for command in commands {
        let mut args: Vec<&str> = command.split_whitespace().collect();
        args.push("--format");
        args.push("text");
        let out = run(&args);
        assert!(
            out.status.success(),
            "text command failed for {command}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).expect("utf8");
        assert!(!text.trim().is_empty(), "text output empty for {command}");
        assert!(text.len() <= 2_000_000, "text output unexpectedly huge for {command}");
        assert!(
            text.contains('{') || text.contains('['),
            "text output should be structured for {command}"
        );
    }
}
