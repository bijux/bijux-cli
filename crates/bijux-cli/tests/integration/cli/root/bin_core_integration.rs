#![forbid(unsafe_code)]
//! Binary/core integration coverage for startup, output routing, and fast paths.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

#[cfg(unix)]
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_with(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], env: &[(&str, &str)]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (key, value) in env {
        cmd.env(key, value);
    }
    cmd.output().expect("binary should execute")
}

fn assert_runtime_status_grade(payload: &serde_json::Value) {
    let status = payload["status"].as_str().expect("status should be a string");
    assert!(matches!(status, "ok" | "warning" | "degraded"));
}

#[test]
fn startup_commands_execute_through_binary() {
    for (args, expect_usage_text) in [
        (vec!["version"], false),
        (vec!["--version"], false),
        (vec!["doctor"], false),
        (vec!["inspect"], false),
        (vec!["repl", "--help"], true),
        (vec!["cli", "status"], false),
    ] {
        let out = run_with(&args);
        assert!(out.status.success(), "expected success for {args:?}");
        assert!(out.stderr.is_empty(), "stderr should be empty for startup command {args:?}");
        assert!(!out.stdout.is_empty(), "stdout should not be empty for startup command {args:?}");
        let text = String::from_utf8(out.stdout).expect("stdout should be utf-8");
        if expect_usage_text {
            assert!(
                text.contains("Usage:"),
                "help-like startup command should include usage for {args:?}"
            );
        } else {
            let payload: serde_json::Value =
                serde_json::from_str(&text).expect("non-help startup command should emit json");
            assert!(payload.is_object(), "startup json payload should be object for {args:?}");
        }
    }
}

#[test]
fn success_machine_output_keeps_stderr_empty() {
    let out = run_with(&["--format", "json", "--no-pretty", "cli", "status"]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    assert!(!out.stdout.is_empty());
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("machine output should be valid json");
    assert_runtime_status_grade(&payload);
}

#[test]
fn failure_output_routes_to_stderr_and_not_stdout() {
    let out = run_with(&["cli", "unknown-command"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "stdout should be empty for usage failures");
    assert!(!out.stderr.is_empty(), "stderr should contain usage details");
    let stderr = String::from_utf8(out.stderr).expect("stderr should be utf-8");
    assert!(stderr.contains("Usage: bijux"), "stderr should include usage summary");
    assert!(stderr.contains("Commands:"), "stderr should include command table");
}

#[test]
fn bin_and_core_outputs_match_for_same_argv() {
    let argv = vec!["bijux".to_string(), "cli".to_string(), "status".to_string()];
    let core = bijux_cli::api::runtime::run_app(&argv).expect("core run_app should succeed");

    let out = run_with(&["cli", "status"]);
    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn trace_mode_executes_through_binary() {
    let out = run_with(&["--log-level", "trace", "cli", "status"]);
    assert!(out.status.success());
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("trace mode stdout should be valid json");
    assert_runtime_status_grade(&payload);
}

#[test]
fn color_mode_executes_through_binary() {
    let out = run_with(&["--color", "always", "cli", "status"]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("stdout should be valid json");
    assert_runtime_status_grade(&payload);
}

#[test]
fn no_color_env_executes_through_binary() {
    let out = run_with_env(&["--color", "always", "cli", "status"], &[("NO_COLOR", "1")]);
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let text = String::from_utf8(out.stdout.clone()).expect("stdout should be utf-8");
    assert!(!text.contains("\u{1b}["), "NO_COLOR should suppress ansi escapes");
    let payload: serde_json::Value =
        serde_json::from_str(&text).expect("stdout should be valid json");
    assert_runtime_status_grade(&payload);
}

#[test]
fn state_path_commands_succeed_when_home_is_unset() {
    let out = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(["cli", "paths", "--format", "json", "--no-pretty"])
        .env_remove("HOME")
        .output()
        .expect("binary should execute");
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).expect("json payload");
    assert!(payload["config"].is_string());
    assert!(payload["history"].is_string());
}

#[test]
fn compact_json_executes_through_binary() {
    let out = run_with(&["--format", "json", "--no-pretty", "cli", "status"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).expect("stdout should be utf-8");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("compact json should parse");
    assert_runtime_status_grade(&parsed);
    assert!(
        text.lines().count() <= 2,
        "compact output should be single-line json with trailing newline"
    );
}

#[test]
fn pretty_json_executes_through_binary() {
    let out = run_with(&["--format", "json", "--pretty", "cli", "status"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).expect("stdout should be utf-8");
    assert!(text.lines().count() > 2, "pretty output should be multiline json");
}

#[test]
fn yaml_executes_through_binary() {
    let out = run_with(&["--format", "yaml", "cli", "status"]);
    assert!(out.status.success());
    let text = String::from_utf8(out.stdout).expect("stdout should be utf-8");
    assert!(text.contains("status:"));
}

#[test]
fn quiet_mode_suppresses_output_for_success() {
    let out = run_with(&["--quiet", "cli", "status"]);
    assert!(out.status.success());
    assert!(out.stdout.is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn help_fast_path_timing_regression_guard() {
    let start = Instant::now();
    let out = run_with(&["--help"]);
    let elapsed = start.elapsed();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("Usage:"),
        "help output should include usage section"
    );
    assert!(elapsed < Duration::from_secs(2), "help fast-path regressed: {elapsed:?}");
}

#[test]
fn version_fast_path_timing_regression_guard() {
    let start = Instant::now();
    let out = run_with(&["version"]);
    let elapsed = start.elapsed();
    assert!(out.status.success());
    assert!(out.stderr.is_empty());
    let payload: serde_json::Value =
        serde_json::from_slice(&out.stdout).expect("version output should be valid json");
    assert!(payload["version"].is_string());
    assert!(elapsed < Duration::from_secs(2), "version fast-path regressed: {elapsed:?}");
}

#[cfg(unix)]
#[test]
fn invalid_utf8_argv_returns_usage_error() {
    use std::os::unix::ffi::OsStringExt;

    let invalid = std::ffi::OsString::from_vec(vec![0x66, 0x80, 0x67]);
    let out = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .arg(invalid)
        .output()
        .expect("binary should execute");

    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("invalid UTF-8 argument in argv"));
}

#[cfg(unix)]
#[test]
fn ctrl_c_exits_safely_on_interactive_repl_process() {
    use std::os::unix::process::ExitStatusExt;

    let mut child = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .arg("repl")
        .spawn()
        .expect("repl process should start");

    std::thread::sleep(Duration::from_millis(150));
    let pid = child.id() as i32;
    let status = Command::new("kill")
        .args(["-INT", &pid.to_string()])
        .status()
        .expect("kill command should execute");
    assert!(status.success(), "kill command should succeed");

    let status = child.wait().expect("child should exit");
    assert!(
        status.code() == Some(0)
            || status.code() == Some(130)
            || status.signal() == Some(libc::SIGINT),
        "unexpected exit status after SIGINT: {status:?}"
    );
}

#[test]
fn repl_process_executes_piped_commands_and_exits_cleanly() {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .arg("repl")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("repl process should start");

    {
        let stdin = child.stdin.as_mut().expect("stdin should be piped");
        stdin
            .write_all(b"status --format json --no-pretty\n:exit\n")
            .expect("write repl transcript");
    }

    let output = child.wait_with_output().expect("repl output");
    assert_eq!(output.status.code(), Some(0));
    assert!(output.stderr.is_empty(), "stderr should stay empty on clean repl transcript");

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let status_line = stdout
        .lines()
        .find(|line| line.trim_start().starts_with('{'))
        .expect("repl should emit a json payload line");
    let payload: serde_json::Value =
        serde_json::from_str(status_line).expect("status payload json");
    assert_eq!(payload["status"], serde_json::json!("ok"));
}
