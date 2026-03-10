#![forbid(unsafe_code)]
//! Snapshot coverage for inspect and developer diagnostics command outputs.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_stdout(args: &[&str]) -> String {
    let home = snapshot_home();
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env("HOME", &home)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "command failed for args: {args:?}");
    normalize_output(String::from_utf8(output.stdout).expect("utf-8 output"), home.as_path())
}

fn snapshot_home() -> PathBuf {
    let home = std::env::temp_dir().join("bijux-cli-diagnostics-snapshots-home");
    let state_dir = home.join(".bijux");
    fs::create_dir_all(&state_dir).expect("create snapshot home state dir");
    fs::write(state_dir.join(".env"), "BIJUXCLI_ALPHA=1\n").expect("seed config");
    fs::write(state_dir.join(".history"), "[]\n").expect("seed history");
    fs::write(state_dir.join(".memory.json"), "{\"int_test_key\":42}\n").expect("seed memory");
    home
}

fn normalize_output(output: String, home: &std::path::Path) -> String {
    let home_text = home.display().to_string();
    output.replace(&home_text, "<HOME>")
}

#[test]
fn inspect_snapshots_match_json_yaml_and_text() {
    let cases: [(&[&str], &str); 3] = [
        (&["inspect", "--format", "json", "--pretty"], include_str!("../snapshots/inspect_json.txt")),
        (&["inspect", "--format", "yaml", "--pretty"], include_str!("../snapshots/inspect_yaml.txt")),
        (&["inspect", "--format", "text"], include_str!("../snapshots/inspect_text.txt")),
    ];

    for (args, expected) in cases {
        let actual = run_stdout(args);
        assert_eq!(actual, expected, "snapshot mismatch for args: {args:?}");
    }
}

#[test]
fn dev_diagnostics_text_snapshots_match() {
    let cases: [(&[&str], &str); 8] = [
        (
            &["dev", "cli", "routes", "--format", "text"],
            include_str!("../snapshots/dev_cli_routes_text.txt"),
        ),
        (
            &["dev", "cli", "registry", "--format", "text"],
            include_str!("../snapshots/dev_cli_registry_text.txt"),
        ),
        (
            &["dev", "cli", "env", "--format", "text"],
            include_str!("../snapshots/dev_cli_env_text.txt"),
        ),
        (
            &["dev", "cli", "doctor", "--format", "text"],
            include_str!("../snapshots/dev_cli_doctor_text.txt"),
        ),
        (
            &["dev", "cli", "contracts", "--format", "text"],
            include_str!("../snapshots/dev_cli_contracts_text.txt"),
        ),
        (
            &["dev", "cli", "runtime-identity", "--format", "text"],
            include_str!("../snapshots/dev_cli_runtime_identity_text.txt"),
        ),
        (
            &["dev", "cli", "state-audit", "--format", "text"],
            include_str!("../snapshots/dev_cli_state_audit_text.txt"),
        ),
        (
            &["dev", "cli", "state-doctor", "--format", "text"],
            include_str!("../snapshots/dev_cli_state_doctor_text.txt"),
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
            include_str!("../snapshots/dev_cli_state_audit_no_color.txt"),
        ),
        (
            &["dev", "cli", "state-doctor", "--format", "text"],
            include_str!("../snapshots/dev_cli_state_doctor_no_color.txt"),
        ),
    ];

    for (args, expected) in cases {
        let home = snapshot_home();
        let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
            .args(args)
            .env("HOME", &home)
            .env("NO_COLOR", "1")
            .output()
            .expect("binary should execute");
        assert!(output.status.success(), "command failed for args: {args:?}");
        let actual = normalize_output(
            String::from_utf8(output.stdout).expect("utf-8 output"),
            home.as_path(),
        );
        assert_eq!(actual, expected, "snapshot mismatch for args: {args:?}");
    }
}
