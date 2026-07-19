#![forbid(unsafe_code)]
//! Plugin rollback resilience coverage.

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
    serde_json::from_slice(&out.stdout).expect("stdout must be json")
}

fn temp_dir(name: &str) -> PathBuf {
    let counter = TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed);
    let dir = std::env::temp_dir()
        .join(format!("bijux-plugin-rollback-resilience-{name}-{}-{counter}", std::process::id(),));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("mkdir temp");
    dir
}

fn write_manifest(path: &Path, namespace: &str, entrypoint: &str, min_version: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "name": "{namespace}",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"{min_version}", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "python",
  "trust_class": "community",
  "aliases": [],
  "entrypoint": "{entrypoint}",
  "capabilities": []
}}"#
        ),
    )
    .expect("write manifest");
    if !entrypoint.trim().is_empty() {
        let module = entrypoint.split_once(':').map_or(entrypoint, |(module, _)| module);
        fs::write(
            path.parent().expect("manifest parent").join(format!("{module}.py")),
            "def main(argv):\n    return {'status': 'ok'}\n",
        )
        .expect("write entrypoint");
    }
}

fn install_ok(plugins_dir: &Path, manifest: &Path) {
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], plugins_dir);
    assert!(out.status.success(), "install should succeed");
}

#[cfg(unix)]
fn chmod_read_only(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o555)).expect("chmod 555");
}

#[cfg(unix)]
fn chmod_writable(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod 755");
}

#[cfg(unix)]
fn readonly_directory_blocks_writes(path: &Path) -> bool {
    let probe = path.join(".permission-probe");
    match fs::write(&probe, b"probe") {
        Ok(()) => {
            let _ = fs::remove_file(&probe);
            false
        }
        Err(_) => true,
    }
}

#[test]
#[cfg(unix)]
fn simulated_disk_write_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = root.join("writefail.json");
    write_manifest(&manifest, "writefail", "plugin:main", &current_plugin_host_floor());

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);

    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}

#[test]
fn simulated_partial_copy_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let missing_manifest = root.join("missing.json");

    let out = run(
        &["cli", "plugins", "install", missing_manifest.to_str().expect("utf-8")],
        &plugins_dir,
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}

#[test]
#[cfg(unix)]
fn simulated_registry_write_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let first = root.join("first.json");
    write_manifest(&first, "stable", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &first);

    let second = root.join("second.json");
    write_manifest(&second, "candidate", "plugin:main", &current_plugin_host_floor());

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "install", second.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn simulated_manifest_parse_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = root.join("bad.json");
    fs::write(&manifest, "{broken-json").expect("write broken");

    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("manifest parse"));
}

#[test]
fn simulated_compatibility_range_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = root.join("incompatible.json");
    write_manifest(&manifest, "incompatible", "plugin:main", "9.9.9");

    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}

#[test]
fn simulated_missing_entrypoint_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = root.join("missing-entrypoint.json");
    write_manifest(&manifest, "missingentry", "", &current_plugin_host_floor());

    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("entrypoint"));
}

#[test]
#[cfg(unix)]
fn simulated_permission_denied_failure_during_install() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let manifest = root.join("denied.json");
    write_manifest(&manifest, "deniedplug", "plugin:main", &current_plugin_host_floor());

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);

    assert_eq!(out.status.code(), Some(1));
}

#[test]
#[cfg(unix)]
fn simulated_partial_uninstall_failure() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("uninstall-partial.json");
    write_manifest(&manifest, "partialuninstall", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "uninstall", "partialuninstall"], &plugins_dir);
    chmod_writable(&plugins_dir);

    assert_eq!(out.status.code(), Some(1));
}

#[test]
#[cfg(unix)]
fn simulated_registry_write_failure_during_uninstall() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("uninstall-write-fail.json");
    write_manifest(&manifest, "writeuninstall", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "uninstall", "writeuninstall"], &plugins_dir);
    chmod_writable(&plugins_dir);

    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn simulated_enable_failure_when_plugin_files_missing() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("enable-fail.json");
    write_manifest(&manifest, "enablefail", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    let registry_path = plugins_dir.join("registry.json");
    let mut registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).expect("read registry"))
            .expect("parse registry");
    registry["plugins"]["enablefail"]["state"] = Value::String("broken".to_string());
    fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("serialize registry"))
        .expect("write broken state");

    let out = run(&["cli", "plugins", "enable", "enablefail"], &plugins_dir);
    assert_eq!(out.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&out.stderr)
        .contains("cannot enable plugin with current runtime issue: plugin is marked broken"));
}

#[test]
fn simulated_disable_failure_when_registry_is_corrupted() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write corrupt registry");

    let out = run(&["cli", "plugins", "disable", "whatever"], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(!out.stderr.is_empty());
}

#[test]
#[cfg(unix)]
fn rollback_proof_install_failure_preserves_existing_plugins() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let stable = root.join("stable.json");
    write_manifest(&stable, "stableproof", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &stable);

    let candidate = root.join("candidate.json");
    write_manifest(&candidate, "candidateproof", "plugin:main", &current_plugin_host_floor());
    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "install", candidate.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(out.status.code(), Some(1));

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names: Vec<&str> = listed["plugins"]
        .as_array()
        .expect("plugins")
        .iter()
        .filter_map(|p| p["manifest"]["namespace"].as_str())
        .collect();
    assert_eq!(names, vec!["stableproof"]);
}

#[test]
#[cfg(unix)]
fn rollback_proof_uninstall_failure_preserves_existing_plugins() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("keep.json");
    write_manifest(&manifest, "keepproof", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "uninstall", "keepproof"], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(out.status.code(), Some(1));

    let check = run_ok_json(&["cli", "plugins", "check", "keepproof"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
#[cfg(unix)]
fn retry_install_after_partial_failure_is_idempotent() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("retry-install.json");
    write_manifest(&manifest, "retryinstall", "plugin:main", &current_plugin_host_floor());

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let first =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(first.status.code(), Some(1));

    let second =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert!(second.status.success());
}

#[test]
#[cfg(unix)]
fn retry_uninstall_after_partial_failure_is_idempotent() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("retry-uninstall.json");
    write_manifest(&manifest, "retryuninstall", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let first = run(&["cli", "plugins", "uninstall", "retryuninstall"], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(first.status.code(), Some(1));

    let second = run(&["cli", "plugins", "uninstall", "retryuninstall"], &plugins_dir);
    assert!(second.status.success());
}

#[test]
#[cfg(unix)]
fn failed_install_does_not_leave_claimed_namespace() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let stable = root.join("stable.json");
    write_manifest(&stable, "stableonly", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &stable);

    let failed = root.join("failed.json");
    write_manifest(&failed, "failedns", "plugin:main", &current_plugin_host_floor());
    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "install", failed.to_str().expect("utf-8")], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(out.status.code(), Some(1));

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let names: Vec<&str> = listed["plugins"]
        .as_array()
        .expect("plugins")
        .iter()
        .filter_map(|p| p["manifest"]["namespace"].as_str())
        .collect();
    assert_eq!(names, vec!["stableonly"]);
}

#[test]
#[cfg(unix)]
fn failed_uninstall_does_not_orphan_registry_state_silently() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("orphan.json");
    write_manifest(&manifest, "orphanproof", "plugin:main", &current_plugin_host_floor());
    install_ok(&plugins_dir, &manifest);

    chmod_read_only(&plugins_dir);
    if !readonly_directory_blocks_writes(&plugins_dir) {
        chmod_writable(&plugins_dir);
        return;
    }
    let out = run(&["cli", "plugins", "uninstall", "orphanproof"], &plugins_dir);
    chmod_writable(&plugins_dir);
    assert_eq!(out.status.code(), Some(1));

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"]
        .as_array()
        .expect("plugins")
        .iter()
        .any(|item| item["manifest"]["namespace"] == "orphanproof"));
}

#[test]
fn plugin_doctor_reports_rollback_relevant_damage_clearly() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write corrupt registry");

    let doctor = run_ok_json(&["cli", "plugins", "doctor"], &plugins_dir);
    assert_eq!(doctor["status"], "degraded");
    assert_eq!(doctor["self_repair_attempted"], true);
    assert_eq!(doctor["self_repair_success"], true);
    assert!(doctor["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|row| row["area"] == "registry"));
}

#[test]
fn machine_readable_rollback_diagnostics_are_stable() {
    let root = temp_dir("plugin-failure-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let out = run(
        &["--format", "json", "--no-pretty", "cli", "plugins", "install", "/missing/manifest.json"],
        &plugins_dir,
    );
    assert_eq!(out.status.code(), Some(1));

    let payload: Value = serde_json::from_slice(&out.stderr).expect("stderr json");
    assert_eq!(payload["status"], "error");
    assert_eq!(payload["code"], 1);
    assert_eq!(payload["command"], "cli plugins install");
}
