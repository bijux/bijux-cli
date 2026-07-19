#![forbid(unsafe_code)]
//! REPL command parity coverage for stable behavior contracts.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli::api::repl::{execute_repl_input, execute_repl_line, startup_repl, ReplInput};
use bijux_cli::api::runtime::run_app;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_bin(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn parse_json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("json")
}

fn temp_dir(label: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("bijux-repl-command-parity-{label}-{nanos}"));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn repl_uses_same_kernel_entrypoint_and_route_resolution_as_non_interactive_cli() {
    let bin = run_bin(&["status", "--format", "json", "--no-pretty"]);
    let core = run_app(&[
        "bijux".to_string(),
        "status".to_string(),
        "--format".to_string(),
        "json".to_string(),
        "--no-pretty".to_string(),
    ])
    .expect("core run");

    let (mut repl, _) = startup_repl("", None);
    let repl_frame = execute_repl_line(&mut repl, "status --format json --no-pretty")
        .expect("repl execute")
        .expect("frame");

    assert_eq!(bin.status.code().unwrap_or(-1), core.exit_code);
    assert_eq!(repl.last_exit_code, bin.status.code().unwrap_or(-1));
    assert_eq!(parse_json(repl_frame.content.as_bytes()), parse_json(&bin.stdout));
}

#[test]
fn repl_machine_and_text_modes_use_same_underlying_payload_law() {
    let (mut repl, _) = startup_repl("", None);

    let repl_json = execute_repl_line(&mut repl, "status --format json --no-pretty")
        .expect("repl json")
        .expect("json frame");
    let bin_json = run_bin(&["status", "--format", "json", "--no-pretty"]);
    assert_eq!(parse_json(repl_json.content.as_bytes()), parse_json(&bin_json.stdout));

    let repl_text = execute_repl_line(&mut repl, "status --format text")
        .expect("repl text")
        .expect("text frame");
    let bin_text = run_bin(&["status", "--format", "text"]);
    assert_eq!(repl_text.content, String::from_utf8(bin_text.stdout).expect("utf-8"));
}

#[test]
fn repl_usage_validation_and_plugin_failures_map_to_same_failure_classes() {
    let (mut repl, _) = startup_repl("", None);

    let _ = execute_repl_line(&mut repl, "config get").expect("repl usage");
    let bin_usage = run_bin(&["config", "get"]);
    assert_eq!(repl.last_exit_code, bin_usage.status.code().unwrap_or(-1));

    let repl_validation =
        execute_repl_line(&mut repl, "status --format not-a-format").expect("repl validation");
    let bin_validation = run_bin(&["status", "--format", "not-a-format"]);
    assert_eq!(bin_validation.status.code(), Some(2));
    assert_eq!(repl.last_exit_code, bin_validation.status.code().unwrap_or(-1));
    let validation_frame = repl_validation.expect("validation frame");
    assert!(validation_frame.content.contains("invalid format"));

    let _ = execute_repl_line(&mut repl, "plugins uninstall").expect("repl plugin failure");
    let bin_plugin = run_bin(&["plugins", "uninstall"]);
    assert_eq!(repl.last_exit_code, bin_plugin.status.code().unwrap_or(-1));
}

#[test]
fn repl_state_corruption_handling_matches_non_interactive_cli_for_shared_commands() {
    let temp = temp_dir("corruption");
    let config = temp.join("broken.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN_LINE\n").expect("broken config");
    let config_arg = config.to_string_lossy().to_string();

    let bin = run_bin(&["config", "get", "alpha", "--config-path", &config_arg]);
    let (mut repl, _) = startup_repl("", None);
    let line = format!("config get alpha --config-path {config_arg}");
    let _ = execute_repl_line(&mut repl, &line).expect("repl corruption command");

    assert_eq!(repl.last_exit_code, bin.status.code().unwrap_or(-1));
}

#[test]
fn repl_quiet_trace_json_yaml_and_history_semantics_match_non_interactive_cli() {
    let (mut repl, _) = startup_repl("", None);

    let _ = execute_repl_input(&mut repl, ReplInput::Line(":set quiet on".to_string()))
        .expect("quiet on");
    let quiet =
        execute_repl_line(&mut repl, "status --format json --no-pretty").expect("quiet status");
    assert!(quiet.is_none());

    let _ = execute_repl_input(&mut repl, ReplInput::Line(":set quiet off".to_string()))
        .expect("quiet off");
    let _ = execute_repl_input(&mut repl, ReplInput::Line(":set trace on".to_string()))
        .expect("trace on");
    let traced = execute_repl_line(&mut repl, "status --format json --no-pretty")
        .expect("trace status")
        .expect("trace frame");
    let bin = run_bin(&["--log-level", "trace", "status", "--format", "json", "--no-pretty"]);
    assert_eq!(parse_json(traced.content.as_bytes()), parse_json(&bin.stdout));

    let repl_yaml = execute_repl_line(&mut repl, "status --format yaml --pretty")
        .expect("repl yaml")
        .expect("yaml frame");
    let bin_yaml = run_bin(&["status", "--format", "yaml", "--pretty"]);
    assert_eq!(repl_yaml.content, String::from_utf8(bin_yaml.stdout).expect("utf-8"));

    let before = execute_repl_line(&mut repl, "status --format json --no-pretty")
        .expect("before history")
        .expect("before frame");
    let _ = execute_repl_line(&mut repl, "version").expect("history write");
    let after = execute_repl_line(&mut repl, "status --format json --no-pretty")
        .expect("after history")
        .expect("after frame");
    assert_eq!(parse_json(before.content.as_bytes()), parse_json(after.content.as_bytes()));
}

#[test]
fn repl_help_for_builtin_and_plugin_commands_matches_non_interactive_help() {
    let (mut repl, _) = startup_repl("", None);

    let repl_help =
        execute_repl_line(&mut repl, "help status").expect("repl help").expect("help frame");
    let bin_help = run_bin(&["help", "status"]);
    let bin_help_text = String::from_utf8(bin_help.stdout).expect("utf-8");
    assert!(repl_help.content.contains("Usage:"));
    assert!(repl_help.content.contains("status"));
    assert!(bin_help_text.contains("Usage:"));
    assert!(bin_help_text.contains("status"));

    let repl_plugin_help = execute_repl_line(&mut repl, "help plugins")
        .expect("repl plugin help")
        .expect("help frame");
    let bin_plugin_help = run_bin(&["help", "plugins"]);
    let bin_plugin_help_text = String::from_utf8(bin_plugin_help.stdout).expect("utf-8");
    assert!(repl_plugin_help.content.contains("Usage:"));
    assert!(repl_plugin_help.content.contains("plugins"));
    assert!(bin_plugin_help_text.contains("Usage:"));
    assert!(bin_plugin_help_text.contains("plugins"));
}
