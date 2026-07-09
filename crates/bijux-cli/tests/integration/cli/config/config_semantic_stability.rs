#![forbid(unsafe_code)]
//! Config semantic stability checks for parsing, mutation, roundtrip, and diagnostics consistency.
//! test_type: config-semantic-stability

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bijux")).args(args).output().expect("binary should execute")
}

fn run_json(args: &[&str]) -> Value {
    let out = run(args);
    assert_eq!(out.status.code(), Some(0), "expected success for {args:?}");
    assert!(out.stderr.is_empty(), "successful command must keep stderr empty: {args:?}");
    assert!(!out.stdout.is_empty(), "successful command must emit stdout payload: {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout should be valid json")
}

fn temp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir()
            .join(format!("bijux-config-semantic-stability-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir");
    root
}

#[test]
fn config_key_normalization_and_parse_behavior_are_stable_across_repeated_inputs() {
    let root = temp_dir("normalization");
    let config = root.join("config.env");
    fs::write(
        &config,
        "BIJUXCLI_ALPHA=1\r\nBIJUXCLI_BETA = two  \n# comment line\n\nBIJUXCLI_GAMMA=three\n",
    )
    .expect("write config");
    let path = config.to_string_lossy().to_string();

    let first = run_json(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    let second = run_json(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(first, second, "repeated parse should be deterministic");
    assert_eq!(first["alpha"], "1");
    assert_eq!(first["beta"], "two");
    assert_eq!(first["gamma"], "three");
    assert!(first.get("# comment line").is_none(), "comments should not become config keys");
}

#[test]
fn config_writer_ordering_and_formatting_rules_are_deterministic() {
    let root = temp_dir("writer-order");
    let config = root.join("config.env");
    let path = config.to_string_lossy().to_string();

    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_Z=9", "--config-path", &path]).status.code(),
        Some(0)
    );
    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_A=1", "--config-path", &path]).status.code(),
        Some(0)
    );
    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_M=5", "--config-path", &path]).status.code(),
        Some(0)
    );

    let written = fs::read_to_string(&config).expect("read written config");
    let lines = written
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    assert_eq!(
        lines,
        vec!["BIJUXCLI_A=1".to_string(), "BIJUXCLI_M=5".to_string(), "BIJUXCLI_Z=9".to_string()],
        "writer ordering should be deterministic and normalized"
    );
    assert!(!written.contains("#"), "writer must not reintroduce dropped comments");
}

#[test]
fn config_export_and_load_preserve_semantic_content_and_roundtrip_exact_values() {
    let root = temp_dir("export-load");
    let source = root.join("source.env");
    let target = root.join("target.env");
    let exported = root.join("exported.env");
    let source_path = source.to_string_lossy().to_string();
    let target_path = target.to_string_lossy().to_string();
    let exported_path = exported.to_string_lossy().to_string();

    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_ALPHA=hello world", "--config-path", &source_path])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        run(&[
            "cli",
            "config",
            "set",
            "BIJUXCLI_QUOTED='quoted value'",
            "--config-path",
            &source_path
        ])
        .status
        .code(),
        Some(0)
    );

    let json_export = run_json(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &source_path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    let yaml_out = run(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &source_path,
        "--format",
        "yaml",
        "--pretty",
    ]);
    let text_out =
        run(&["cli", "config", "list", "--config-path", &source_path, "--format", "text"]);
    assert_eq!(yaml_out.status.code(), Some(0));
    assert_eq!(text_out.status.code(), Some(0));

    assert_eq!(
        run(&["cli", "config", "export", &exported_path, "--config-path", &source_path])
            .status
            .code(),
        Some(0)
    );
    assert_eq!(
        run(&["cli", "config", "load", &exported_path, "--config-path", &target_path])
            .status
            .code(),
        Some(0)
    );
    let loaded_json = run_json(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &target_path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(json_export, loaded_json, "load+export should preserve semantic content");

    let roundtrip_get = run_json(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--config-path",
        &source_path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(roundtrip_get["value"], "hello world");
}

#[test]
fn config_unset_clear_and_repeated_mutations_follow_expected_semantics() {
    let root = temp_dir("unset-clear");
    let config = root.join("config.env");
    let path = config.to_string_lossy().to_string();

    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_ALPHA=1", "--config-path", &path]).status.code(),
        Some(0)
    );
    assert_eq!(
        run(&["cli", "config", "set", "BIJUXCLI_BETA=2", "--config-path", &path]).status.code(),
        Some(0)
    );
    assert_eq!(
        run(&["cli", "config", "unset", "alpha", "--config-path", &path]).status.code(),
        Some(0)
    );
    let missing = run(&[
        "cli",
        "config",
        "get",
        "alpha",
        "--config-path",
        &path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(missing.status.code(), Some(2));

    assert_eq!(run(&["cli", "config", "clear", "--config-path", &path]).status.code(), Some(0));
    let listed = run_json(&[
        "cli",
        "config",
        "list",
        "--config-path",
        &path,
        "--format",
        "json",
        "--no-pretty",
    ]);
    assert_eq!(listed, serde_json::json!({}), "clear should remove all managed keys");

    let first = run(&["cli", "config", "set", "BIJUXCLI_ALPHA=1", "--config-path", &path]);
    let second = run(&["cli", "config", "set", "BIJUXCLI_ALPHA=1", "--config-path", &path]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    let after_first = fs::read_to_string(&config).expect("read after first");
    let after_second = fs::read_to_string(&config).expect("read after second");
    assert_eq!(after_first, after_second, "same-input repeated mutations should be deterministic");
}

#[test]
fn root_and_cli_config_path_override_behavior_is_identical_for_list() {
    let root = temp_dir("path-override");
    let config = root.join("config.env");
    let path = config.to_string_lossy().to_string();
    fs::write(&config, "BIJUXCLI_SHARED=from-file\nBIJUXCLI_ANOTHER=x\n").expect("write config");

    let root_list =
        run(&["config", "list", "--config-path", &path, "--format", "json", "--no-pretty"]);
    let cli_list =
        run(&["cli", "config", "list", "--config-path", &path, "--format", "json", "--no-pretty"]);

    assert_eq!(root_list.status.code(), Some(0));
    assert_eq!(cli_list.status.code(), Some(0));
    assert_eq!(root_list.stdout, cli_list.stdout);
    assert_eq!(root_list.stderr, cli_list.stderr);
}
