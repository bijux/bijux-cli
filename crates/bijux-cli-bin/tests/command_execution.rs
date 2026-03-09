#![forbid(unsafe_code)]
//! Integration coverage for implemented built-in and developer commands.

use std::process::Command;

use anyhow as _;
use bijux_cli_contracts as _;
use bijux_cli_core as _;
use bijux_cli_output as _;
use bijux_cli_python as _;
use bijux_cli_routing as _;
use clap as _;
use serde_json as _;

fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute");
    assert!(output.status.success(), "process failed for args: {args:?}");
    String::from_utf8(output.stdout).expect("stdout should be UTF-8")
}

#[test]
fn executes_root_commands() {
    for args in [
        vec!["version"],
        vec!["doctor"],
        vec!["repl"],
        vec!["completion"],
        vec!["inspect"],
    ] {
        let stdout = run(&args);
        assert!(stdout.contains('{'), "expected structured payload for {args:?}");
    }
}

#[test]
fn executes_cli_namespace_commands() {
    for args in [
        vec!["cli", "status"],
        vec!["cli", "paths"],
        vec!["cli", "config", "get"],
        vec!["cli", "config", "set"],
        vec!["cli", "self-test"],
        vec!["cli", "plugins", "list"],
        vec!["cli", "plugins", "inspect"],
    ] {
        let stdout = run(&args);
        assert!(stdout.contains('{'), "expected structured payload for {args:?}");
    }
}

#[test]
fn executes_dev_cli_namespace_commands() {
    for args in [
        vec!["dev", "cli", "routes"],
        vec!["dev", "cli", "registry"],
        vec!["dev", "cli", "env"],
        vec!["dev", "cli", "doctor"],
        vec!["dev", "cli", "contracts"],
    ] {
        let stdout = run(&args);
        assert!(stdout.contains('{'), "expected structured payload for {args:?}");
    }
}
