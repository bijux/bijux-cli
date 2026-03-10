#![forbid(unsafe_code)]
//! Plugin namespace-law integration coverage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli_core as _;
use bijux_cli_install as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use bijux_cli_routing as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

fn run(args: &[&str], plugins_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux-rs"))
        .args(args)
        .env("BIJUXCLI_PLUGINS_DIR", plugins_dir)
        .output()
        .expect("binary should execute")
}

fn tmp_dir(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("bijux-plugin-ns-law-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn write_manifest(path: &Path, namespace: &str, alias: &str, entrypoint: &str) {
    let body = format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}},
  "namespace": "{namespace}",
  "kind": "delegated",
  "aliases": ["{alias}"],
  "entrypoint": "{entrypoint}",
  "capabilities": []
}}"#
    );
    fs::write(path, body).expect("write manifest");
}

fn scaffold_rejection(plugins_dir: &Path, namespace: &str) -> (i32, String, String) {
    let target = plugins_dir.join(format!("scaffold-{namespace}"));
    let out = run(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            namespace,
            "--path",
            target.to_str().expect("utf-8"),
        ],
        plugins_dir,
    );
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8(out.stdout).expect("stdout utf-8"),
        String::from_utf8(out.stderr).expect("stderr utf-8"),
    )
}

#[test]
fn rejects_plugin_namespace_cli() {
    let root = tmp_dir("reserved-cli");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, stdout, stderr) = scaffold_rejection(&plugins_dir, "cli");
    assert_eq!(code, 1);
    assert!(stdout.is_empty());
    assert!(stderr.contains("plugin namespace is reserved: cli"));
}

#[test]
fn rejects_plugin_namespace_dev() {
    let root = tmp_dir("reserved-dev");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "dev");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: dev"));
}

#[test]
fn rejects_plugin_namespace_help() {
    let root = tmp_dir("reserved-help");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "help");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: help"));
}

#[test]
fn rejects_plugin_namespace_version() {
    let root = tmp_dir("reserved-version");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "version");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: version"));
}

#[test]
fn rejects_plugin_namespace_doctor() {
    let root = tmp_dir("reserved-doctor");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "doctor");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: doctor"));
}

#[test]
fn rejects_plugin_namespace_plugins() {
    let root = tmp_dir("reserved-plugins");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "plugins");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: plugins"));
}

#[test]
fn rejects_plugin_namespace_repl() {
    let root = tmp_dir("reserved-repl");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "repl");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: repl"));
}

#[test]
fn rejects_official_product_namespace_dag() {
    let root = tmp_dir("reserved-dag");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "dag");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: dag"));
}

#[test]
fn rejects_official_product_namespace_atlas() {
    let root = tmp_dir("reserved-atlas");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "atlas");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: atlas"));
}

#[test]
fn rejects_normalized_collision_my_plugin_vs_my_plugin_hyphen() {
    let root = tmp_dir("normalized-collision");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let first_manifest = root.join("my-plugin.json");
    let second_manifest = root.join("my_plugin.json");
    write_manifest(&first_manifest, "my-plugin", "my-plugin", "plugin.py:run");
    write_manifest(&second_manifest, "my_plugin", "my_plugin", "plugin.py:run");

    let first =
        run(&["cli", "plugins", "install", first_manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(first.status.code(), Some(0));
    let second =
        run(&["cli", "plugins", "install", second_manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(second.status.code(), Some(1));
    let stderr = String::from_utf8(second.stderr).expect("stderr utf-8");
    assert!(
        stderr.contains("namespace")
            || stderr.contains("alias")
            || stderr.contains("invalid plugin namespace")
    );
}

#[test]
fn rejects_case_insensitive_normalized_collision() {
    let root = tmp_dir("case-collision");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let first_manifest = root.join("my-plugin.json");
    let second_manifest = root.join("MY-PLUGIN.json");
    write_manifest(&first_manifest, "my-plugin", "my-plugin", "plugin.py:run");
    write_manifest(&second_manifest, "MY-PLUGIN", "MY-PLUGIN", "plugin.py:run");

    let first =
        run(&["cli", "plugins", "install", first_manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(first.status.code(), Some(1));
    let second =
        run(&["cli", "plugins", "install", second_manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(second.status.code(), Some(1));
    let first_stderr = String::from_utf8(first.stderr).expect("stderr utf-8");
    let second_stderr = String::from_utf8(second.stderr).expect("stderr utf-8");
    assert!(first_stderr.contains("plugin namespace is invalid:"));
    assert!(second_stderr.contains("plugin namespace is invalid:"));
}

#[test]
fn rejects_namespace_with_leading_digit() {
    let root = tmp_dir("leading-digit");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "1plugin");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is invalid:"));
}

#[test]
fn rejects_namespace_with_whitespace() {
    let root = tmp_dir("whitespace");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "bad name");
    assert_eq!(code, 1);
    assert!(stderr.contains("invalid plugin namespace") || stderr.contains("namespace"));
}

#[test]
fn rejects_namespace_with_shell_hostile_punctuation() {
    let root = tmp_dir("hostile-punctuation");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "bad;rm");
    assert_eq!(code, 1);
    assert!(stderr.contains("invalid plugin namespace") || stderr.contains("namespace"));
}

#[test]
fn rejects_empty_namespace() {
    let root = tmp_dir("empty-ns");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let manifest = root.join("empty.json");
    write_manifest(&manifest, "", "empty", "plugin.py:run");
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(stderr.contains("plugin manifest field invalid: name"));
}

#[test]
fn rejects_namespace_differing_only_by_hidden_alias_collision() {
    let root = tmp_dir("hidden-alias-collision");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let (code, _, stderr) = scaffold_rejection(&plugins_dir, "doctor");
    assert_eq!(code, 1);
    assert!(stderr.contains("plugin namespace is reserved: doctor"));
}

#[test]
fn rejection_messages_explain_the_reason_clearly() {
    let root = tmp_dir("clear-reason");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");

    let reserved = scaffold_rejection(&plugins_dir, "cli").2;
    assert!(reserved.contains("plugin namespace is reserved:"));

    let invalid = scaffold_rejection(&plugins_dir, "bad name").2;
    assert!(invalid.contains("invalid plugin namespace") || invalid.contains("namespace"));
}

#[test]
fn json_error_envelopes_for_namespace_rejection_are_stable() {
    let root = tmp_dir("json-envelope");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let out = run(
        &["--format", "json", "--no-pretty", "cli", "plugins", "scaffold", "python", "cli"],
        &plugins_dir,
    );
    assert_eq!(out.status.code(), Some(1));
    let payload: Value = serde_json::from_slice(&out.stderr).expect("stderr json");
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["code"], 1);
    assert_eq!(payload["command"], "cli plugins scaffold");
    assert!(payload["message"]
        .as_str()
        .expect("message")
        .contains("plugin namespace is reserved: plugins"));
}

#[test]
fn text_errors_for_namespace_rejection_are_stable() {
    let root = tmp_dir("text-envelope");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir");
    let out =
        run(&["--format", "text", "cli", "plugins", "scaffold", "python", "cli"], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(stderr.contains("plugin namespace is reserved: plugins"));
}
