#![forbid(unsafe_code)]
//! Exit-code law matrix and cross-surface failure-class consistency checks.
//! test_type: exit-code-law

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use bijux_cli::repl::{execute_repl_line, startup_repl};
use bijux_cli_python as _;
use bijux_cli_python::execution_outcome_api;
use libc as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn code(args: &[&str]) -> i32 {
    run(args).status.code().unwrap_or(-1)
}

fn bridge_exit(args: &[&str]) -> i32 {
    let argv = std::iter::once("bijux".to_string())
        .chain(args.iter().map(|item| item.to_string()))
        .collect::<Vec<_>>();
    let value: Value =
        serde_json::from_str(&execution_outcome_api(&argv).expect("bridge execution"))
            .expect("bridge json");
    value["exit_code"].as_i64().unwrap_or(-1) as i32
}

fn temp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bijux-exit-code-law-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn root_command_exit_code_matrix_is_complete_and_stable() {
    let cases = [
        (vec!["version"], 0),
        (vec!["status"], 0),
        (vec!["doctor"], 0),
        (vec!["inspect"], 0),
        (vec!["docs"], 0),
        (vec!["audit"], 0),
        (vec!["sleep", "0"], 0),
        (vec!["sleep", "--bad-flag"], 2),
    ];
    for (args, expected) in cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected exit code for root args {args:?}"
        );
    }
}

#[test]
fn cli_command_exit_code_matrix_is_complete_and_stable() {
    let cases = [
        (vec!["cli", "status"], 0),
        (vec!["cli", "paths"], 0),
        (vec!["cli", "self-test"], 0),
        (vec!["cli", "config", "get"], 2),
        (vec!["cli", "config", "set", "BIJUXCLI_X=1"], 0),
        (vec!["cli", "plugins", "list"], 0),
        (vec!["cli", "plugins", "inspect"], 0),
    ];
    for (args, expected) in cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected exit code for cli args {args:?}"
        );
    }
}

#[test]
fn dev_cli_command_exit_code_matrix_is_complete_and_stable() {
    let cases = [
        (vec!["dev", "cli", "routes"], 0),
        (vec!["dev", "cli", "registry"], 0),
        (vec!["dev", "cli", "env"], 0),
        (vec!["dev", "cli", "doctor"], 0),
        (vec!["dev", "cli", "contracts"], 0),
        (vec!["dev", "cli", "status"], 0),
        (vec!["dev", "cli", "parity"], 0),
        (vec!["dev", "cli", "does-not-exist"], 2),
    ];
    for (args, expected) in cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected exit code for dev cli args {args:?}"
        );
    }
}

#[test]
fn plugin_lifecycle_command_exit_code_matrix_is_complete_and_stable() {
    let cases = [
        (vec!["plugins", "list"], 0),
        (vec!["plugins", "inspect"], 0),
        (vec!["plugins", "doctor"], 0),
        (vec!["plugins", "uninstall"], 1),
        (vec!["plugins", "enable"], 1),
        (vec!["plugins", "disable"], 1),
    ];
    for (args, expected) in cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected exit code for plugin args {args:?}"
        );
    }
}

#[test]
fn config_history_memory_and_diagnostics_exit_code_matrices_are_complete_and_stable() {
    let config_cases = [
        (vec!["cli", "config", "list"], 0),
        (vec!["cli", "config", "get"], 2),
        (vec!["cli", "config", "set", "INVALID"], 2),
    ];
    for (args, expected) in config_cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected config exit code for {args:?}"
        );
    }

    let history_cases = [
        (vec!["history"], 0),
        (vec!["history", "--format", "json", "--no-pretty"], 0),
        (vec!["history", "--bad-flag"], 2),
    ];
    for (args, expected) in history_cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected history exit code for {args:?}"
        );
    }

    let memory_cases = [
        (vec!["memory"], 0),
        (vec!["memory", "list", "--format", "json", "--no-pretty"], 0),
        (vec!["memory", "set"], 2),
    ];
    for (args, expected) in memory_cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected memory exit code for {args:?}"
        );
    }

    let diagnostics_cases = [
        (vec!["inspect"], 0),
        (vec!["doctor"], 0),
        (vec!["dev", "cli", "state-doctor", "invalid"], 2),
    ];
    for (args, expected) in diagnostics_cases {
        assert_eq!(
            code(&args),
            expected,
            "unexpected diagnostics exit code for {args:?}"
        );
    }
}

#[test]
fn identical_usage_and_validation_failures_map_to_same_code_across_surfaces() {
    let usage_root = code(&["config", "get"]);
    let usage_cli = code(&["cli", "config", "get"]);
    let usage_dev = code(&["dev", "cli", "does-not-exist"]);
    assert_eq!(usage_root, 2);
    assert_eq!(usage_cli, usage_root);
    assert_eq!(usage_dev, usage_root);

    let validation_root = code(&["--format", "not-a-format", "status"]);
    let validation_cli = code(&["--format", "not-a-format", "cli", "status"]);
    let validation_dev = code(&["--format", "not-a-format", "dev", "cli", "status"]);
    assert_eq!(validation_root, 1);
    assert_eq!(validation_cli, validation_root);
    assert_eq!(validation_dev, validation_root);
}

#[test]
fn identical_plugin_and_internal_failure_classes_map_to_same_code_across_surfaces() {
    let plugin_root = code(&["plugins", "uninstall"]);
    let plugin_cli = code(&["cli", "plugins", "uninstall"]);
    assert_eq!(plugin_root, 1);
    assert_eq!(plugin_cli, plugin_root);

    let internal_like_root = code(&["plugins", "enable"]);
    let internal_like_cli = code(&["cli", "plugins", "enable"]);
    assert_eq!(internal_like_root, 1);
    assert_eq!(internal_like_cli, internal_like_root);
}

#[test]
fn binary_python_bridge_and_repl_agree_on_exit_code_classes_for_covered_commands() {
    let cases: Vec<Vec<&str>> = vec![
        vec!["status", "--format", "json", "--no-pretty"],
        vec!["config", "get"],
        vec!["plugins", "uninstall"],
        vec!["dev", "cli", "does-not-exist"],
    ];

    let mut session = startup_repl("", None).0;
    for args in cases {
        let refs = args;
        let bin = code(&refs);
        let bridge = bridge_exit(&refs);
        assert_eq!(bridge, bin, "bridge exit drift for {refs:?}");

        let line = refs.join(" ");
        let _ = execute_repl_line(&mut session, &line).expect("repl execute");
        assert_eq!(session.last_exit_code, bin, "repl exit drift for {refs:?}");
    }
}

#[test]
fn machine_readable_and_text_failures_keep_same_exit_codes() {
    let text_usage = code(&["cli", "config", "get", "--format", "text"]);
    let json_usage = code(&["cli", "config", "get", "--format", "json", "--no-pretty"]);
    let yaml_usage = code(&["cli", "config", "get", "--format", "yaml"]);
    assert_eq!(text_usage, 2);
    assert_eq!(json_usage, text_usage);
    assert_eq!(yaml_usage, text_usage);
}

#[test]
fn corrupted_state_and_missing_file_failures_do_not_drift_in_exit_class() {
    let root = temp_dir("corrupted-state");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_OK=1\nMALFORMED\n").expect("write malformed config");
    let config_text = config.to_string_lossy().to_string();

    let corrupted_text = code(&["config", "--config-path", &config_text]);
    let corrupted_json = code(&[
        "config",
        "--config-path",
        &config_text,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(corrupted_text, 1);
    assert_eq!(corrupted_json, corrupted_text);

    let missing = root.join("missing.env").to_string_lossy().to_string();
    let missing_text = code(&["config", "get", "alpha", "--config-path", &missing]);
    let missing_json = code(&[
        "config",
        "get",
        "alpha",
        "--config-path",
        &missing,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(missing_text, 2);
    assert_eq!(missing_json, missing_text);
}
