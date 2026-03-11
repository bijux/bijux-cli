#![forbid(unsafe_code)]
//! Help tree law coverage for TODOs 341-355.
//! test_type: help-law

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
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

fn json(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout json")
}

fn temp_dir(name: &str) -> PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let root =
        std::env::temp_dir().join(format!("bijux-help-law-{name}-{}-{nanos}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn parse_help_commands(help: &str) -> Vec<String> {
    let mut in_commands = false;
    let mut names = Vec::new();
    for line in help.lines() {
        let trimmed = line.trim_end();
        if trimmed == "Commands:" {
            in_commands = true;
            continue;
        }
        if !in_commands {
            continue;
        }
        if trimmed.is_empty() {
            continue;
        }
        if !line.starts_with("  ") {
            break;
        }
        let first = trimmed.split_whitespace().next().unwrap_or_default();
        if !first.is_empty() {
            names.push(first.to_string());
        }
    }
    names
}

fn parse_error_payload(out: &Output) -> Value {
    if !out.stdout.is_empty() {
        return serde_json::from_slice(&out.stdout).expect("stdout error json");
    }
    serde_json::from_slice(&out.stderr).expect("stderr error json")
}

fn scaffold_manifest(root: &Path, plugins_dir: &Path, namespace: &str) -> PathBuf {
    let scaffold_dir = root.join(format!("{namespace}_scaffold"));
    let out = run_with_env(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            namespace,
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0), "scaffold should succeed");
    scaffold_dir.join("plugin.manifest.json")
}

fn install_plugin(root: &Path, plugins_dir: &Path, namespace: &str) {
    let manifest = scaffold_manifest(root, plugins_dir, namespace);
    let out = run_with_env(
        &["cli", "plugins", "install", manifest.to_str().expect("utf-8")],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))],
    );
    assert_eq!(out.status.code(), Some(0), "plugin install should succeed");
}

#[test]
fn root_help_lists_commands_in_stable_order() {
    let a = run(&["--help"]);
    let b = run(&["--help"]);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(b.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
    let commands = parse_help_commands(&String::from_utf8(a.stdout).expect("utf-8"));
    assert!(commands.starts_with(&["cli".into(), "dev".into(), "status".into()]));
}

#[test]
fn cli_help_lists_subcommands_in_stable_order() {
    let a = run(&["cli", "--help"]);
    let b = run(&["cli", "--help"]);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
    let commands = parse_help_commands(&String::from_utf8(a.stdout).expect("utf-8"));
    assert_eq!(commands, vec!["status", "paths", "config", "self-test", "plugins", "help"]);
}

#[test]
fn dev_cli_help_lists_subcommands_in_stable_order() {
    let a = run(&["dev", "cli", "--help"]);
    let b = run(&["dev", "cli", "--help"]);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
    let commands = parse_help_commands(&String::from_utf8(a.stdout).expect("utf-8"));
    assert!(commands.starts_with(&[
        "scripts".into(),
        "rustdoc".into(),
        "release".into(),
        "evidence".into()
    ]));
    assert!(commands.contains(&"env".to_string()));
    assert!(commands.contains(&"runtime-identity".to_string()));
}

#[test]
fn plugin_installed_help_keeps_builtin_order_stable() {
    let root = temp_dir("plugin-help-order");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let before = run_with_env(
        &["--help"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))],
    );
    assert_eq!(before.status.code(), Some(0));
    install_plugin(&root, &plugins_dir, "helporderplug");

    let after = run_with_env(
        &["--help"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))],
    );
    assert_eq!(after.status.code(), Some(0));

    let before_cmds = parse_help_commands(&String::from_utf8(before.stdout).expect("utf-8"));
    let after_cmds = parse_help_commands(&String::from_utf8(after.stdout).expect("utf-8"));

    let before_builtin: Vec<&str> =
        before_cmds.iter().map(String::as_str).filter(|s| *s != "help").collect();
    let after_builtin: Vec<&str> =
        after_cmds.iter().map(String::as_str).filter(|s| *s != "help").collect();
    for window in before_builtin.windows(2) {
        let left = window[0];
        let right = window[1];
        let left_pos = after_builtin.iter().position(|item| *item == left).expect("left present");
        let right_pos =
            after_builtin.iter().position(|item| *item == right).expect("right present");
        assert!(left_pos < right_pos, "built-in order changed between {left} and {right}");
    }
}

#[test]
fn no_color_root_help_and_grouped_help_are_stable() {
    let root_help = run(&["--color", "never", "--help"]);
    assert_eq!(root_help.status.code(), Some(0));
    let root_text = String::from_utf8(root_help.stdout).expect("utf-8");
    assert!(!root_text.contains("\u{1b}["));

    let group_help = run(&["cli", "plugins", "--color", "never", "--help"]);
    assert_eq!(group_help.status.code(), Some(0));
    let group_text = String::from_utf8(group_help.stdout).expect("utf-8");
    assert!(!group_text.contains("\u{1b}["));
}

#[test]
fn unknown_command_suggestions_are_deterministic_and_namespace_scoped() {
    let first = run(&["sttaus", "--format", "json", "--no-pretty"]);
    let second = run(&["sttaus", "--format", "json", "--no-pretty"]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(first.stdout, second.stdout);

    let payload = parse_error_payload(&first);
    assert_eq!(payload["command"], "sttaus");
    assert_eq!(payload["message"], "unknown route: sttaus");

    let namespaced = run(&["devv", "cli", "routes", "--format", "json", "--no-pretty"]);
    assert_eq!(namespaced.status.code(), Some(1));
    let err = parse_error_payload(&namespaced);
    assert_eq!(err["message"], "unknown route: devv");
}

#[test]
fn hidden_aliases_do_not_appear_as_canonical_help_entries() {
    let root_help = run(&["--help"]);
    assert_eq!(root_help.status.code(), Some(0));
    let root_commands = parse_help_commands(&String::from_utf8(root_help.stdout).expect("utf-8"));

    let dev_help = run(&["dev", "--help"]);
    assert_eq!(dev_help.status.code(), Some(0));
    let dev_commands = parse_help_commands(&String::from_utf8(dev_help.stdout).expect("utf-8"));

    assert!(root_commands.contains(&"dev".to_string()));
    assert!(dev_commands.contains(&"cli".to_string()));
    assert!(!dev_commands.contains(&"doctor".to_string()));
    assert!(!dev_commands.contains(&"routes".to_string()));
}

#[test]
fn inspect_metadata_agrees_with_help_names_and_command_tree_export() {
    let inspect = json(&["inspect", "--format", "json", "--no-pretty"]);
    let routes = json(&["dev", "cli", "routes", "--format", "json", "--no-pretty"]);
    let root_help = run(&["--help"]);
    assert_eq!(root_help.status.code(), Some(0));
    let help_commands: BTreeSet<String> =
        parse_help_commands(&String::from_utf8(root_help.stdout).expect("utf-8"))
            .into_iter()
            .collect();

    let inspect_roots: BTreeSet<String> = inspect["route_sources"]
        .as_array()
        .expect("route_sources")
        .iter()
        .filter_map(|row| {
            row["segments"].as_array().and_then(|segments| segments.first()).and_then(Value::as_str)
        })
        .map(ToString::to_string)
        .collect();

    let route_roots: BTreeSet<String> = routes["routes"]
        .as_array()
        .expect("routes")
        .iter()
        .filter_map(|row| {
            row["segments"].as_array().and_then(|segments| segments.first()).and_then(Value::as_str)
        })
        .map(ToString::to_string)
        .collect();

    assert_eq!(inspect_roots, route_roots);
    for name in ["cli", "dev", "status", "config", "plugins", "history", "memory"] {
        assert!(help_commands.contains(name));
        assert!(inspect_roots.contains(name));
    }
}

#[test]
fn help_under_broken_plugin_registry_and_corrupted_state_is_stable_and_useful() {
    let root = temp_dir("broken-help");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write broken registry");

    let config = root.join("broken.env");
    fs::write(&config, "BIJUXCLI_ALPHA=1\nBROKEN\n").expect("write broken config");

    let envs = [
        ("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8")),
        ("BIJUXCLI_CONFIG", config.to_str().expect("utf-8")),
    ];

    let first = run_with_env(&["--help"], &envs);
    let second = run_with_env(&["--help"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    let help_text = String::from_utf8(first.stdout).expect("utf-8");
    assert!(help_text.contains("Usage:"));
    assert!(help_text.contains("Commands:"));
}

#[test]
fn command_tree_is_stable_across_repeated_plugin_discovery_runs() {
    let root = temp_dir("repeated-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_plugin(&root, &plugins_dir, "discoverystable");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let first = run_with_env(&["dev", "cli", "routes", "--format", "json", "--no-pretty"], &envs);
    let second = run_with_env(&["dev", "cli", "routes", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);

    let help_a = run_with_env(&["--help"], &envs);
    let help_b = run_with_env(&["--help"], &envs);
    assert_eq!(help_a.status.code(), Some(0));
    assert_eq!(help_a.stdout, help_b.stdout);
}
