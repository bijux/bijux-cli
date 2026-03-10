#![forbid(unsafe_code)]
//! Integration coverage for implemented built-in and developer commands.

use std::process::Command;
use std::{env, fs};

use bijux_cli_core as _;
use libc as _;
use serde_json as _;

fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "process failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

fn run_raw(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], key: &str, value: &str) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env(key, value)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "process failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn executes_root_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["status"], "status"),
        (vec!["version"], "version"),
        (vec!["doctor"], "status"),
        (vec!["audit"], "checks"),
        (vec!["docs"], "topics"),
        (vec!["atlas"], "mount"),
        (vec!["sleep", "0"], "slept_seconds"),
        (vec!["history"], "entries"),
        (vec!["memory"], "count"),
        (vec!["memory", "list"], "keys"),
        (vec!["plugins", "list"], "plugins"),
        (vec!["plugins"], "plugins"),
        (vec!["plugins", "info"], "plugins"),
        (vec!["plugins", "inspect"], "status"),
        (vec!["plugins", "doctor"], "doctor"),
        (vec!["plugins", "reserved-names"], "reserved_namespaces"),
        (vec!["plugins", "where"], "plugins_dir"),
        (vec!["plugins", "explain"], "diagnostics"),
        (vec!["plugins", "schema"], "schema"),
        (vec!["repl"], "mode"),
        (vec!["completion"], "shells"),
        (vec!["inspect"], "route_sources"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("root command must emit valid json");
        if args == vec!["config"] {
            assert!(payload.is_object(), "config root should return object payload");
            continue;
        }
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn executes_cli_namespace_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["cli", "status"], "runtime"),
        (vec!["cli", "paths"], "path_binaries"),
        (vec!["cli", "config", "set", "TEST_KEY=1"], "status"),
        (vec!["cli", "self-test"], "checks"),
        (vec!["cli", "plugins", "list"], "plugins"),
        (vec!["cli", "plugins", "info"], "plugins"),
        (vec!["cli", "plugins", "inspect"], "compatibility_warnings"),
        (vec!["cli", "plugins", "doctor"], "doctor"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("cli command must emit valid json");
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn cli_paths_reports_active_binary_metadata() {
    let stdout = run(&["cli", "paths"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload.get("active_binary").is_some());
    assert!(payload.get("path_binaries").is_some());
    assert!(payload.get("post_install_hint").is_some());
}

#[test]
fn cli_doctor_reports_install_diagnostics() {
    let stdout = run(&["doctor"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let install = payload.get("install").expect("install diagnostics");
    assert!(install.get("has_path_shadowing").is_some());
    assert!(install.get("has_duplicate_installs").is_some());
    assert!(install.get("stale_wrapper_scripts").is_some());
    assert!(install.get("legacy_installer_conflicts").is_some());
    assert!(install.get("has_mismatched_wheel_binary_versions").is_some());
}

#[test]
fn executes_dev_cli_namespace_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["dev", "cli", "inventory"], "scripts"),
        (vec!["dev", "cli", "routes"], "routes"),
        (vec!["dev", "cli", "route-audit"], "summary"),
        (vec!["dev", "cli", "registry"], "registry"),
        (vec!["dev", "cli", "parity"], "rust_python"),
        (vec!["dev", "cli", "docs"], "docs_count"),
        (vec!["dev", "cli", "docs-audit"], "docs_audit"),
        (vec!["dev", "cli", "plugin-health"], "machine_report"),
        (vec!["dev", "cli", "status"], "current_rust_state"),
        (vec!["dev", "cli", "script-audit"], "scripts"),
        (vec!["dev", "cli", "snapshots-audit"], "snapshots"),
        (vec!["dev", "cli", "fixture-audit"], "parity_fixtures"),
        (vec!["dev", "cli", "crate-health"], "crate_metrics"),
        (vec!["dev", "cli", "package-health"], "install_state_assumptions"),
        (vec!["dev", "cli", "env"], "source_precedence"),
        (vec!["dev", "cli", "doctor"], "issues"),
        (vec!["dev", "cli", "contracts"], "contracts"),
        (vec!["dev", "cli", "runtime-identity"], "entrypoints"),
        (vec!["dev", "cli", "docs-prune-plan"], "target_cap"),
        (vec!["dev", "cli", "state-audit"], "paths"),
        (vec!["dev", "cli", "state-doctor"], "doctor"),
        (vec!["dev", "cli", "atlas"], "mount"),
        (vec!["dev", "cli", "di"], "container"),
        (vec!["dev", "cli", "list-products"], "products"),
        (vec!["dev", "cli", "list-plugins"], "plugins"),
    ];
    for (args, required_key) in cases {
        let stdout = run(&args);
        let payload: serde_json::Value =
            serde_json::from_str(&stdout).expect("dev cli command must emit valid json");
        assert!(
            payload.get(required_key).is_some(),
            "expected key `{required_key}` for args {args:?}"
        );
    }
}

#[test]
fn unsupported_config_set_input_returns_usage_error() {
    let output = run_raw(&["cli", "config", "set", "INVALID_PAIR"]);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    assert!(stderr.contains("Invalid argument"), "unexpected stderr: {stderr}");
}

#[test]
fn runtime_identity_reports_ambiguous_active_binary_selection() {
    let temp = env::temp_dir().join(format!("bijux-runtime-identity-{}", std::process::id()));
    let _ = fs::remove_dir_all(&temp);
    let first = temp.join("first");
    let second = temp.join("second");
    fs::create_dir_all(&first).expect("create first dir");
    fs::create_dir_all(&second).expect("create second dir");
    fs::write(first.join("bijux"), b"#!/bin/sh\n").expect("write first binary");
    fs::write(second.join("bijux"), b"#!/bin/sh\n").expect("write second binary");
    let path_value = env::join_paths([&first, &second]).expect("join path");

    let stdout = run_with_env(
        &["dev", "cli", "runtime-identity"],
        "PATH",
        path_value.to_str().expect("utf-8 path"),
    );
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(payload["active_binary_selection_is_ambiguous"], true);
    assert_eq!(payload["diagnostics"]["path_shadowing_detected"], true);
    let binaries = payload["path_binaries"].as_array().expect("array");
    assert!(binaries.len() >= 2);

    fs::remove_dir_all(&temp).expect("cleanup temp");
}

#[test]
fn runtime_identity_reports_python_bridge_support_diagnostic() {
    let stdout = run_with_env(
        &["dev", "cli", "runtime-identity"],
        "BIJUX_PYTHON_BRIDGE_SUPPORTED",
        "0",
    );
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert_eq!(payload["diagnostics"]["python_bridge_supported"], false);
}

#[test]
fn package_health_exposes_install_state_assumptions() {
    let stdout = run(&["dev", "cli", "package-health"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    let assumptions = payload["install_state_assumptions"].as_array().expect("array");
    assert!(!assumptions.is_empty());
}

#[test]
fn crate_health_exposes_decision_report_payload() {
    let stdout = run(&["dev", "cli", "crate-health"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload["crate_metrics"].is_object());
    assert!(payload["crate_report"].is_object());
}

#[test]
fn dev_cli_status_surfaces_next_phase_priorities() {
    let stdout = run(&["dev", "cli", "status", "--format", "json", "--no-pretty"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload["next_phase_priorities"].is_object());
    assert!(payload["next_phase_summary_text"].is_string());
    assert!(payload["command_migration"].is_object());
}

#[test]
fn dev_cli_parity_surfaces_dashboard_artifact() {
    let stdout = run(&["dev", "cli", "parity", "--format", "json", "--no-pretty"]);
    let payload: serde_json::Value = serde_json::from_str(&stdout).expect("valid json");
    assert!(payload["parity_dashboard"].is_object());
    assert!(payload["parity_dashboard_text"].is_string());
}
