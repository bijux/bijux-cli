#![forbid(unsafe_code)]
//! Flag normalization laws for global and local argument handling.

use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

#[test]
fn global_flags_before_namespace_are_accepted() {
    let out = run(&["--format", "json", "--no-pretty", "cli", "status"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
}

#[test]
fn global_flags_after_namespace_are_accepted_when_supported() {
    let out = run(&["cli", "status", "--format", "json", "--no-pretty"]);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
}

#[test]
fn global_flags_before_and_after_namespace_normalize_to_same_intent() {
    let before = run(&["--format", "json", "--no-pretty", "cli", "status"]);
    let after = run(&["cli", "status", "--format", "json", "--no-pretty"]);
    assert_eq!(before.status.code(), Some(0));
    assert_eq!(after.status.code(), Some(0));
    assert_eq!(before.stdout, after.stdout);
}

#[test]
fn repeated_format_flags_are_rejected_deterministically() {
    let out = run(&["--format", "text", "--format", "json", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_pretty_flags_are_rejected_deterministically() {
    let out = run(&["--pretty", "--pretty", "--format", "json", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_no_pretty_flags_are_rejected_deterministically() {
    let out = run(&["--no-pretty", "--no-pretty", "--format", "json", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_quiet_flags_are_rejected_deterministically() {
    let out = run(&["--quiet", "--quiet", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_trace_flags_are_rejected_deterministically() {
    let out = run(&["--log-level", "trace", "--log-level", "trace", "inspect"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_color_flags_are_rejected_deterministically() {
    let out = run(&["--color", "always", "--color", "never", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn repeated_config_flags_are_rejected_deterministically() {
    let out = run(&["--config-path", "a", "--config-path", "b", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn conflicting_pretty_and_no_pretty_have_stable_resolution() {
    let out = run(&["--pretty", "--no-pretty", "--format", "json", "cli", "status"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).expect("stdout utf-8");
    assert!(stdout.contains("\"runtime\""));
    assert!(stdout.contains("\"status\""));
}

#[test]
fn conflicting_color_always_and_never_are_rejected() {
    let out = run(&["--color", "always", "--color", "never", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn invalid_format_value_is_rejected() {
    let out = run(&["--format", "nope", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid format: nope"));
}

#[test]
fn invalid_color_value_is_rejected() {
    let out = run(&["--color", "nope", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("invalid color mode: nope"));
}

#[test]
fn missing_value_after_config_flag_is_rejected() {
    let out = run(&["--config-path"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("a value is required for '--config-path <PATH>'"));
}

#[test]
fn missing_value_after_format_flag_is_rejected() {
    let out = run(&["--format"]);
    assert_eq!(out.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("a value is required for '--format <FORMAT>'"));
}

#[test]
fn unknown_global_flag_at_root_is_rejected() {
    let out = run(&["--wat", "cli", "status"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn unknown_local_flag_in_grouped_command_is_rejected() {
    let out = run(&["cli", "status", "--wat"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn mixed_global_local_flag_ordering_abuse_is_rejected() {
    let out = run(&["cli", "status", "--format", "json", "--wat", "--no-pretty"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}
