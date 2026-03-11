#![forbid(unsafe_code)]
//! Stdout/stderr discipline matrix and stream routing invariants.
//! test_type: stream-discipline

use std::process::{Command, Output};

use bijux_cli as _;
use bijux_cli::api::repl::{execute_repl_line, startup_repl};
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .output()
        .expect("binary should execute")
}

#[test]
fn successful_machine_readable_commands_keep_stderr_empty() {
    let cases = [
        vec!["status", "--format", "json", "--no-pretty"],
        vec!["inspect", "--format", "json", "--no-pretty"],
        vec!["doctor", "--format", "json", "--no-pretty"],
        vec!["dev", "cli", "routes", "--format", "json", "--no-pretty"],
    ];
    for args in cases {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(
            !out.stdout.is_empty(),
            "stdout should contain payload for {args:?}"
        );
        assert!(
            out.stderr.is_empty(),
            "stderr should remain empty for {args:?}"
        );
    }
}

#[test]
fn text_success_commands_do_not_leak_diagnostics_to_stderr_in_normal_mode() {
    let cases = [
        vec!["status", "--format", "text"],
        vec!["doctor", "--format", "text"],
        vec!["cli", "paths", "--format", "text"],
        vec!["plugins", "list", "--format", "text"],
    ];
    for args in cases {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(
            !out.stdout.is_empty(),
            "stdout should contain text output for {args:?}"
        );
        assert!(
            out.stderr.is_empty(),
            "stderr should stay empty for {args:?}"
        );
    }
}

#[test]
fn usage_validation_plugin_and_internal_failures_route_to_stderr_only() {
    let usage = run(&["config", "get"]);
    assert_eq!(usage.status.code(), Some(2));
    assert!(usage.stdout.is_empty());
    assert!(!usage.stderr.is_empty());

    let validation = run(&["--format", "not-a-format", "status"]);
    assert_eq!(validation.status.code(), Some(1));
    assert!(validation.stdout.is_empty());
    assert!(!validation.stderr.is_empty());

    let plugin = run(&["plugins", "uninstall"]);
    assert_eq!(plugin.status.code(), Some(1));
    assert!(plugin.stdout.is_empty());
    assert!(!plugin.stderr.is_empty());

    let internal_like = run(&["plugins", "enable"]);
    assert_eq!(internal_like.status.code(), Some(1));
    assert!(internal_like.stdout.is_empty());
    assert!(!internal_like.stderr.is_empty());
}

#[test]
fn quiet_mode_suppresses_success_stdout_and_nonessential_stderr_noise() {
    let out = run(&["--quiet", "status", "--format", "json", "--no-pretty"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(
        out.stdout.is_empty(),
        "quiet mode should suppress success stdout"
    );
    assert!(
        out.stderr.is_empty(),
        "quiet mode should suppress nonessential stderr noise"
    );
}

#[test]
fn trace_mode_preserves_stream_contract_without_corrupting_output_envelope() {
    let plain = run(&["status", "--format", "json", "--no-pretty"]);
    let traced = run(&[
        "--log-level",
        "trace",
        "status",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(plain.status.code(), Some(0));
    assert_eq!(traced.status.code(), Some(0));
    assert!(plain.stderr.is_empty());
    assert!(traced.stderr.is_empty());

    let plain_payload: Value = serde_json::from_slice(&plain.stdout).expect("plain json");
    let traced_payload: Value = serde_json::from_slice(&traced.stdout).expect("trace json");
    assert_eq!(plain_payload["status"], traced_payload["status"]);
    assert!(
        traced_payload.is_object(),
        "trace output envelope should remain valid json object"
    );
}

#[test]
fn pretty_compact_json_and_yaml_all_respect_stream_discipline() {
    let pretty = run(&["status", "--format", "json", "--pretty"]);
    assert_eq!(pretty.status.code(), Some(0));
    assert!(!pretty.stdout.is_empty());
    assert!(pretty.stderr.is_empty());

    let compact = run(&["status", "--format", "json", "--no-pretty"]);
    assert_eq!(compact.status.code(), Some(0));
    assert!(!compact.stdout.is_empty());
    assert!(compact.stderr.is_empty());

    let yaml = run(&["status", "--format", "yaml", "--pretty"]);
    assert_eq!(yaml.status.code(), Some(0));
    assert!(!yaml.stdout.is_empty());
    assert!(yaml.stderr.is_empty());
}

#[test]
fn help_and_version_fast_paths_do_not_leak_unrelated_diagnostics_to_stderr() {
    let help = run(&["help", "status"]);
    assert_eq!(help.status.code(), Some(0));
    assert!(!help.stdout.is_empty());
    assert!(help.stderr.is_empty());

    let version = run(&["version"]);
    assert_eq!(version.status.code(), Some(0));
    assert!(!version.stdout.is_empty());
    assert!(version.stderr.is_empty());
}

#[test]
fn plugin_and_state_doctor_commands_obey_builtin_stream_law() {
    let plugin_ok = run(&["plugins", "list", "--format", "json", "--no-pretty"]);
    assert_eq!(plugin_ok.status.code(), Some(0));
    assert!(!plugin_ok.stdout.is_empty());
    assert!(plugin_ok.stderr.is_empty());

    let plugin_fail = run(&["plugins", "uninstall"]);
    assert_eq!(plugin_fail.status.code(), Some(1));
    assert!(plugin_fail.stdout.is_empty());
    assert!(!plugin_fail.stderr.is_empty());

    let state_doctor_ok = run(&[
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(state_doctor_ok.status.code(), Some(0));
    assert!(!state_doctor_ok.stdout.is_empty());
    assert!(state_doctor_ok.stderr.is_empty());
}

#[test]
fn repl_exit_class_matches_binary_for_stream_routed_failures() {
    let mut session = startup_repl("", None).0;
    let _ = execute_repl_line(&mut session, "config get").expect("repl should return control");
    assert_eq!(
        session.last_exit_code,
        run(&["config", "get"]).status.code().unwrap_or(-1)
    );

    let _ = execute_repl_line(&mut session, "status --format json --no-pretty")
        .expect("repl status should return control");
    assert_eq!(
        session.last_exit_code,
        run(&["status", "--format", "json", "--no-pretty"])
            .status
            .code()
            .unwrap_or(-1)
    );
}
