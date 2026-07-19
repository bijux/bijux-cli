#![forbid(unsafe_code)]
//! Plugin lifecycle end-to-end coverage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

use super::current_plugin_host_floor;

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

fn run(args: &[&str], plugins_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bijux"))
        .args(args)
        .env("BIJUXCLI_PLUGINS_DIR", plugins_dir)
        .output()
        .expect("binary should execute")
}

fn run_with_env(args: &[&str], plugins_dir: &Path, envs: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
    command.args(args).env("BIJUXCLI_PLUGINS_DIR", plugins_dir);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("binary should execute")
}

fn run_ok_json(args: &[&str], plugins_dir: &Path) -> Value {
    let out = run(args, plugins_dir);
    assert!(out.status.success(), "command failed: {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn tmp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir()
        .join(format!("bijux-plugin-lifecycle-{name}-{}-{counter}", std::process::id(),));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp root");
    root
}

fn manifest_path(scaffold_dir: &Path) -> String {
    scaffold_dir.join("plugin.manifest.json").to_str().expect("utf-8").to_string()
}

fn add_alias(manifest_path: &Path, alias: &str) {
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("read manifest"))
            .expect("parse manifest");
    manifest["aliases"] = Value::Array(vec![Value::String(alias.to_string())]);
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest).expect("serialize manifest"))
        .expect("write manifest");
}

fn scaffold_and_install(kind: &str, namespace: &str, root: &Path, plugins_dir: &Path) -> String {
    let scaffold_dir = root.join(format!("{namespace}_{kind}_scaffold"));
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            kind,
            namespace,
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        plugins_dir,
    );
    let manifest = manifest_path(&scaffold_dir);
    run_ok_json(&["cli", "plugins", "install", manifest.as_str()], plugins_dir);
    manifest
}

fn write_external_exec_manifest(path: &Path, namespace: &str, entrypoint: &Path) {
    let current_plugin_host_floor = current_plugin_host_floor();
    let manifest = format!(
        r#"{{
  "name": "{namespace}",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"{current_plugin_host_floor}", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "external-exec",
  "trust_class": "community",
  "aliases": [],
  "entrypoint": "{}",
  "capabilities": []
}}"#,
        entrypoint.to_string_lossy()
    );
    fs::write(path, manifest).expect("write external-exec manifest");
}

#[cfg(unix)]
fn mark_executable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;

    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod +x");
}

#[test]
fn python_scaffold_install_list_inspect_uninstall_end_to_end() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "pychain", &root, &plugins_dir);

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"]
        .as_array()
        .expect("plugins array")
        .iter()
        .any(|item| item["manifest"]["namespace"] == "pychain"));

    let inspected = run_ok_json(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(inspected["status"], "loaded");

    let uninstalled = run_ok_json(&["cli", "plugins", "uninstall", "pychain"], &plugins_dir);
    assert_eq!(uninstalled["status"], "uninstalled");
}

#[test]
fn rust_scaffold_install_list_inspect_uninstall_end_to_end() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("rust", "rustchain", &root, &plugins_dir);

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"]
        .as_array()
        .expect("plugins array")
        .iter()
        .any(|item| item["manifest"]["namespace"] == "rustchain"));

    let inherited_target_dir = root.join("shared-target");
    let routed = run_with_env(
        &["rustchain", "--help"],
        &plugins_dir,
        &[("CARGO_TARGET_DIR", inherited_target_dir.to_str().expect("shared target utf-8"))],
    );
    assert_eq!(
        routed.status.code(),
        Some(0),
        "generated Rust plugin failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&routed.stdout),
        String::from_utf8_lossy(&routed.stderr),
    );
    assert!(String::from_utf8_lossy(&routed.stdout).contains("Usage: rustchain [ARGS]"));
    assert!(routed.stderr.is_empty());

    let inspected = run_ok_json(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(inspected["status"], "loaded");

    let uninstalled = run_ok_json(&["cli", "plugins", "uninstall", "rustchain"], &plugins_dir);
    assert_eq!(uninstalled["status"], "uninstalled");
}

#[test]
fn installed_plugin_route_executes_structured_python_output() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "helpplug", &root, &plugins_dir);

    let out = run(&["helpplug", "--help"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["argv"], serde_json::json!(["--help"]));
}

#[test]
fn installed_plugin_alias_route_executes_the_canonical_plugin() {
    let root = tmp_dir("plugin-alias-route");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let scaffold_dir = root.join("alias_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "aliasplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    add_alias(&scaffold_dir.join("plugin.manifest.json"), "alias-short");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "install",
            scaffold_dir.join("plugin.manifest.json").to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );

    let out = run(&["alias-short", "--help"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["argv"], serde_json::json!(["--help"]));
}

#[test]
fn runtime_does_not_invent_placeholder_plugin_routes() {
    let root = tmp_dir("no-placeholder-plugin-route");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let out = run(&["community", "status"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    let stderr = String::from_utf8(out.stderr).expect("stderr utf-8");
    assert!(stderr.contains("unknown route") || stderr.contains("Did you mean"));
}

#[test]
fn installed_external_exec_plugin_route_passthrough_is_stable() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let entry_file = root.join("routeplug.sh");
    fs::write(
        &entry_file,
        "#!/bin/sh\nprintf 'route:%s\\n' \"$1\"\nprintf 'warn:%s\\n' \"$2\" >&2\n",
    )
    .expect("write entrypoint");
    #[cfg(unix)]
    mark_executable(&entry_file);

    let manifest = root.join("routeplug.manifest.json");
    write_external_exec_manifest(&manifest, "routeplug", &entry_file);
    run_ok_json(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);

    let out = run(&["routeplug", "alpha", "beta"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout), "route:alpha\n");
    assert_eq!(String::from_utf8_lossy(&out.stderr), "warn:beta\n");
}

#[test]
fn installed_plugin_disable_rejects_plugin_check() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "disableplug", &root, &plugins_dir);
    let disabled = run_ok_json(&["cli", "plugins", "disable", "disableplug"], &plugins_dir);
    assert_eq!(disabled["status"], "disabled");

    let out = run(&["cli", "plugins", "check", "disableplug"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("disabled"));
}

#[test]
fn disabled_plugin_enable_restores_plugin_check() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "toggleplug", &root, &plugins_dir);
    run_ok_json(&["cli", "plugins", "disable", "toggleplug"], &plugins_dir);
    let enabled = run_ok_json(&["cli", "plugins", "enable", "toggleplug"], &plugins_dir);
    assert_eq!(enabled["status"], "enabled");

    let check = run_ok_json(&["cli", "plugins", "check", "toggleplug"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
fn duplicate_install_without_force_is_deterministic_rejection() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = scaffold_and_install("python", "dupeplug", &root, &plugins_dir);
    let out = run(&["cli", "plugins", "install", manifest.as_str()], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("already installed"));
}

#[test]
fn duplicate_install_force_flag_behavior_is_deterministic_when_unsupported() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = scaffold_and_install("python", "forceplug", &root, &plugins_dir);
    let out = run(&["cli", "plugins", "install", manifest.as_str(), "--force"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("Usage: bijux"));
}

#[test]
fn uninstall_missing_plugin_returns_stable_failure() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let out = run(&["cli", "plugins", "uninstall", "missingplug"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not found"));
}

#[test]
fn inspect_broken_registry_returns_stable_diagnostics() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write broken registry");
    let out = run(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["status"], "degraded");
    assert_eq!(payload["integrity_status"], "degraded");
    assert!(payload["integrity_issues"].as_array().is_some_and(|rows| !rows.is_empty()));
    assert!(payload["plugins"].as_array().expect("plugins array").is_empty());
    assert!(out.stderr.is_empty());
}

#[test]
fn plugin_check_after_entrypoint_deletion_reports_stable_failure() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let entry_file = root.join("goneplug.sh");
    fs::write(&entry_file, "#!/bin/sh\necho ok\n").expect("write entrypoint");
    #[cfg(unix)]
    mark_executable(&entry_file);
    let manifest = root.join("goneplug.manifest.json");
    write_external_exec_manifest(&manifest, "goneplug", &entry_file);
    run_ok_json(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    fs::remove_file(entry_file).expect("remove entrypoint file");

    let out = run(&["cli", "plugins", "check", "goneplug"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr).contains("entrypoint"));
}

#[test]
fn plugin_help_flows_through_root_help_tree() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let out = run(&["help", "cli", "plugins"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    let text = String::from_utf8(out.stdout).expect("utf-8");
    assert!(text.contains("inspect"));
    assert!(text.contains("uninstall"));
}

#[test]
fn plugin_command_output_uses_core_envelope_rules() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed.get("plugins").is_some());
    assert!(listed.get("directory").is_some());
}

#[test]
fn plugin_command_stderr_stdout_discipline_is_stable() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let out = run(&["cli", "plugins", "install", "/missing/manifest.json"], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());
}

#[test]
fn plugin_command_exit_codes_map_through_core_rules() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let success = run(&["cli", "plugins", "list"], &plugins_dir);
    assert_eq!(success.status.code(), Some(0));

    let failure = run(&["cli", "plugins", "uninstall", "missing"], &plugins_dir);
    assert_eq!(failure.status.code(), Some(2));
}

#[test]
fn two_plugins_keep_stable_ordering_in_list() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "zeta", &root, &plugins_dir);
    scaffold_and_install("python", "alpha", &root, &plugins_dir);

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names: Vec<String> = listed["plugins"]
        .as_array()
        .expect("plugins array")
        .iter()
        .filter_map(|item| item["manifest"]["namespace"].as_str().map(ToOwned::to_owned))
        .collect();
    assert_eq!(names, vec!["alpha".to_string(), "zeta".to_string()]);
}

#[test]
fn uninstalling_one_plugin_does_not_affect_other() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "firstplug", &root, &plugins_dir);
    scaffold_and_install("python", "secondplug", &root, &plugins_dir);

    run_ok_json(&["cli", "plugins", "uninstall", "firstplug"], &plugins_dir);

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names: Vec<&str> = listed["plugins"]
        .as_array()
        .expect("plugins array")
        .iter()
        .filter_map(|item| item["manifest"]["namespace"].as_str())
        .collect();
    assert_eq!(names, vec!["secondplug"]);
}

#[test]
fn registry_survives_restart_after_successful_install() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "restartinstall", &root, &plugins_dir);

    let first = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let second = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert_eq!(first, second);
}

#[test]
fn registry_survives_restart_after_successful_uninstall() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "restartremove", &root, &plugins_dir);
    run_ok_json(&["cli", "plugins", "uninstall", "restartremove"], &plugins_dir);

    let first = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let second = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert_eq!(first, second);
    assert!(first["plugins"].as_array().expect("plugins array").is_empty());
}

#[test]
fn plugin_check_reports_healthy_and_unhealthy_in_same_registry() {
    let root = tmp_dir("plugin-lifecycle");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    scaffold_and_install("python", "healthyplug", &root, &plugins_dir);
    let entry_file = root.join("unhealthyplug.sh");
    fs::write(&entry_file, "#!/bin/sh\necho broken\n").expect("write unhealthy entrypoint");
    #[cfg(unix)]
    mark_executable(&entry_file);
    let unhealthy_manifest = root.join("unhealthyplug.manifest.json");
    write_external_exec_manifest(&unhealthy_manifest, "unhealthyplug", &entry_file);
    run_ok_json(
        &["cli", "plugins", "install", unhealthy_manifest.to_str().expect("utf-8")],
        &plugins_dir,
    );
    fs::remove_file(entry_file).expect("remove entrypoint");

    let healthy = run_ok_json(&["cli", "plugins", "check", "healthyplug"], &plugins_dir);
    assert_eq!(healthy["status"], "healthy");

    let unhealthy = run(&["cli", "plugins", "check", "unhealthyplug"], &plugins_dir);
    assert_eq!(unhealthy.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&unhealthy.stderr).contains("entrypoint"));
}
