#![forbid(unsafe_code)]
//! Determinism checks under hostile state conditions.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!("bijux-hostile-determinism-{name}-{}-{counter}", std::process::id(),));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn write_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\n").expect("write binary");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(path).expect("metadata").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(path, perms).expect("chmod +x");
    }
}

fn setup_python_plugin(root: &Path, plugins_dir: &Path, namespace: &str) {
    let scaffold_dir = root.join(format!("{namespace}_scaffold"));
    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];

    let scaffold = run(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            namespace,
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert!(scaffold.status.success());

    let install = run(
        &[
            "cli",
            "plugins",
            "install",
            scaffold_dir.join("plugin.manifest.json").to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert!(install.status.success());
}

fn setup_external_plugin(root: &Path, plugins_dir: &Path, namespace: &str, entrypoint: &Path) {
    let manifest = root.join(format!("{namespace}.manifest.json"));
    fs::write(
        &manifest,
        format!(
            r#"{{
  "name": "{namespace}",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"0.1.0", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "external-exec",
  "trust_class": "community",
  "aliases": [],
  "entrypoint": "{}",
  "capabilities": []
}}"#,
            entrypoint.to_string_lossy()
        ),
    )
    .expect("write manifest");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let install = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &envs);
    assert!(install.status.success());
}

#[test]
fn corrupted_config_failure_class_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let config = root.join("broken.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write broken config");
    let args = ["cli", "config", "reload", "--config-path", config.to_str().expect("utf-8")];

    let first = run(&args, &[]);
    let second = run(&args, &[]);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn corrupted_plugin_registry_failure_class_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write broken registry");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let first = run(&["cli", "plugins", "disable", "missing"], &envs);
    let second = run(&["cli", "plugins", "disable", "missing"], &envs);

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn broken_history_file_recovery_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let history = root.join("broken.history");
    fs::write(&history, "{oops:true}").expect("write broken history");
    let envs = [("BIJUXCLI_HISTORY_FILE", history.to_str().expect("utf-8"))];

    let first = run(&["history", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["history", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert!(first.stdout.is_empty());
    assert!(second.stdout.is_empty());
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn malformed_memory_state_recovery_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    fs::write(&memory, "{broken").expect("write broken memory");
    let envs = [("HOME", home.to_str().expect("utf-8"))];

    let first = run(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["memory", "list", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert!(first.stdout.is_empty());
    assert!(second.stdout.is_empty());
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn missing_config_file_defaulting_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let missing = root.join("missing.env");
    let args = ["cli", "config", "reload", "--config-path", missing.to_str().expect("utf-8")];

    let first = run(&args, &[]);
    let second = run(&args, &[]);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn missing_plugin_directory_empty_behavior_is_stable_across_runs() {
    let root = temp_dir("hostile-state");
    let missing_plugins = root.join("missing-plugins");
    let envs = [("BIJUXCLI_PLUGINS_DIR", missing_plugins.to_str().expect("utf-8"))];

    let first = run(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn broken_plugin_does_not_nondeterministically_affect_healthy_output() {
    let root = temp_dir("hostile-state");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_python_plugin(&root, &plugins_dir, "healthyplug");
    let entrypoint = root.join("broken-entry.sh");
    write_executable(&entrypoint);
    setup_external_plugin(&root, &plugins_dir, "brokenplug", &entrypoint);
    fs::remove_file(&entrypoint).expect("remove broken entrypoint");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let first = run(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["cli", "plugins", "list", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);

    let payload: Value = serde_json::from_slice(&first.stdout).expect("json");
    let names: Vec<&str> = payload["plugins"]
        .as_array()
        .expect("plugins")
        .iter()
        .filter_map(|item| item["manifest"]["namespace"].as_str())
        .collect();
    assert!(names.contains(&"healthyplug"));
}

#[test]
fn conflicting_plugin_installs_fail_deterministically() {
    let root = temp_dir("hostile-state");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_python_plugin(&root, &plugins_dir, "conflictplug");

    let manifest = root.join("conflictplug_scaffold").join("plugin.manifest.json");
    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];

    let first = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &envs);
    let second = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &envs);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn plugin_doctor_json_is_stable_under_same_corrupted_state() {
    let root_a = temp_dir("hostile-state-a");
    let plugins_a = root_a.join("plugins");
    fs::create_dir_all(&plugins_a).expect("mkdir plugins a");
    fs::write(plugins_a.join("registry.json"), "{broken-json").expect("write broken a");

    let root_b = temp_dir("hostile-state-b");
    let plugins_b = root_b.join("plugins");
    fs::create_dir_all(&plugins_b).expect("mkdir plugins b");
    fs::write(plugins_b.join("registry.json"), "{broken-json").expect("write broken b");

    let a = run(
        &["cli", "plugins", "doctor", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_a.to_str().expect("utf-8"))],
    );
    let b = run(
        &["cli", "plugins", "doctor", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_b.to_str().expect("utf-8"))],
    );
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(b.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn plugin_doctor_text_is_stable_under_same_corrupted_state() {
    let root_a = temp_dir("hostile-state-a");
    let plugins_a = root_a.join("plugins");
    fs::create_dir_all(&plugins_a).expect("mkdir plugins a");
    fs::write(plugins_a.join("registry.json"), "{broken-json").expect("write broken a");

    let root_b = temp_dir("hostile-state-b");
    let plugins_b = root_b.join("plugins");
    fs::create_dir_all(&plugins_b).expect("mkdir plugins b");
    fs::write(plugins_b.join("registry.json"), "{broken-json").expect("write broken b");

    let a = run(
        &["cli", "plugins", "doctor", "--format", "text"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_a.to_str().expect("utf-8"))],
    );
    let b = run(
        &["cli", "plugins", "doctor", "--format", "text"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_b.to_str().expect("utf-8"))],
    );
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(b.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
}

#[test]
fn command_tree_export_is_stable_with_broken_optional_state() {
    let root = temp_dir("hostile-state");
    let history = root.join("broken.history");
    fs::write(&history, "{oops:true}").expect("write broken history");
    let plugins = root.join("plugins");
    fs::create_dir_all(&plugins).expect("mkdir plugins");
    fs::write(plugins.join("registry.json"), "{broken-json").expect("write broken registry");

    let envs = [
        ("BIJUXCLI_HISTORY_FILE", history.to_str().expect("utf-8")),
        ("BIJUXCLI_PLUGINS_DIR", plugins.to_str().expect("utf-8")),
    ];

    let first = run(&["inspect", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["inspect", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}
