#![forbid(unsafe_code)]
//! Dev CLI command surface matrix coverage and maintainer-control law tests.
//! test_type: dev-cli-command-surface

use std::collections::BTreeSet;
use std::process::{Command, Output};

use bijux_cli::interface::cli::dispatch::run_app;
use bijux_cli_python as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn run_ok_json(args: &[&str]) -> Value {
    let out = run(args);
    assert!(out.status.success(), "expected success for {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn parity_against_core(args: &[&str]) {
    let out = run(args);
    assert!(out.status.success(), "expected success for {args:?}");

    let mut argv = vec!["bijux".to_string()];
    argv.extend(args.iter().map(|a| a.to_string()));
    let core = run_app(&argv).expect("core should run");

    assert_eq!(out.status.code(), Some(core.exit_code));
    assert_eq!(String::from_utf8_lossy(&out.stdout), core.stdout);
    assert_eq!(String::from_utf8_lossy(&out.stderr), core.stderr);
}

#[test]
fn parity_for_key_dev_cli_commands_against_current_behavior() {
    parity_against_core(&["dev", "cli", "routes"]);
    parity_against_core(&["dev", "cli", "registry"]);
    parity_against_core(&["dev", "cli", "env"]);
    parity_against_core(&["dev", "cli", "doctor"]);
    parity_against_core(&["dev", "cli", "contracts"]);
    parity_against_core(&["dev", "cli", "status"]);
    parity_against_core(&["dev", "cli", "parity"]);
}

#[test]
fn help_snapshots_exist_for_all_dev_cli_subcommands() {
    let commands: Vec<Vec<&str>> =
        include_str!("../../data/fixtures/routing/dev_cli_subcommands.txt")
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.split_whitespace().collect::<Vec<_>>())
            .collect();

    for cmd in commands {
        let mut args = cmd.clone();
        args.push("--help");
        let first = run(&args);
        let second = run(&args);
        assert!(first.status.success(), "help failed for {cmd:?}");
        assert!(second.status.success(), "help failed for {cmd:?}");
        let text = String::from_utf8(first.stdout.clone()).expect("utf-8");
        assert!(text.contains("Usage:"), "missing Usage in help for {cmd:?}");
        assert_eq!(first.stdout, second.stdout, "help output drift for {cmd:?}");
    }
}

#[test]
fn json_and_text_outputs_are_available_for_machine_and_text_heavy_dev_cli_commands() {
    let json_cases: &[(&[&str], &str)] = &[
        (&["dev", "cli", "routes", "--format", "json", "--no-pretty"], "routes"),
        (&["dev", "cli", "registry", "--format", "json", "--no-pretty"], "registry"),
        (&["dev", "cli", "env", "--format", "json", "--no-pretty"], "source_precedence"),
        (&["dev", "cli", "doctor", "--format", "json", "--no-pretty"], "issues"),
        (&["dev", "cli", "contracts", "--format", "json", "--no-pretty"], "contracts"),
        (&["dev", "cli", "status", "--format", "json", "--no-pretty"], "command_migration"),
        (&["dev", "cli", "parity", "--format", "json", "--no-pretty"], "command_matrix"),
    ];

    for (args, key) in json_cases {
        let out = run(args);
        assert!(out.status.success(), "json run failed for {args:?}");
        let payload: Value = serde_json::from_slice(&out.stdout).expect("json parse");
        assert!(payload.get(*key).is_some(), "missing key {key} for {args:?}");
        assert!(out.stderr.is_empty(), "stderr should be empty for successful json output");
    }

    let text_cases: &[&[&str]] = &[
        &["dev", "cli", "routes", "--format", "text"],
        &["dev", "cli", "registry", "--format", "text"],
        &["dev", "cli", "env", "--format", "text"],
        &["dev", "cli", "contracts", "--format", "text"],
        &["dev", "cli", "state-doctor", "--format", "text"],
    ];
    for args in text_cases {
        let out = run(args);
        assert!(out.status.success(), "text run failed for {args:?}");
        let text = String::from_utf8(out.stdout).expect("utf-8");
        assert!(!text.trim().is_empty(), "text output empty for {args:?}");
    }
}

#[test]
fn stderr_stdout_and_exit_code_discipline_for_dev_cli_commands() {
    let success_cases: &[&[&str]] = &[
        &["dev", "cli", "routes"],
        &["dev", "cli", "registry"],
        &["dev", "cli", "env"],
        &["dev", "cli", "doctor"],
        &["dev", "cli", "contracts"],
    ];
    for args in success_cases {
        let out = run(args);
        assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
        assert!(!out.stdout.is_empty(), "stdout should not be empty for {args:?}");
        assert!(out.stderr.is_empty(), "stderr should be empty for {args:?}");
    }

    let fail_cases: &[&[&str]] =
        &[&["dev", "cli", "does-not-exist"], &["dev", "cli", "state-doctor", "invalid"]];
    for args in fail_cases {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "expected failure for {args:?}");
        assert!(out.stdout.is_empty(), "stdout should be empty for failure {args:?}");
        assert!(!out.stderr.is_empty(), "stderr should be present for failure {args:?}");
    }
}

#[test]
fn malformed_input_is_rejected_for_dev_cli_subcommands() {
    let malformed: &[&[&str]] = &[
        &["dev", "cli", "state-doctor", "bad-mode"],
        &["dev", "cli", "route-audit", "--unknown-flag"],
        &["dev", "cli", "status", "--unknown-flag"],
    ];
    for args in malformed {
        let out = run(args);
        assert_ne!(out.status.code(), Some(0), "malformed input should fail for {args:?}");
        assert!(out.stdout.is_empty(), "stdout should be empty for malformed failure {args:?}");
        assert!(!out.stderr.is_empty(), "stderr should be present for malformed failure {args:?}");
    }
}

#[test]
fn repeated_run_determinism_for_machine_readable_dev_cli_commands() {
    let deterministic: &[&[&str]] = &[
        &["dev", "cli", "routes", "--format", "json", "--no-pretty"],
        &["dev", "cli", "registry", "--format", "json", "--no-pretty"],
        &["dev", "cli", "env", "--format", "json", "--no-pretty"],
        &["dev", "cli", "doctor", "--format", "json", "--no-pretty"],
        &["dev", "cli", "contracts", "--format", "json", "--no-pretty"],
    ];
    for args in deterministic {
        let first = run(args);
        let second = run(args);
        assert!(first.status.success(), "first run failed for {args:?}");
        assert!(second.status.success(), "second run failed for {args:?}");
        assert_eq!(first.stdout, second.stdout, "stdout drift for {args:?}");
        assert_eq!(first.stderr, second.stderr, "stderr drift for {args:?}");
    }
}

#[test]
fn consistency_across_dev_cli_routes_inspect_and_registry_state() {
    let routes = run_ok_json(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let inspect = run_ok_json(&["inspect", "--format", "json", "--no-pretty"]);
    let registry = run_ok_json(&["dev", "cli", "registry", "--format", "json", "--no-pretty"]);

    let route_roots: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes array")
        .iter()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();

    let inspect_roots: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route_sources array")
        .iter()
        .filter_map(|row| row.get("segments"))
        .filter_map(Value::as_array)
        .filter_map(|segments| segments.first())
        .filter_map(Value::as_str)
        .map(ToString::to_string)
        .collect();

    assert!(!route_roots.is_empty());
    assert!(!inspect_roots.is_empty());
    assert!(route_roots.iter().all(|root| inspect_roots.contains(root)));
    assert!(registry.get("registry").is_some(), "dev cli registry payload missing registry field");
}

#[test]
fn consistency_across_dev_cli_env_and_config_resolution_paths() {
    let root =
        std::env::temp_dir().join(format!("bijux-dev-cli-env-consistency-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("mkdir");
    let config = root.join("config.env");
    std::fs::write(&config, "BIJUXCLI_ALPHA=from-file\n").expect("write config");

    let config_text = config.to_string_lossy().to_string();
    let env_report_out = run_with_env(
        &["dev", "cli", "env", "--format", "json", "--no-pretty", "--config-path", &config_text],
        &[("BIJUXCLI_CONFIG", &config_text)],
    );
    assert!(env_report_out.status.success());
    let env_payload: Value = serde_json::from_slice(&env_report_out.stdout).expect("env json");
    assert_eq!(env_payload["active"]["config_file"], config_text);

    let config_get_out = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        &config_text,
    ]);
    assert!(config_get_out.status.success());
    let config_payload: Value =
        serde_json::from_slice(&config_get_out.stdout).expect("config json");
    assert_eq!(config_payload["value"], "from-file");

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn dev_cli_command_matrix_artifact_smoke_uses_supported_commands() {
    let checks: &[&[&str]] = &[
        &["dev", "cli", "routes"],
        &["dev", "cli", "registry"],
        &["dev", "cli", "env"],
        &["dev", "cli", "doctor"],
        &["dev", "cli", "status"],
        &["dev", "cli", "parity"],
    ];
    for args in checks {
        let out = run(args);
        assert!(out.status.success(), "matrix command should succeed for {args:?}");
        let payload: Value = serde_json::from_slice(&out.stdout).expect("json payload");
        assert!(payload.is_object());
    }
}
