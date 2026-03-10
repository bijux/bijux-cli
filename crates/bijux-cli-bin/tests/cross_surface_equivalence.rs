#![forbid(unsafe_code)]
//! Cross-surface equivalence coverage for binary, core, python bridge, and REPL.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli_core::app::{run_app, AppRunResult};
use bijux_cli_python as _;
use bijux_cli_python::{command_tree_introspection_api, execution_outcome_api};
use bijux_cli_repl::{execute_repl_input, execute_repl_line, startup_repl, ReplInput, ReplStream};
use bijux_cli_routing as _;
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run_binary(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_core_cmd(args: &[&str]) -> AppRunResult {
    let mut argv = vec!["bijux".to_string()];
    argv.extend(args.iter().map(|arg| arg.to_string()));
    run_app(&argv).expect("core execution should succeed")
}

fn bridge_outcome(args: &[&str]) -> Value {
    let mut argv = vec!["bijux".to_string()];
    argv.extend(args.iter().map(|arg| arg.to_string()));
    serde_json::from_str(&execution_outcome_api(&argv).expect("bridge outcome should serialize"))
        .expect("bridge outcome should be json")
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

fn root_namespaces_from_inspect(payload: &Value) -> Vec<String> {
    let mut namespaces: Vec<String> = payload
        .get("builtins")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| entry.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    namespaces.sort();
    namespaces.dedup();
    namespaces
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
fn binary_vs_python_bridge_version_result_matches() {
    let bin = run_binary(&["version"]);
    let bridge = bridge_outcome(&["version"]);
    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));
}

#[test]
fn binary_vs_python_bridge_status_result_matches() {
    let bin = run_binary(&["status"]);
    let bridge = bridge_outcome(&["status"]);
    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));
}

#[test]
fn binary_vs_python_bridge_doctor_result_matches() {
    let bin = run_binary(&["doctor"]);
    let bridge = bridge_outcome(&["doctor"]);
    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));
}

#[test]
fn binary_vs_python_bridge_plugins_list_result_matches() {
    let bin = run_binary(&["plugins", "list"]);
    let bridge = bridge_outcome(&["plugins", "list"]);
    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));
}

#[test]
fn binary_vs_python_bridge_config_get_result_matches() {
    let root = temp_dir("config-get-bridge");
    let config = root.join("config.env");
    fs::write(&config, "sample_key=from-file\n").expect("write config");

    let config_text = config.to_string_lossy().to_string();
    let bin = run_binary(&["--config-path", &config_text, "config", "get", "sample_key"]);
    let bridge = bridge_outcome(&["--config-path", &config_text, "config", "get", "sample_key"]);

    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));

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
        bijux_cli_repl::ReplEvent::Continue(Some(frame)) => {
            assert_eq!(frame.stream, ReplStream::Stderr);
            assert!(!frame.content.is_empty());
        }
        other => panic!("expected stderr continue frame, got {other:?}"),
    }
}

#[test]
fn binary_vs_python_bridge_namespace_rejection_behavior_matches() {
    let root = temp_dir("bridge-namespace-rejection");
    let plugin_path = root.join("plugin-out");
    let path_text = plugin_path.to_string_lossy().to_string();

    let bin = run_binary(&["cli", "plugins", "scaffold", "python", "cli", "--path", &path_text]);
    let bridge =
        bridge_outcome(&["cli", "plugins", "scaffold", "python", "cli", "--path", &path_text]);

    assert_eq!(bridge["exit_code"].as_i64(), Some(i64::from(bin.status.code().unwrap_or(-1))));
    assert_eq!(bridge["stdout"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stdout));
    assert_eq!(bridge["stderr"].as_str().unwrap_or_default(), String::from_utf8_lossy(&bin.stderr));
    assert!(String::from_utf8_lossy(&bin.stderr).contains("plugin namespace is reserved: cli"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn binary_vs_python_bridge_error_envelope_shape_matches() {
    let bin = run_binary(&["unknown-command"]);
    let bridge = bridge_outcome(&["unknown-command"]);

    let bin_stderr = String::from_utf8_lossy(&bin.stderr);
    let bin_error = parse_json(&bin_stderr);
    let bridge_stderr = bridge["stderr"].as_str().unwrap_or_default();
    let bridge_error = parse_json(bridge_stderr);

    let mut bin_keys: Vec<String> = bin_error
        .as_object()
        .expect("binary error envelope should be object")
        .keys()
        .cloned()
        .collect();
    bin_keys.sort();
    let mut bridge_keys: Vec<String> = bridge_error
        .as_object()
        .expect("bridge error envelope should be object")
        .keys()
        .cloned()
        .collect();
    bridge_keys.sort();

    assert_eq!(bin_keys, bridge_keys);
}

#[test]
fn binary_vs_python_bridge_stdout_stderr_discipline_matches() {
    let success_bin = run_binary(&["status"]);
    let success_bridge = bridge_outcome(&["status"]);
    assert!(success_bin.status.success());
    assert!(!success_bin.stdout.is_empty());
    assert!(success_bin.stderr.is_empty());
    assert!(!success_bridge["stdout"].as_str().unwrap_or_default().is_empty());
    assert!(success_bridge["stderr"].as_str().unwrap_or_default().is_empty());

    let fail_bin = run_binary(&["unknown-command"]);
    let fail_bridge = bridge_outcome(&["unknown-command"]);
    assert!(fail_bin.stdout.is_empty());
    assert!(!fail_bin.stderr.is_empty());
    assert!(fail_bridge["stdout"].as_str().unwrap_or_default().is_empty());
    assert!(!fail_bridge["stderr"].as_str().unwrap_or_default().is_empty());
}

#[test]
fn route_registry_snapshots_match_across_binary_core_and_bridge() {
    let bin = run_binary(&["inspect"]);
    assert!(bin.status.success());
    let bin_payload = parse_json(&String::from_utf8_lossy(&bin.stdout));

    let core = run_core_cmd(&["inspect"]);
    let core_payload = parse_json(&core.stdout);

    assert_eq!(bin_payload, core_payload);

    let mut bridge_namespaces: Vec<String> = parse_json(&command_tree_introspection_api())
        .get("namespaces")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();
    bridge_namespaces.sort();
    bridge_namespaces.dedup();

    let namespaces_from_inspect = root_namespaces_from_inspect(&bin_payload);
    for required in ["cli", "dev", "plugins"] {
        assert!(namespaces_from_inspect.iter().any(|name| name == required));
        assert!(bridge_namespaces.iter().any(|name| name == required));
    }
}
