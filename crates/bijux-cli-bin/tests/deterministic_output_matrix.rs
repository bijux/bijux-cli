#![forbid(unsafe_code)]
//! Deterministic output matrix coverage.
//! test_type: deterministic-repeated-run

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_install as _;
use bijux_cli_output as _;
use bijux_cli_routing as _;
use shlex as _;
use thiserror as _;
use bijux_cli_repl as _;
use libc as _;
use serde_json::Value;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .output()
        .expect("binary should execute")
}

fn run_with_env(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bijux-deterministic-output-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn setup_plugin(root: &Path, plugins_dir: &Path, namespace: &str) {
    let scaffold_dir = root.join(format!("{namespace}_scaffold"));
    let scaffold = run_with_env(
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
    assert!(scaffold.status.success());

    let manifest = scaffold_dir.join("plugin.manifest.json");
    let install = run_with_env(
        &["cli", "plugins", "install", manifest.to_str().expect("utf-8")],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))],
    );
    assert!(install.status.success());
}

#[test]
fn status_json_is_byte_stable_across_runs() {
    let first = run(&["status", "--format", "json", "--no-pretty"]);
    let second = run(&["status", "--format", "json", "--no-pretty"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn plugins_list_json_is_byte_stable_across_runs() {
    let root = temp_dir("todo-122");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_plugin(&root, &plugins_dir, "stablelist");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let first = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn config_get_json_is_byte_stable_across_runs() {
    let root = temp_dir("todo-123");
    let config_path = root.join("config.env");
    fs::write(&config_path, "BIJUXCLI_ALPHA=stable\n").expect("write config");

    let args = [
        "cli",
        "config",
        "get",
        "alpha",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config_path.to_str().expect("utf-8"),
    ];
    let first = run(&args);
    let second = run(&args);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn inspect_json_is_byte_stable_across_runs() {
    let first = run(&["inspect", "--format", "json", "--no-pretty"]);
    let second = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn json_envelope_field_order_is_stable() {
    let mut baseline: Option<Vec<u8>> = None;
    for _ in 0..6 {
        let out = run(&["status", "--format", "json", "--no-pretty"]);
        assert_eq!(out.status.code(), Some(0));
        let body = String::from_utf8(out.stdout.clone()).expect("utf-8");
        let runtime = body.find("\"runtime\"").expect("runtime key");
        let status = body.find("\"status\"").expect("status key");
        assert!(runtime < status);
        if let Some(prev) = &baseline {
            assert_eq!(prev, &out.stdout, "repeated run changed json order or payload");
        } else {
            baseline = Some(out.stdout);
        }
    }
}

#[test]
fn yaml_envelope_field_order_is_stable() {
    let first = run(&["inspect", "--format", "yaml", "--pretty"]);
    let second = run(&["inspect", "--format", "yaml", "--pretty"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn plugin_list_machine_output_order_is_stable() {
    let root = temp_dir("todo-128");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_plugin(&root, &plugins_dir, "zetaorder");
    setup_plugin(&root, &plugins_dir, "alphaorder");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let out = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("json");
    let names: Vec<&str> = payload["plugins"]
        .as_array()
        .expect("plugins")
        .iter()
        .filter_map(|row| row["manifest"]["namespace"].as_str())
        .collect();
    assert_eq!(names, vec!["alphaorder", "zetaorder"]);
}

#[test]
fn diagnostic_ordering_is_stable_in_machine_output() {
    let first = run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]);
    let second = run(&["dev", "cli", "doctor", "--format", "json", "--no-pretty"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn state_doctor_ordering_is_stable_in_machine_output() {
    let root = temp_dir("todo-130");
    let home = root.join("home");
    fs::create_dir_all(&home).expect("mkdir home");
    let home_str = home.to_str().expect("utf-8");

    let envs = [("HOME", home_str)];
    let first = run_with_env(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"], &envs);
    let second = run_with_env(&["dev", "cli", "state-doctor", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn repeated_runs_do_not_introduce_timestamp_noise_when_disallowed() {
    let first = run(&["status", "--format", "json", "--no-pretty"]);
    let second = run(&["status", "--format", "json", "--no-pretty"]);
    let first_text = String::from_utf8(first.stdout).expect("utf-8");
    let second_text = String::from_utf8(second.stdout).expect("utf-8");
    assert_eq!(first_text, second_text);
    assert!(!first_text.contains("timestamp"));
}

#[test]
fn repeated_runs_do_not_introduce_path_order_noise() {
    let first = run(&["inspect", "--format", "json", "--no-pretty"]);
    let second = run(&["inspect", "--format", "json", "--no-pretty"]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn repeated_runs_do_not_introduce_plugin_discovery_order_noise() {
    let root = temp_dir("todo-133");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_plugin(&root, &plugins_dir, "omegaorder");
    setup_plugin(&root, &plugins_dir, "betaorder");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let first = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run_with_env(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn repeated_runs_do_not_introduce_environment_order_noise() {
    let first = run_with_env(
        &["status", "--format", "json", "--no-pretty"],
        &[("ZZZ_IGNORED", "1"), ("AAA_IGNORED", "2")],
    );
    let second = run_with_env(
        &["status", "--format", "json", "--no-pretty"],
        &[("AAA_IGNORED", "2"), ("ZZZ_IGNORED", "1")],
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn text_output_stability_holds_under_no_color_mode() {
    let first = run_with_env(&["help", "cli", "plugins"], &[("NO_COLOR", "1")]);
    let second = run_with_env(&["help", "cli", "plugins"], &[("NO_COLOR", "1")]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
    let text = String::from_utf8(first.stdout).expect("utf-8");
    assert!(!text.contains("\u{1b}["));
}

#[test]
fn stderr_payloads_are_stable_for_identical_failures() {
    let first = run(&["--format", "invalid", "cli", "status"]);
    let second = run(&["--format", "invalid", "cli", "status"]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn exit_codes_are_stable_for_identical_failures() {
    let first = run(&["--color", "invalid", "cli", "status"]);
    let second = run(&["--color", "invalid", "cli", "status"]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
}
