#![forbid(unsafe_code)]
//! CLI command surface coverage and explicit public-law tests.
//! test_type: cli-command-surface

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli::api::runtime::run_app;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

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
    let root = std::env::temp_dir()
        .join(format!("bijux-cli-command-coverage-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn parity_against_core(args: &[&str]) {
    let out = run(args);
    assert!(out.status.success(), "expected success for {args:?}");
    assert!(out.stderr.is_empty(), "successful parity command must keep stderr empty: {args:?}");
    assert!(!out.stdout.is_empty(), "successful parity command must emit stdout: {args:?}");

    let mut argv = vec!["bijux".to_string()];
    argv.extend(args.iter().map(|a| (*a).to_string()));
    let core = run_app(&argv).expect("core should run");

    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_cli_status_paths_and_self_test_against_current_behavior() {
    parity_against_core(&["cli", "status"]);
    parity_against_core(&["cli", "paths"]);
    parity_against_core(&["cli", "self-test"]);
}

#[test]
fn parity_cli_config_get_and_set_against_current_behavior() {
    let root = temp_dir("parity-config");
    let config = root.join("config.env");
    let config_text = config.to_string_lossy().to_string();

    parity_against_core(&["cli", "config", "set", "coverage_key=42", "--config-path", &config_text]);
    parity_against_core(&["cli", "config", "get", "coverage_key", "--config-path", &config_text]);
}

#[test]
fn parity_cli_plugins_list_and_inspect_against_current_behavior() {
    parity_against_core(&["cli", "plugins", "list"]);
    parity_against_core(&["cli", "plugins", "inspect"]);
}

#[test]
fn help_snapshots_exist_for_all_cli_subcommands() {
    let commands: &[&[&str]] = &[
        &["cli", "status"],
        &["cli", "paths"],
        &["cli", "self-test"],
        &["cli", "config", "get"],
        &["cli", "config", "list"],
        &["cli", "config", "set"],
        &["cli", "config", "unset"],
        &["cli", "config", "clear"],
        &["cli", "config", "reload"],
        &["cli", "config", "export"],
        &["cli", "config", "load"],
        &["cli", "plugins", "list"],
        &["cli", "plugins", "info"],
        &["cli", "plugins", "inspect"],
        &["cli", "plugins", "check"],
        &["cli", "plugins", "install"],
        &["cli", "plugins", "uninstall"],
        &["cli", "plugins", "enable"],
        &["cli", "plugins", "disable"],
        &["cli", "plugins", "scaffold"],
        &["cli", "plugins", "doctor"],
        &["cli", "plugins", "reserved-names"],
        &["cli", "plugins", "where"],
        &["cli", "plugins", "explain"],
        &["cli", "plugins", "schema"],
    ];

    for cmd in commands {
        let mut args = cmd.to_vec();
        args.push("--help");
        let first = run(&args);
        let second = run(&args);
        assert!(first.status.success(), "help failed for {cmd:?}");
        assert!(second.status.success(), "help failed for {cmd:?}");
        assert!(first.stderr.is_empty(), "help stderr should be empty for {cmd:?}");
        assert!(second.stderr.is_empty(), "help stderr should be empty for {cmd:?}");
        let first_text = String::from_utf8(first.stdout.clone()).expect("utf-8");
        assert!(first_text.contains("Usage:"), "help for {cmd:?} missing Usage");
        assert_eq!(first.stdout, second.stdout, "help output drift for {cmd:?}");
    }
}

#[test]
fn stderr_stdout_and_exit_code_discipline_for_cli_commands() {
    let success_cases: &[&[&str]] = &[
        &["cli", "status"],
        &["cli", "paths"],
        &["cli", "self-test"],
        &["cli", "config", "list"],
        &["cli", "plugins", "list"],
    ];
    for args in success_cases {
        let out = run(args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(!out.stdout.is_empty(), "expected stdout for {args:?}");
        assert!(out.stderr.is_empty(), "expected empty stderr for {args:?}");
    }

    let failure_cases: &[&[&str]] = &[
        &["cli", "config", "get"],
        &["cli", "config", "set", "BADPAIR"],
        &["cli", "plugins", "uninstall"],
    ];
    for args in failure_cases {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "expected failure for {args:?}");
        assert!(out.stdout.is_empty(), "expected empty stdout for failure {args:?}");
        assert!(!out.stderr.is_empty(), "expected stderr for failure {args:?}");
    }
}

#[test]
fn machine_readable_cli_commands_support_json_and_yaml() {
    let root = temp_dir("formats");
    let config = root.join("config.env");
    fs::write(&config, "BIJUXCLI_FMT=1\n").expect("write config");
    let config_text = config.to_string_lossy().to_string();

    let machine_cases: Vec<Vec<String>> = vec![
        vec!["cli", "status"],
        vec!["cli", "paths"],
        vec!["cli", "self-test"],
        vec!["cli", "config", "list", "--config-path", &config_text],
        vec!["cli", "config", "get", "fmt", "--config-path", &config_text],
        vec!["cli", "plugins", "list"],
        vec!["cli", "plugins", "inspect"],
    ]
    .into_iter()
    .map(|x| x.into_iter().map(str::to_string).collect())
    .collect();

    for base in machine_cases {
        let mut json_args = base.clone();
        json_args.extend(["--format".to_string(), "json".to_string(), "--no-pretty".to_string()]);
        let json_refs: Vec<&str> = json_args.iter().map(String::as_str).collect();
        let json_out = run(&json_refs);
        assert!(json_out.status.success(), "json failed for {base:?}");
        assert!(json_out.stderr.is_empty(), "json command should not write to stderr for {base:?}");
        let _: Value = serde_json::from_slice(&json_out.stdout).expect("json parse");

        let mut yaml_args = base.clone();
        yaml_args.extend(["--format".to_string(), "yaml".to_string(), "--pretty".to_string()]);
        let yaml_refs: Vec<&str> = yaml_args.iter().map(String::as_str).collect();
        let yaml_out = run(&yaml_refs);
        assert!(yaml_out.status.success(), "yaml failed for {base:?}");
        assert!(yaml_out.stderr.is_empty(), "yaml command should not write to stderr for {base:?}");
        let yaml_text = String::from_utf8(yaml_out.stdout).expect("utf-8");
        assert!(!yaml_text.trim().is_empty());
    }
}

#[test]
fn quiet_mode_and_no_color_behavior_for_relevant_cli_commands() {
    let quiet_cases: &[&[&str]] = &[
        &["cli", "status"],
        &["cli", "paths"],
        &["cli", "self-test"],
        &["cli", "plugins", "list"],
    ];

    for base in quiet_cases {
        let mut args = vec!["--quiet"];
        args.extend(base.iter().copied());
        let out = run(&args);
        assert!(out.status.success(), "quiet failed for {base:?}");
        assert!(out.stdout.is_empty(), "quiet stdout should be empty for {base:?}");
        assert!(out.stderr.is_empty(), "quiet stderr should be empty for {base:?}");
    }

    let text_cases: &[&[&str]] = &[
        &["cli", "status", "--format", "text"],
        &["cli", "paths", "--format", "text"],
        &["cli", "plugins", "list", "--format", "text"],
    ];
    for base in text_cases {
        let out = run_with_env(base, &[("NO_COLOR", "1")]);
        assert!(out.status.success(), "no-color failed for {base:?}");
        assert!(out.stderr.is_empty(), "no-color success should keep stderr empty for {base:?}");
        let text = String::from_utf8(out.stdout).expect("utf-8");
        assert!(!text.contains("\u{1b}["));
    }
}

#[test]
fn malformed_input_is_rejected_for_argument_taking_cli_subcommands() {
    let malformed: &[&[&str]] = &[
        &["cli", "config", "get"],
        &["cli", "config", "set", "NO_EQUALS"],
        &["cli", "config", "unset"],
        &["cli", "plugins", "install"],
        &["cli", "plugins", "enable"],
        &["cli", "plugins", "disable"],
        &["cli", "config", "load"],
    ];
    for args in malformed {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "malformed input should fail for {args:?}");
        assert!(out.stdout.is_empty(), "malformed input should not use stdout for {args:?}");
        assert!(!out.stderr.is_empty(), "malformed input should use stderr for {args:?}");
        let stderr: Value = serde_json::from_slice(&out.stderr).expect("malformed stderr json");
        assert_eq!(stderr["status"], "error");
        assert!(stderr["code"].as_i64().unwrap_or(0) > 0);
        assert!(
            stderr["message"].as_str().is_some_and(|msg| !msg.trim().is_empty()),
            "malformed input should emit actionable diagnostics for {args:?}"
        );
    }
}

#[test]
fn repeated_run_stability_for_machine_readable_cli_commands() {
    let deterministic: &[&[&str]] = &[
        &["cli", "status", "--format", "json", "--no-pretty"],
        &["cli", "paths", "--format", "json", "--no-pretty"],
        &["cli", "self-test", "--format", "json", "--no-pretty"],
        &["cli", "plugins", "list", "--format", "json", "--no-pretty"],
        &["cli", "plugins", "inspect", "--format", "json", "--no-pretty"],
    ];
    for args in deterministic {
        let first = run(args);
        let second = run(args);
        assert!(first.status.success(), "first run failed for {args:?}");
        assert!(second.status.success(), "second run failed for {args:?}");
        assert!(first.stderr.is_empty(), "first run should keep stderr empty for {args:?}");
        assert!(second.stderr.is_empty(), "second run should keep stderr empty for {args:?}");
        let first_json: Value = serde_json::from_slice(&first.stdout).expect("first json payload");
        let second_json: Value =
            serde_json::from_slice(&second.stdout).expect("second json payload");
        assert!(first_json.is_object(), "first payload should be object for {args:?}");
        assert!(second_json.is_object(), "second payload should be object for {args:?}");
        assert_eq!(first.stdout, second.stdout, "stdout drift for {args:?}");
        assert_eq!(first.stderr, second.stderr, "stderr drift for {args:?}");
    }
}

#[test]
fn cli_command_coverage_artifact_smoke_uses_supported_commands() {
    let checks: &[(&[&str], Option<&str>, bool)] = &[
        (&["cli", "status"], Some("runtime"), false),
        (&["cli", "paths"], Some("active_binary"), false),
        (&["cli", "self-test"], Some("checks"), false),
        (&["cli", "config", "list"], None, true),
        (&["cli", "plugins", "list"], Some("plugins"), false),
    ];
    for (args, required_key, allow_empty_object) in checks {
        let out = run(args);
        assert!(out.status.success(), "coverage command should succeed for {args:?}");
        assert!(
            out.stderr.is_empty(),
            "successful coverage command should keep stderr empty for {args:?}"
        );
        let payload: Value = serde_json::from_slice(&out.stdout).expect("json payload");
        let object = payload.as_object().expect("coverage payload should be object");
        if !allow_empty_object {
            assert!(!object.is_empty(), "coverage payload should not be empty for {args:?}");
        }
        if let Some(key) = required_key {
            assert!(
                object.contains_key(*key),
                "coverage payload missing required key `{key}` for {args:?}"
            );
        }
    }
}
