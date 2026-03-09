#![forbid(unsafe_code)]
//! Help snapshot coverage for root, cli, and dev-cli command surfaces.

use std::process::Command;

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_plugin as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use clap as _;
use serde_json as _;

fn run_help(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "help command failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn help_snapshots_match_expected_output() {
    let cases: [(&[&str], &str); 18] = [
        (&["--help"], include_str!("snapshots/help_root.txt")),
        (&["version", "--help"], include_str!("snapshots/help_version.txt")),
        (&["doctor", "--help"], include_str!("snapshots/help_doctor.txt")),
        (&["repl", "--help"], include_str!("snapshots/help_repl.txt")),
        (&["completion", "--help"], include_str!("snapshots/help_completion.txt")),
        (&["inspect", "--help"], include_str!("snapshots/help_inspect.txt")),
        (&["cli", "status", "--help"], include_str!("snapshots/help_cli_status.txt")),
        (&["cli", "paths", "--help"], include_str!("snapshots/help_cli_paths.txt")),
        (
            &["cli", "config", "get", "--help"],
            include_str!("snapshots/help_cli_config_get.txt"),
        ),
        (
            &["cli", "config", "set", "--help"],
            include_str!("snapshots/help_cli_config_set.txt"),
        ),
        (&["cli", "self-test", "--help"], include_str!("snapshots/help_cli_self_test.txt")),
        (
            &["cli", "plugins", "list", "--help"],
            include_str!("snapshots/help_cli_plugins_list.txt"),
        ),
        (
            &["cli", "plugins", "inspect", "--help"],
            include_str!("snapshots/help_cli_plugins_inspect.txt"),
        ),
        (&["dev", "cli", "routes", "--help"], include_str!("snapshots/help_dev_cli_routes.txt")),
        (
            &["dev", "cli", "registry", "--help"],
            include_str!("snapshots/help_dev_cli_registry.txt"),
        ),
        (&["dev", "cli", "env", "--help"], include_str!("snapshots/help_dev_cli_env.txt")),
        (&["dev", "cli", "doctor", "--help"], include_str!("snapshots/help_dev_cli_doctor.txt")),
        (
            &["dev", "cli", "contracts", "--help"],
            include_str!("snapshots/help_dev_cli_contracts.txt"),
        ),
    ];

    for (args, expected) in cases {
        let actual = run_help(args);
        assert_eq!(actual, expected, "help snapshot mismatch for args: {args:?}");
    }
}
