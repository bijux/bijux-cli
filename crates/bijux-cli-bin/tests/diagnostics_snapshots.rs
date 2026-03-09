#![forbid(unsafe_code)]
//! Snapshot coverage for inspect and developer diagnostics command outputs.

use std::process::Command;

use bijux_cli_core as _;
use libc as _;
use serde_json as _;

fn run_stdout(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "command failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("utf-8 output")
}

#[test]
fn inspect_snapshots_match_json_yaml_and_text() {
    let cases: [(&[&str], &str); 3] = [
        (&["inspect", "--format", "json", "--pretty"], include_str!("snapshots/inspect_json.txt")),
        (&["inspect", "--format", "yaml", "--pretty"], include_str!("snapshots/inspect_yaml.txt")),
        (&["inspect", "--format", "text"], include_str!("snapshots/inspect_text.txt")),
    ];

    for (args, expected) in cases {
        let actual = run_stdout(args);
        assert_eq!(actual, expected, "snapshot mismatch for args: {args:?}");
    }
}

#[test]
fn dev_diagnostics_text_snapshots_match() {
    let cases: [(&[&str], &str); 8] = [
        (&["dev", "cli", "routes", "--format", "text"], include_str!("snapshots/dev_cli_routes_text.txt")),
        (&["dev", "cli", "registry", "--format", "text"], include_str!("snapshots/dev_cli_registry_text.txt")),
        (&["dev", "cli", "env", "--format", "text"], include_str!("snapshots/dev_cli_env_text.txt")),
        (&["dev", "cli", "doctor", "--format", "text"], include_str!("snapshots/dev_cli_doctor_text.txt")),
        (
            &["dev", "cli", "contracts", "--format", "text"],
            include_str!("snapshots/dev_cli_contracts_text.txt"),
        ),
        (
            &["dev", "cli", "runtime-identity", "--format", "text"],
            include_str!("snapshots/dev_cli_runtime_identity_text.txt"),
        ),
        (
            &["dev", "cli", "state-audit", "--format", "text"],
            include_str!("snapshots/dev_cli_state_audit_text.txt"),
        ),
        (
            &["dev", "cli", "state-doctor", "--format", "text"],
            include_str!("snapshots/dev_cli_state_doctor_text.txt"),
        ),
    ];

    for (args, expected) in cases {
        let actual = run_stdout(args);
        assert_eq!(actual, expected, "snapshot mismatch for args: {args:?}");
    }
}

#[test]
fn state_diagnostics_no_color_snapshots_match() {
    let cases: [(&[&str], &str); 2] = [
        (
            &["dev", "cli", "state-audit", "--format", "text"],
            include_str!("snapshots/dev_cli_state_audit_no_color.txt"),
        ),
        (
            &["dev", "cli", "state-doctor", "--format", "text"],
            include_str!("snapshots/dev_cli_state_doctor_no_color.txt"),
        ),
    ];

    for (args, expected) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
            .args(args)
            .env("NO_COLOR", "1")
            .output()
            .expect("binary should execute");
        assert!(output.status.success(), "command failed for args: {args:?}");
        let actual = String::from_utf8(output.stdout).expect("utf-8 output");
        assert_eq!(actual, expected, "snapshot mismatch for args: {args:?}");
    }
}
