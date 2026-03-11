#![forbid(unsafe_code)]
//! Invariants for the `dev cli` maintainer command family.

use std::collections::BTreeSet;
use std::process::Command;

use serde_json::Value;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs")).args(args).output().expect("binary should execute")
}

fn dev_cli_commands() -> Vec<Vec<String>> {
    include_str!("../../../data/fixtures/routing/dev_cli_subcommands.txt")
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.split_whitespace().map(ToString::to_string).collect())
        .collect()
}

fn to_refs(values: &[String]) -> Vec<&str> {
    values.iter().map(String::as_str).collect()
}

#[test]
fn dev_cli_command_registry_has_stable_unique_names() {
    let commands = dev_cli_commands();
    let set: BTreeSet<Vec<String>> = commands.iter().cloned().collect();
    assert_eq!(
        set.len(),
        commands.len(),
        "dev cli fixture must not contain duplicate command names"
    );
}

#[test]
fn dev_cli_json_outputs_are_parseable_for_all_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--format".to_string());
        args.push("json".to_string());
        args.push("--no-pretty".to_string());
        let out = run(&to_refs(&args));
        assert!(out.status.success(), "json command failed for {:?}", args);
        let parsed: Value = serde_json::from_slice(&out.stdout).expect("valid json");
        assert!(parsed.is_object(), "dev cli output must be object for {:?}", args);
    }
}

#[test]
fn dev_cli_text_outputs_are_non_empty_for_all_commands() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--format".to_string());
        args.push("text".to_string());
        let out = run(&to_refs(&args));
        assert!(out.status.success(), "text command failed for {:?}", args);
        let text = String::from_utf8(out.stdout).expect("utf8");
        assert!(!text.trim().is_empty(), "empty text output for {:?}", args);
    }
}

#[test]
fn dev_cli_help_output_is_stable_across_repeated_runs() {
    for command in dev_cli_commands() {
        let mut args = command.clone();
        args.push("--help".to_string());
        let first = run(&to_refs(&args));
        let second = run(&to_refs(&args));
        assert!(first.status.success(), "first help run failed for {:?}", args);
        assert!(second.status.success(), "second help run failed for {:?}", args);
        assert_eq!(first.stdout, second.stdout, "help output drift for {:?}", args);
    }
}

#[test]
fn dev_cli_quiet_mode_keeps_exit_semantics_stable() {
    let base = run(&["dev", "cli", "status", "--format", "json", "--no-pretty"]);
    let quiet = run(&["dev", "cli", "status", "--format", "json", "--no-pretty", "--quiet"]);
    assert_eq!(base.status.code(), quiet.status.code(), "quiet mode changed exit semantics");
}
