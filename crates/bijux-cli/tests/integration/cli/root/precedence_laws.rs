#![forbid(unsafe_code)]
//! Precedence laws for flags, environment, config files, defaults, and rendering policies.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!("bijux-precedence-laws-{name}-{}-{counter}", std::process::id(),));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn strip_ansi(input: &str) -> String {
    let mut out = String::new();
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\u{1b}' {
            if matches!(chars.peek(), Some('[')) {
                let _ = chars.next();
                for c in chars.by_ref() {
                    if c.is_ascii_alphabetic() {
                        break;
                    }
                }
                continue;
            }
        }
        out.push(ch);
    }
    out
}

#[test]
fn cli_flags_override_env_values() {
    let root = temp_dir("precedence-laws");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=config\n").expect("write config");

    let out = run_with_env(
        &["cli", "config", "get", "alpha", "--config-path", config.to_str().expect("utf-8")],
        &[("BIJUXCLI_ALPHA", "env"), ("BIJUXCLI_CONFIG", "/should/not/win")],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["value"], "env");
    assert_eq!(payload["source"], "env");
    assert!(payload["source_path"].is_null());
}

#[test]
fn env_values_override_config_file_values() {
    let root = temp_dir("precedence-laws");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=config\n").expect("write config");

    let out = run_with_env(
        &["cli", "config", "get", "alpha", "--config-path", config.to_str().expect("utf-8")],
        &[("BIJUXCLI_ALPHA", "env")],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["value"], "env");
    assert_eq!(payload["source"], "env");
}

#[test]
fn config_file_values_override_defaults() {
    let root = temp_dir("precedence-laws");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_ALPHA=config\n").expect("write config");

    let out =
        run(&["cli", "config", "get", "alpha", "--config-path", config.to_str().expect("utf-8")]);
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["value"], "config");
    assert_eq!(payload["source"], "file");
}

#[test]
fn defaults_apply_when_nothing_is_supplied() {
    let out = run(&["cli", "status"]);
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    let status = payload["status"].as_str().expect("status should be a string");
    assert!(matches!(status, "ok" | "warning" | "degraded"));
}

#[test]
fn explicit_config_path_overrides_default_config_path() {
    let root = temp_dir("precedence-laws");
    let path_a = root.join("a.env");
    let path_b = root.join("b.env");
    fs::write(&path_a, "BIJUXCLI_ALPHA=from_a\n").expect("write a");
    fs::write(&path_b, "BIJUXCLI_ALPHA=from_b\n").expect("write b");

    let out = run_with_env(
        &["cli", "config", "get", "alpha", "--config-path", path_b.to_str().expect("utf-8")],
        &[("BIJUXCLI_CONFIG", path_a.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["value"], "from_b");
}

#[test]
fn explicit_config_path_overrides_env_config_path() {
    let root = temp_dir("precedence-laws");
    let env_path = root.join("env.env");
    let arg_path = root.join("arg.env");
    fs::write(&env_path, "BIJUXCLI_ALPHA=from_env_path\n").expect("write env path");
    fs::write(&arg_path, "BIJUXCLI_ALPHA=from_arg_path\n").expect("write arg path");

    let out = run_with_env(
        &["cli", "config", "get", "alpha", "--config-path", arg_path.to_str().expect("utf-8")],
        &[("BIJUXCLI_CONFIG", env_path.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["value"], "from_arg_path");
}

#[test]
fn local_command_flags_do_not_override_global_policy_unexpectedly() {
    let global_first = run(&["--format", "json", "cli", "status", "--format", "text"]);
    assert_eq!(global_first.status.code(), Some(0));
    let text = String::from_utf8(global_first.stdout).expect("utf-8");
    assert!(text.contains("status:"));
    assert!(global_first.stderr.is_empty());
}

#[test]
fn quiet_mode_does_not_change_command_success_semantics() {
    let normal = run(&["cli", "status"]);
    let quiet = run(&["--quiet", "cli", "status"]);
    assert_eq!(normal.status.code(), Some(0));
    assert_eq!(quiet.status.code(), Some(0));
    assert!(quiet.stdout.is_empty());
}

#[test]
fn trace_mode_does_not_change_command_result_semantics() {
    let base = run(&["inspect", "--format", "json", "--no-pretty"]);
    let traced = run(&["--log-level", "trace", "inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(base.status.code(), Some(0));
    assert_eq!(traced.status.code(), Some(0));
    assert_eq!(base.stdout, traced.stdout);
}

#[test]
fn pretty_mode_changes_rendering_not_data() {
    let pretty = run(&["inspect", "--format", "json", "--pretty"]);
    let compact = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(pretty.status.code(), Some(0));
    assert_eq!(compact.status.code(), Some(0));
    let pretty_json: Value = serde_json::from_slice(&pretty.stdout).expect("pretty json");
    let compact_json: Value = serde_json::from_slice(&compact.stdout).expect("compact json");
    assert_eq!(pretty_json, compact_json);
}

#[test]
fn no_pretty_mode_changes_rendering_not_data() {
    let implicit = run(&["inspect", "--format", "json"]);
    let explicit_no_pretty = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(implicit.status.code(), Some(0));
    assert_eq!(explicit_no_pretty.status.code(), Some(0));
    let implicit_json: Value = serde_json::from_slice(&implicit.stdout).expect("implicit json");
    let no_pretty_json: Value =
        serde_json::from_slice(&explicit_no_pretty.stdout).expect("compact json");
    assert_eq!(implicit_json, no_pretty_json);
}

#[test]
fn color_affects_only_text_rendering() {
    let always = run(&["--color", "always", "cli", "status", "--format", "text"]);
    let never = run(&["--color", "never", "cli", "status", "--format", "text"]);
    assert_eq!(always.status.code(), Some(0));
    assert_eq!(never.status.code(), Some(0));
    let always_text = String::from_utf8(always.stdout).expect("utf-8");
    let never_text = String::from_utf8(never.stdout).expect("utf-8");
    assert_eq!(strip_ansi(&always_text), never_text);
}

#[test]
fn json_mode_ignores_color_settings_functionally() {
    let always = run(&["--color", "always", "--format", "json", "--no-pretty", "cli", "status"]);
    let never = run(&["--color", "never", "--format", "json", "--no-pretty", "cli", "status"]);
    assert_eq!(always.status.code(), Some(0));
    assert_eq!(never.status.code(), Some(0));
    assert_eq!(always.stdout, never.stdout);
}

#[test]
fn yaml_mode_ignores_color_settings_functionally() {
    let always = run(&["--color", "always", "--format", "yaml", "cli", "status"]);
    let never = run(&["--color", "never", "--format", "yaml", "cli", "status"]);
    assert_eq!(always.status.code(), Some(0));
    assert_eq!(never.status.code(), Some(0));
    assert_eq!(
        strip_ansi(&String::from_utf8(always.stdout).expect("utf-8")),
        String::from_utf8(never.stdout).expect("utf-8")
    );
}

#[test]
fn no_color_env_presence_disables_text_ansi_even_when_empty() {
    let out = run_with_env(
        &["--color", "always", "--format", "text", "cli", "status"],
        &[("NO_COLOR", "")],
    );
    assert_eq!(out.status.code(), Some(0));
    let rendered = String::from_utf8(out.stdout).expect("utf-8");
    assert!(!rendered.contains("\u{1b}["), "NO_COLOR presence should suppress ansi escapes");
}

#[test]
fn help_fast_path_honors_safe_output_policy() {
    let out = run(&["--quiet", "--help"]);
    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(stdout.contains("Usage: bijux"));
    assert!(out.stderr.is_empty());
}

#[test]
fn version_fast_path_is_stable_under_irrelevant_flags() {
    let base = run(&["version", "--format", "json", "--no-pretty"]);
    let variant = run(&["--color", "always", "version", "--format", "json", "--no-pretty"]);
    assert_eq!(base.status.code(), Some(0));
    assert_eq!(variant.status.code(), Some(0));
    assert_eq!(base.stdout, variant.stdout);
}

#[test]
fn deterministic_flag_reports_stable_unsupported_behavior() {
    let first = run(&["--deterministic", "cli", "status"]);
    let second = run(&["--deterministic", "cli", "status"]);
    assert_eq!(first.status.code(), Some(2));
    assert_eq!(second.status.code(), Some(2));
    assert_eq!(first.stderr, second.stderr);
}
