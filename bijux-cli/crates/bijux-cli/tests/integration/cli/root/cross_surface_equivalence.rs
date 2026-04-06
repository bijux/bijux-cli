#![forbid(unsafe_code)]
//! Cross-surface equivalence coverage for binary, core, and REPL.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli::api::repl::{
    execute_repl_input, execute_repl_line, startup_repl, ReplInput, ReplStream,
};
use bijux_cli::api::runtime::{run_app, AppRunResult};
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_binary(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_core_cmd(args: &[&str]) -> AppRunResult {
    let mut argv = vec!["bijux".to_string()];
    argv.extend(args.iter().map(|arg| (*arg).to_string()));
    run_app(&argv).expect("core execution should succeed")
}

fn parse_json(text: &str) -> Value {
    serde_json::from_str(text).expect("payload should be valid json")
}

fn temp_dir(name: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("bijux-cross-surface-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).expect("create temp directory");
    path
}

#[test]
fn binary_vs_direct_core_version_result_matches() {
    let bin = run_binary(&["version"]);
    let core = run_core_cmd(&["version"]);
    assert_eq!(bin.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&bin.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&bin.stderr), core.stderr);
}

#[test]
fn binary_vs_direct_core_status_result_matches() {
    let bin = run_binary(&["status"]);
    let core = run_core_cmd(&["status"]);
    assert_eq!(bin.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&bin.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&bin.stderr), core.stderr);
}

#[test]
fn binary_vs_direct_core_doctor_result_matches() {
    let bin = run_binary(&["doctor"]);
    let core = run_core_cmd(&["doctor"]);
    assert_eq!(bin.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&bin.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&bin.stderr), core.stderr);
}

#[test]
fn binary_vs_direct_core_plugins_list_result_matches() {
    let bin = run_binary(&["plugins", "list"]);
    let core = run_core_cmd(&["plugins", "list"]);
    assert_eq!(bin.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&bin.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&bin.stderr), core.stderr);
}

#[test]
fn binary_vs_direct_core_config_get_result_matches() {
    let root = temp_dir("config-get-direct-core");
    let config = root.join("config.env");
    fs::write(&config, "sample_key=from-file\n").expect("write config");

    let config_text = config.to_string_lossy().to_string();
    let bin = run_binary(&["--config-path", &config_text, "config", "get", "sample_key"]);
    let core = run_core_cmd(&["--config-path", &config_text, "config", "get", "sample_key"]);

    assert_eq!(bin.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&bin.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&bin.stderr), core.stderr);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn binary_vs_repl_status_result_matches_where_sensible() {
    let bin = run_binary(&["status"]);
    assert!(bin.status.success());

    let (mut repl, _) = startup_repl("default", None);
    let frame = execute_repl_line(&mut repl, "status")
        .expect("repl status should execute")
        .expect("repl should emit output frame");

    assert_eq!(frame.stream, ReplStream::Stdout);
    let repl_payload = parse_json(&frame.content);
    let bin_payload = parse_json(&String::from_utf8_lossy(&bin.stdout));
    assert_eq!(repl_payload, bin_payload);
}

#[test]
fn binary_vs_repl_unknown_command_exit_semantics_match_where_sensible() {
    let bin = run_binary(&["unknown-command"]);
    let bin_code = bin.status.code().unwrap_or(-1);

    let (mut repl, _) = startup_repl("default", None);
    let event = execute_repl_input(&mut repl, ReplInput::Line("unknown-command".to_string()))
        .expect("repl should continue with stderr frame");

    assert_eq!(repl.last_exit_code, bin_code);
    match event {
        bijux_cli::api::repl::ReplEvent::Continue(Some(frame)) => {
            assert_eq!(frame.stream, ReplStream::Stderr);
            assert!(!frame.content.is_empty());
        }
        other => panic!("expected stderr continue frame, got {other:?}"),
    }
}
