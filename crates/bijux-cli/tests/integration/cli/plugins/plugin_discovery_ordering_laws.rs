#![forbid(unsafe_code)]
//! Plugin discovery and ordering laws.

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

fn run_ok_json(args: &[&str], plugins_dir: &Path) -> Value {
    let out = run(args, plugins_dir);
    assert!(out.status.success(), "command failed: {args:?}");
    serde_json::from_slice(&out.stdout).expect("stdout json")
}

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir()
        .join(format!("bijux-plugin-discovery-laws-{name}-{}-{counter}", std::process::id(),));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn scaffold_manifest(root: &Path, plugins_dir: &Path, namespace: &str) -> PathBuf {
    let scaffold_dir = root.join(format!("{namespace}_scaffold"));
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            namespace,
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        plugins_dir,
    );
    scaffold_dir.join("plugin.manifest.json")
}

fn install_python(root: &Path, plugins_dir: &Path, namespace: &str) {
    let manifest = scaffold_manifest(root, plugins_dir, namespace);
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], plugins_dir);
    assert!(out.status.success(), "install should succeed for {namespace}");
}

fn install_external_exec(root: &Path, plugins_dir: &Path, namespace: &str, entrypoint: &Path) {
    let current_plugin_host_floor = current_plugin_host_floor();
    let manifest = root.join(format!("{namespace}.manifest.json"));
    fs::write(
        &manifest,
        format!(
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
        ),
    )
    .expect("write external manifest");
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], plugins_dir);
    assert!(out.status.success(), "external install should succeed");
}

fn write_executable(path: &Path) {
    fs::write(path, "#!/bin/sh\necho ok\n").expect("write executable");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod +x");
    }
}

fn plugin_names(listed: &Value) -> Vec<String> {
    listed["plugins"]
        .as_array()
        .expect("plugins array")
        .iter()
        .filter_map(|item| item["manifest"]["namespace"].as_str().map(ToOwned::to_owned))
        .collect()
}

#[test]
fn deterministic_discovery_under_shuffled_install_order() {
    let root_a = temp_dir("plugin-discovery-a");
    let dir_a = root_a.join("plugins");
    fs::create_dir_all(&dir_a).expect("mkdir plugins");
    install_python(&root_a, &dir_a, "gamma");
    install_python(&root_a, &dir_a, "alpha");

    let root_b = temp_dir("plugin-discovery-b");
    let dir_b = root_b.join("plugins");
    fs::create_dir_all(&dir_b).expect("mkdir plugins");
    install_python(&root_b, &dir_b, "alpha");
    install_python(&root_b, &dir_b, "gamma");

    let names_a = plugin_names(&run_ok_json(&["cli", "plugins", "list"], &dir_a));
    let names_b = plugin_names(&run_ok_json(&["cli", "plugins", "list"], &dir_b));
    assert_eq!(names_a, vec!["alpha".to_string(), "gamma".to_string()]);
    assert_eq!(names_a, names_b);
}

#[test]
fn deterministic_plugin_list_ordering() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "zeta");
    install_python(&root, &plugins_dir, "beta");

    let names = plugin_names(&run_ok_json(&["cli", "plugins", "list"], &plugins_dir));
    assert_eq!(names, vec!["beta".to_string(), "zeta".to_string()]);
}

#[test]
fn deterministic_plugin_inspect_ordering_multiple_plugins() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "omega");
    install_python(&root, &plugins_dir, "alpha");

    let first = run_ok_json(&["cli", "plugins", "inspect"], &plugins_dir);
    let second = run_ok_json(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(first, second);
}

#[test]
fn deterministic_help_ordering_with_plugins_installed() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "helpalpha");

    let first = run(&["help", "cli", "plugins"], &plugins_dir);
    let second = run(&["help", "cli", "plugins"], &plugins_dir);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn deterministic_route_registration_with_different_install_orders() {
    let root_a = temp_dir("plugin-discovery-a");
    let dir_a = root_a.join("plugins");
    fs::create_dir_all(&dir_a).expect("mkdir plugins");
    install_python(&root_a, &dir_a, "routegamma");
    install_python(&root_a, &dir_a, "routealpha");

    let root_b = temp_dir("plugin-discovery-b");
    let dir_b = root_b.join("plugins");
    fs::create_dir_all(&dir_b).expect("mkdir plugins");
    install_python(&root_b, &dir_b, "routealpha");
    install_python(&root_b, &dir_b, "routegamma");

    let a = run(&["routealpha", "--help"], &dir_a);
    let b = run(&["routealpha", "--help"], &dir_b);
    assert_eq!(a.status.code(), Some(0));
    assert_eq!(b.status.code(), Some(0));
    assert_eq!(a.stdout, b.stdout);
    assert_eq!(a.stderr, b.stderr);
}

#[test]
fn deterministic_route_registration_after_uninstall_reinstall_cycles() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = scaffold_manifest(&root, &plugins_dir, "cycleplug");
    let first_install =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert!(first_install.status.success());

    run_ok_json(&["cli", "plugins", "uninstall", "cycleplug"], &plugins_dir);
    let second_install =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert!(second_install.status.success());

    let out = run(&["cycleplug", "--help"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    let payload: Value = serde_json::from_slice(&out.stdout).expect("stdout json");
    assert_eq!(payload["status"], "ok");
    assert_eq!(payload["argv"], serde_json::json!(["--help"]));
    assert!(out.stderr.is_empty());
}

#[test]
fn deterministic_namespace_conflict_resolution_messages() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "dupeplug");

    let manifest = root.join("dupeplug_scaffold").join("plugin.manifest.json");
    let first =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    let second =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(first.status.code(), Some(1));
    assert_eq!(second.status.code(), Some(1));
    assert_eq!(first.stderr, second.stderr);
}

#[test]
fn deterministic_plugins_list_json_output() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "jsonlist");

    let first = run(&["--format", "json", "--no-pretty", "cli", "plugins", "list"], &plugins_dir);
    let second = run(&["--format", "json", "--no-pretty", "cli", "plugins", "list"], &plugins_dir);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn deterministic_plugins_check_json_output() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "jsoncheck");

    let first = run(
        &["cli", "plugins", "check", "jsoncheck", "--format", "json", "--no-pretty"],
        &plugins_dir,
    );
    let second = run(
        &["cli", "plugins", "check", "jsoncheck", "--format", "json", "--no-pretty"],
        &plugins_dir,
    );
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn deterministic_plugins_inspect_json_output() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "jsoninspect");

    let first =
        run(&["--format", "json", "--no-pretty", "cli", "plugins", "inspect"], &plugins_dir);
    let second =
        run(&["--format", "json", "--no-pretty", "cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(first.stdout, second.stdout);
}

#[test]
fn discovery_ignores_unrelated_filesystem_clutter() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "cleanplug");

    fs::write(plugins_dir.join("README.txt"), "not a plugin").expect("write clutter");
    fs::create_dir_all(plugins_dir.join("random-dir")).expect("mkdir clutter dir");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert_eq!(names, vec!["cleanplug".to_string()]);
}

#[test]
fn discovery_ignores_partially_written_temporary_files() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "stableplug");

    fs::write(plugins_dir.join("registry.json.tmp"), "{partial").expect("write tmp file");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert_eq!(names, vec!["stableplug".to_string()]);
}

#[test]
fn discovery_ignores_invalid_directories_cleanly() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "validplug");

    let invalid_dir = plugins_dir.join("invalid-plugin-dir");
    fs::create_dir_all(&invalid_dir).expect("mkdir invalid dir");
    fs::write(invalid_dir.join("junk.txt"), "junk").expect("write junk");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert_eq!(names, vec!["validplug".to_string()]);
}

#[test]
#[cfg(unix)]
fn discovery_is_stable_under_broken_symlink_entries() {
    use std::os::unix::fs::symlink;

    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "symlinkplug");

    symlink(root.join("missing-target"), plugins_dir.join("broken-link"))
        .expect("create broken symlink");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert_eq!(names, vec!["symlinkplug".to_string()]);
}

#[test]
fn broken_plugin_does_not_reorder_healthy_plugins() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    install_python(&root, &plugins_dir, "alphahealthy");
    install_python(&root, &plugins_dir, "bravohealthy");

    let broken_entrypoint = root.join("broken.sh");
    write_executable(&broken_entrypoint);
    install_external_exec(&root, &plugins_dir, "brokenplug", &broken_entrypoint);
    fs::remove_file(&broken_entrypoint).expect("remove broken entrypoint");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert_eq!(names, vec!["alphahealthy", "bravohealthy", "brokenplug"]);
}

#[test]
fn broken_plugin_does_not_hide_healthy_plugins() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    install_python(&root, &plugins_dir, "visiblehealthy");
    let broken_entrypoint = root.join("broken-visible.sh");
    write_executable(&broken_entrypoint);
    install_external_exec(&root, &plugins_dir, "visiblebroken", &broken_entrypoint);
    fs::remove_file(&broken_entrypoint).expect("remove broken entrypoint");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names = plugin_names(&listed);
    assert!(names.contains(&"visiblehealthy".to_string()));
    assert!(names.contains(&"visiblebroken".to_string()));
}

#[test]
fn registry_and_discovery_disagreement_diagnostics_are_deterministic() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write corrupt registry");
    let first = run(&["cli", "plugins", "doctor"], &plugins_dir);
    let second = run(&["cli", "plugins", "doctor"], &plugins_dir);
    let third = run(&["cli", "plugins", "doctor"], &plugins_dir);
    assert_eq!(first.status.code(), Some(0));
    assert_eq!(second.status.code(), Some(0));
    assert_eq!(third.status.code(), Some(0));
    let first_payload: Value = serde_json::from_slice(&first.stdout).expect("first doctor payload");
    let second_payload: Value =
        serde_json::from_slice(&second.stdout).expect("second doctor payload");
    let third_payload: Value = serde_json::from_slice(&third.stdout).expect("third doctor payload");
    assert_eq!(first_payload["status"], "degraded");
    assert_eq!(first_payload["self_repair_attempted"], true);
    assert_eq!(first_payload["self_repair_success"], true);
    assert_eq!(second_payload["status"], "ok");
    assert_eq!(second_payload["self_repair_attempted"], false);
    assert_eq!(second_payload["self_repair_success"], false);
    assert_eq!(second_payload, third_payload);
    assert_eq!(second.stdout, third.stdout);
}

#[test]
fn plugin_metadata_ordering_is_stable_in_machine_output() {
    let root = temp_dir("plugin-discovery");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    install_python(&root, &plugins_dir, "metastable");

    let out = run(&["--format", "json", "--no-pretty", "cli", "plugins", "list"], &plugins_dir);
    assert_eq!(out.status.code(), Some(0));
    let body = String::from_utf8(out.stdout).expect("stdout utf-8");
    let pos_plugins = body.find("\"plugins\"").expect("plugins key");
    let pos_directory = body.find("\"directory\"").expect("directory key");
    assert!(pos_directory < pos_plugins, "top-level key order should stay stable");
}
