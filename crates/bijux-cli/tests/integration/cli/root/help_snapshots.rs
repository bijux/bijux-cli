#![forbid(unsafe_code)]
//! Help snapshot coverage for root, cli, and dev-cli command surfaces.

use std::process::Command;
use std::time::{Duration, Instant};

use bijux_cli as _;
use libc as _;
use serde_json as _;
use shlex as _;
use thiserror as _;

fn run_help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "help command failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

fn run_help_with_env(args: &[&str], envs: &[(&str, &str)]) -> String {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let output = cmd.output().expect("binary should execute");
    assert!(output.status.success(), "help command failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn help_snapshots_match_expected_output() {
    let cases: [(&[&str], &str); 45] = [
        (&["--help"], include_str!("../../../data/golden/cli_surface/help_root.txt")),
        (
            &["--color", "never", "--help"],
            include_str!("../../../data/golden/cli_surface/help_root_no_color.txt"),
        ),
        (&["cli", "--help"], include_str!("../../../data/golden/cli_surface/help_cli.txt")),
        (&["dev", "--help"], include_str!("../../../data/golden/cli_surface/help_dev.txt")),
        (&["status", "--help"], include_str!("../../../data/golden/cli_surface/help_status.txt")),
        (&["audit", "--help"], include_str!("../../../data/golden/cli_surface/help_audit.txt")),
        (&["docs", "--help"], include_str!("../../../data/golden/cli_surface/help_docs.txt")),
        (&["sleep", "--help"], include_str!("../../../data/golden/cli_surface/help_sleep.txt")),
        (&["version", "--help"], include_str!("../../../data/golden/cli_surface/help_version.txt")),
        (&["doctor", "--help"], include_str!("../../../data/golden/cli_surface/help_doctor.txt")),
        (&["config", "--help"], include_str!("../../../data/golden/cli_surface/help_config.txt")),
        (&["plugins", "--help"], include_str!("../../../data/golden/cli_surface/help_plugins.txt")),
        (
            &["plugins", "install", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_install.txt"),
        ),
        (
            &["plugins", "uninstall", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_uninstall.txt"),
        ),
        (
            &["plugins", "scaffold", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_scaffold.txt"),
        ),
        (
            &["plugins", "doctor", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_doctor.txt"),
        ),
        (
            &["plugins", "reserved-names", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_reserved_names.txt"),
        ),
        (
            &["plugins", "where", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_where.txt"),
        ),
        (
            &["plugins", "explain", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_explain.txt"),
        ),
        (
            &["plugins", "schema", "--help"],
            include_str!("../../../data/golden/cli_surface/help_plugins_schema.txt"),
        ),
        (&["repl", "--help"], include_str!("../../../data/golden/cli_surface/help_repl.txt")),
        (
            &["completion", "--help"],
            include_str!("../../../data/golden/cli_surface/help_completion.txt"),
        ),
        (&["inspect", "--help"], include_str!("../../../data/golden/cli_surface/help_inspect.txt")),
        (&["history", "--help"], include_str!("../../../data/golden/cli_surface/help_history.txt")),
        (&["memory", "--help"], include_str!("../../../data/golden/cli_surface/help_memory.txt")),
        (
            &["cli", "status", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_status.txt"),
        ),
        (
            &["cli", "paths", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_paths.txt"),
        ),
        (
            &["cli", "config", "get", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_config_get.txt"),
        ),
        (
            &["cli", "config", "set", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_config_set.txt"),
        ),
        (
            &["cli", "self-test", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_self_test.txt"),
        ),
        (
            &["cli", "plugins", "list", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_list.txt"),
        ),
        (
            &["cli", "plugins", "inspect", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_inspect.txt"),
        ),
        (
            &["cli", "plugins", "install", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_install.txt"),
        ),
        (
            &["cli", "plugins", "uninstall", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_uninstall.txt"),
        ),
        (
            &["cli", "plugins", "scaffold", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_scaffold.txt"),
        ),
        (
            &["cli", "plugins", "doctor", "--help"],
            include_str!("../../../data/golden/cli_surface/help_cli_plugins_doctor.txt"),
        ),
        (
            &["dev", "cli", "routes", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_routes.txt"),
        ),
        (
            &["dev", "cli", "registry", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_registry.txt"),
        ),
        (
            &["dev", "cli", "env", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_env.txt"),
        ),
        (
            &["dev", "cli", "doctor", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_doctor.txt"),
        ),
        (
            &["dev", "cli", "contracts", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_contracts.txt"),
        ),
        (
            &["dev", "cli", "runtime-identity", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_runtime_identity.txt"),
        ),
        (
            &["dev", "cli", "state-audit", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_state_audit.txt"),
        ),
        (
            &["dev", "cli", "state-doctor", "--help"],
            include_str!("../../../data/golden/cli_surface/help_dev_cli_state_doctor.txt"),
        ),
        (
            &["status", "--format", "json", "--help"],
            include_str!("../../../data/golden/cli_surface/help_status.txt"),
        ),
    ];

    for (args, expected) in cases {
        let actual = run_help(args);
        assert_eq!(actual, expected, "help snapshot mismatch for args: {args:?}");
    }
}

#[test]
fn nested_help_and_unknown_command_diagnostics_are_stable() {
    let nested = run_help(&["cli", "--help"]);
    assert!(nested.contains("Usage:"));
    assert!(nested.contains("status"));

    let output = Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(["stattus", "--format", "json", "--no-pretty"])
        .output()
        .expect("binary should execute");
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).expect("stderr utf-8");
    assert!(stderr.contains("unknown route: stattus"));
}

#[test]
fn hidden_alias_help_matches_canonical_options_and_flags() {
    let alias_plugins = run_help(&["plugins", "inspect", "--help"]);
    let canonical_plugins = run_help(&["cli", "plugins", "inspect", "--help"]);
    let alias_plugins_tail =
        alias_plugins.split_once("\n\n").map(|(_, tail)| tail).expect("help sections");
    let canonical_plugins_tail =
        canonical_plugins.split_once("\n\n").map(|(_, tail)| tail).expect("help sections");
    assert_eq!(alias_plugins_tail, canonical_plugins_tail);

    let alias_dev_doctor = run_help(&["dev", "doctor", "--help"]);
    let canonical_dev_doctor = run_help(&["dev", "cli", "doctor", "--help"]);
    let alias_dev_tail =
        alias_dev_doctor.split_once("\n\n").map(|(_, tail)| tail).expect("help sections");
    let canonical_dev_tail =
        canonical_dev_doctor.split_once("\n\n").map(|(_, tail)| tail).expect("help sections");
    assert_eq!(alias_dev_tail, canonical_dev_tail);
}

#[test]
fn help_no_color_and_wrapped_width_are_stable() {
    let no_color = run_help(&["--color", "never", "--help"]);
    assert!(!no_color.contains("\u{1b}["));

    let wrapped = run_help_with_env(&["--help"], &[("COLUMNS", "50")]);
    assert!(wrapped.contains("Commands:"));
    assert!(!wrapped.contains("\u{1b}["));
}

#[test]
fn help_rendering_stays_within_budget() {
    let start = Instant::now();
    let _ = run_help(&["--help"]);
    let elapsed = start.elapsed();
    assert!(
        elapsed < Duration::from_millis(1500),
        "root help render budget exceeded: {:?}",
        elapsed
    );
}
