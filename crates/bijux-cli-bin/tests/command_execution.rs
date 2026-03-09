#![forbid(unsafe_code)]
//! Integration coverage for implemented built-in and developer commands.

use std::process::Command;

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
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

#[test]
fn executes_root_commands() {
    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["status"], "status"),
        (vec!["version"], "version"),
        (vec!["doctor"], "status"),
        (vec!["audit"], "checks"),
        (vec!["docs"], "topics"),
        (vec!["sleep", "0"], "slept_seconds"),
        (vec!["history"], "entries"),
        (vec!["memory"], "count"),
        (vec!["memory", "list"], "keys"),
        (vec!["plugins", "list"], "plugins"),
        (vec!["plugins", "inspect"], "status"),
        (vec!["plugins", "check", "sample"], "plugin"),
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
        (vec!["cli", "plugins", "inspect"], "compatibility_warnings"),
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
        (vec!["dev", "cli", "registry"], "registry"),
        (vec!["dev", "cli", "parity"], "rust_python"),
        (vec!["dev", "cli", "docs"], "docs_count"),
        (vec!["dev", "cli", "status"], "current_rust_state"),
        (vec!["dev", "cli", "scripts-audit"], "scripts"),
        (vec!["dev", "cli", "snapshots-audit"], "snapshots"),
        (vec!["dev", "cli", "fixture-audit"], "parity_fixtures"),
        (vec!["dev", "cli", "crate-health"], "crate_metrics"),
        (vec!["dev", "cli", "package-health"], "package_entrypoints"),
        (vec!["dev", "cli", "env"], "source_precedence"),
        (vec!["dev", "cli", "doctor"], "issues"),
        (vec!["dev", "cli", "contracts"], "contracts"),
        (vec!["dev", "cli", "runtime-identity"], "entrypoints"),
        (vec!["dev", "cli", "docs-prune-plan"], "target_cap"),
        (vec!["dev", "cli", "state-audit"], "paths"),
        (vec!["dev", "cli", "state-doctor"], "doctor"),
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
