#![forbid(unsafe_code)]
//! Root command matrix coverage and explicit root-surface law tests.

use std::process::{Command, Output};

use bijux_cli::api::runtime::run_app;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(out.status.success(), "expected success for {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout should be valid json")
}

#[test]
fn parity_version_against_current_expected_behavior() {
    let out = run(&["version"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("version").is_some());

    let core = run_app(&["bijux".to_string(), "version".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_version_flag_matches_version_command() {
    let flag = run(&["--version"]);
    assert!(flag.status.success());
    let flagged_payload: Value = serde_json::from_slice(&flag.stdout).expect("json");
    assert!(flagged_payload.get("version").is_some());

    let command = run(&["version"]);
    assert_eq!(flag.status.code(), command.status.code());
    assert_eq!(flag.stdout, command.stdout);
    assert_eq!(flag.stderr, command.stderr);
}

#[test]
fn parity_status_against_current_expected_behavior() {
    let out = run(&["status"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert_eq!(payload["status"], "ok");

    let core = run_app(&["bijux".to_string(), "status".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_doctor_against_current_expected_behavior() {
    let out = run(&["doctor"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("status").is_some());
    assert!(payload.get("install").is_some());

    let core = run_app(&["bijux".to_string(), "doctor".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_inspect_against_current_expected_behavior() {
    let out = run(&["inspect"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("route_sources").is_some());

    let core = run_app(&["bijux".to_string(), "inspect".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_docs_against_current_expected_behavior() {
    let out = run(&["docs"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("topics").is_some());

    let core = run_app(&["bijux".to_string(), "docs".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_audit_against_current_expected_behavior() {
    let out = run(&["audit"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("checks").is_some());

    let core = run_app(&["bijux".to_string(), "audit".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_sleep_against_current_expected_behavior() {
    let out = run(&["sleep", "0"]);
    assert!(out.status.success());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    assert!(payload.get("slept_seconds").is_some());

    let core = run_app(&["bijux".to_string(), "sleep".to_string(), "0".to_string()]).expect("core");
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn help_snapshot_exists_for_every_root_command() {
    let roots = [
        "status",
        "version",
        "doctor",
        "inspect",
        "docs",
        "audit",
        "sleep",
        "config",
        "plugins",
        "history",
        "memory",
        "repl",
        "completion",
    ];
    for cmd in roots {
        let out = run(&[cmd, "--help"]);
        assert!(out.status.success(), "help must succeed for root command {cmd}");
        let stdout = String::from_utf8(out.stdout).expect("utf-8");
        assert!(stdout.contains("Usage:"), "help for {cmd} should include Usage");
    }
}

#[test]
fn exit_code_and_stream_discipline_for_root_commands() {
    let success_cases: &[&[&str]] = &[
        &["version"],
        &["status"],
        &["doctor"],
        &["inspect"],
        &["docs"],
        &["audit"],
        &["sleep", "0"],
    ];
    for args in success_cases {
        let out = run(args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(!out.stdout.is_empty(), "stdout should contain payload for {args:?}");
        assert!(out.stderr.is_empty(), "stderr should be empty for {args:?}");
    }

    let fail = run(&["config", "get"]);
    assert_ne!(fail.status.code(), Some(0));
    assert!(fail.stdout.is_empty());
    assert!(!fail.stderr.is_empty());
}

#[test]
fn machine_readable_root_commands_support_json_and_yaml() {
    let machine_cases: &[&[&str]] = &[
        &["status"],
        &["doctor"],
        &["inspect"],
        &["docs"],
        &["audit"],
        &["sleep", "0"],
        &["history"],
        &["memory"],
        &["plugins", "list"],
    ];

    for base in machine_cases {
        let mut json_args = base.to_vec();
        json_args.extend(["--format", "json", "--no-pretty"]);
        let json_out = run(&json_args);
        assert!(json_out.status.success(), "json mode failed for {base:?}");
        let _: Value = serde_json::from_slice(&json_out.stdout).expect("json parse");

        let mut yaml_args = base.to_vec();
        yaml_args.extend(["--format", "yaml", "--pretty"]);
        let yaml_out = run(&yaml_args);
        assert!(yaml_out.status.success(), "yaml mode failed for {base:?}");
        let yaml = String::from_utf8(yaml_out.stdout).expect("utf-8");
        assert!(!yaml.trim().is_empty(), "yaml output should not be empty for {base:?}");
    }
}

#[test]
fn quiet_mode_is_supported_for_relevant_root_commands() {
    let relevant: &[&[&str]] =
        &[&["status"], &["doctor"], &["inspect"], &["docs"], &["audit"], &["sleep", "0"]];
    for args in relevant {
        let mut quiet_args = args.to_vec();
        quiet_args.insert(0, "--quiet");
        let out = run(&quiet_args);
        assert!(out.status.success(), "quiet mode failed for {args:?}");
        assert!(out.stdout.is_empty(), "quiet should suppress stdout for {args:?}");
        assert!(out.stderr.is_empty(), "quiet should suppress stderr for {args:?}");
    }
}

#[test]
fn no_color_is_supported_for_text_root_commands() {
    for args in [vec!["help"], vec!["help", "status"], vec!["help", "plugins"]] {
        let mut argv = vec!["--color", "never"];
        argv.extend(args);
        let out = run(&argv);
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).expect("utf-8");
        assert!(!text.contains("\u{1b}["));
    }
}

#[test]
fn malformed_input_is_rejected_for_argument_taking_root_commands() {
    let malformed: &[&[&str]] = &[
        &["config", "get"],
        &["plugins", "uninstall"],
        &["sleep", "0", "--unknown-flag"],
        &["history", "--bad-flag"],
        &["memory", "set"],
    ];
    for args in malformed {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "malformed input should fail for {args:?}");
        assert!(out.stdout.is_empty(), "malformed input should not print stdout for {args:?}");
        assert!(!out.stderr.is_empty(), "malformed input should print stderr for {args:?}");
    }
}

#[test]
fn repeated_run_determinism_for_machine_readable_root_commands() {
    let deterministic: &[&[&str]] = &[
        &["status", "--format", "json", "--no-pretty"],
        &["doctor", "--format", "json", "--no-pretty"],
        &["inspect", "--format", "json", "--no-pretty"],
        &["docs", "--format", "json", "--no-pretty"],
        &["audit", "--format", "json", "--no-pretty"],
        &["sleep", "0", "--format", "json", "--no-pretty"],
    ];

    for args in deterministic {
        let first = run(args);
        let second = run(args);
        assert!(first.status.success());
        assert!(second.status.success());
        assert_eq!(first.stdout, second.stdout, "output drift for {args:?}");
        assert_eq!(first.stderr, second.stderr, "stderr drift for {args:?}");
    }
}

#[test]
fn root_command_matrix_artifact_smoke_uses_supported_commands() {
    // Smoke check for the matrix command list used by report generation.
    let matrix = [
        ["version"].as_slice(),
        ["status"].as_slice(),
        ["doctor"].as_slice(),
        ["inspect"].as_slice(),
        ["docs"].as_slice(),
        ["audit"].as_slice(),
        ["sleep", "0"].as_slice(),
    ];
    for args in matrix {
        let payload = run_ok_json(args);
        assert!(payload.is_object(), "matrix command should return object payload: {args:?}");
    }
}
