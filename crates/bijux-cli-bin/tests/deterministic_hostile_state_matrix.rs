#![forbid(unsafe_code)]
//! Deterministic behavior matrix under hostile state conditions.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli_core as _;
use bijux_cli_python as _;
use bijux_cli_repl as _;
use libc as _;
use serde_json::Value;

fn run(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_bijux-rs"));
    cmd.args(args);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    cmd.output().expect("binary should execute")
}

fn temp_dir(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("bijux-hostile-determinism-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
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
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {{"min_inclusive":"0.1.0", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "external-exec",
  "aliases": [],
  "entrypoint": "{}",
  "capabilities": []
}}"#,
            entrypoint.to_string_lossy()
        ),
    )
    .expect("write manifest");

    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];
    let install = run(
        &[
            "cli",
            "plugins",
            "install",
            manifest.to_str().expect("utf-8"),
        ],
        &envs,
    );
    assert!(install.status.success());
}

#[test]
fn corrupted_config_failure_class_is_stable_across_runs() {
    let root = temp_dir("todo-141");
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
    let root = temp_dir("todo-142");
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
    let root = temp_dir("todo-143");
    let history = root.join("broken.history");
    fs::write(&history, "{oops:true}").expect("write broken history");
    let envs = [("BIJUXCLI_HISTORY_FILE", history.to_str().expect("utf-8"))];

    let first = run(&["history", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["history", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn malformed_memory_state_recovery_is_stable_across_runs() {
    let root = temp_dir("todo-144");
    let home = root.join("home");
    let memory = home.join(".bijux").join(".memory.json");
    fs::create_dir_all(memory.parent().expect("parent")).expect("mkdir");
    fs::write(&memory, "{broken").expect("write broken memory");
    let envs = [("HOME", home.to_str().expect("utf-8"))];

    let first = run(&["memory", "list", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["memory", "list", "--format", "json", "--no-pretty"], &envs);

    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn missing_config_file_defaulting_is_stable_across_runs() {
    let root = temp_dir("todo-145");
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
    let root = temp_dir("todo-146");
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
    let root = temp_dir("todo-147");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_python_plugin(&root, &plugins_dir, "healthyplug");
    let missing_entry = root.join("missing-entry.sh");
    setup_external_plugin(&root, &plugins_dir, "brokenplug", &missing_entry);

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
    let root = temp_dir("todo-148");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    setup_python_plugin(&root, &plugins_dir, "conflictplug");

    let manifest = root.join("conflictplug_scaffold").join("plugin.manifest.json");
    let envs = [("BIJUXCLI_PLUGINS_DIR", plugins_dir.to_str().expect("utf-8"))];

    let first = run(
        &["cli", "plugins", "install", manifest.to_str().expect("utf-8")],
        &envs,
    );
    let second = run(
        &["cli", "plugins", "install", manifest.to_str().expect("utf-8")],
        &envs,
    );
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn path_shadowing_diagnostics_are_stable_across_runs() {
    let root = temp_dir("todo-149");
    let first_dir = root.join("first");
    let second_dir = root.join("second");
    fs::create_dir_all(&first_dir).expect("mkdir first");
    fs::create_dir_all(&second_dir).expect("mkdir second");
    fs::write(first_dir.join("bijux"), "#!/bin/sh\n").expect("write first binary");
    fs::write(second_dir.join("bijux"), "#!/bin/sh\n").expect("write second binary");

    let joined = std::env::join_paths([&first_dir, &second_dir]).expect("join PATH");
    let path = joined.to_str().expect("utf-8");
    let envs = [("PATH", path)];

    let first = run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn runtime_identity_output_is_stable_under_same_ambiguous_state() {
    let root = temp_dir("todo-150");
    let first_dir = root.join("one");
    let second_dir = root.join("two");
    fs::create_dir_all(&first_dir).expect("mkdir one");
    fs::create_dir_all(&second_dir).expect("mkdir two");
    fs::write(first_dir.join("bijux"), "#!/bin/sh\n").expect("write one binary");
    fs::write(second_dir.join("bijux"), "#!/bin/sh\n").expect("write two binary");
    let path = std::env::join_paths([&first_dir, &second_dir]).expect("join PATH");
    let path_str = path.to_str().expect("utf-8");

    let envs = [("PATH", path_str), ("BIJUX_WHEEL_VERSION", "9.9.9")];
    let first = run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"], &envs);
    let second = run(&["dev", "cli", "runtime-identity", "--format", "json", "--no-pretty"], &envs);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn state_doctor_json_is_stable_under_same_corrupted_state() {
    let root = temp_dir("todo-151");
    let config = root.join("corrupt.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write corrupt config");
    let args = [
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "json",
        "--no-pretty",
        "--config-path",
        config.to_str().expect("utf-8"),
    ];

    let first = run(&args, &[]);
    let second = run(&args, &[]);
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn state_doctor_text_is_stable_under_same_corrupted_state() {
    let root = temp_dir("todo-152");
    let config = root.join("corrupt.env");
    fs::write(&config, "BROKEN_LINE\n").expect("write corrupt config");
    let args = [
        "dev",
        "cli",
        "state-doctor",
        "--format",
        "text",
        "--config-path",
        config.to_str().expect("utf-8"),
    ];

    let first = run(&args, &[]);
    let second = run(&args, &[]);
    assert_eq!(first.status.code(), second.status.code());
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn plugin_doctor_json_is_stable_under_same_corrupted_state() {
    let root_a = temp_dir("todo-153-a");
    let plugins_a = root_a.join("plugins");
    fs::create_dir_all(&plugins_a).expect("mkdir plugins a");
    fs::write(plugins_a.join("registry.json"), "{broken-json").expect("write broken a");

    let root_b = temp_dir("todo-153-b");
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
    let root_a = temp_dir("todo-154-a");
    let plugins_a = root_a.join("plugins");
    fs::create_dir_all(&plugins_a).expect("mkdir plugins a");
    fs::write(plugins_a.join("registry.json"), "{broken-json").expect("write broken a");

    let root_b = temp_dir("todo-154-b");
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
    let root = temp_dir("todo-155");
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
