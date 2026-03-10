#![forbid(unsafe_code)]
//! Plugin lifecycle failure-injection coverage: install/uninstall/enable/disable/check resilience.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli as _;
use bijux_cli_python as _;
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

fn run_ok_json(args: &[&str], plugins_dir: &Path) -> Value {
    let out = run(args, plugins_dir);
    assert!(out.status.success(), "command failed: {args:?}");
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn tmp_dir(name: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "bijux-plugin-failure-{}-{}",
        name,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir temp");
    base
}

fn write_python_manifest(path: &Path, namespace: &str, entrypoint: &str) {
    fs::write(
        path,
        format!(
            r#"{{
  "name": "{namespace}",
  "version": "0.1.0",
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {{"min_inclusive":"0.1.0", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "python",
  "aliases": [],
  "entrypoint": "{entrypoint}",
  "capabilities": []
}}"#
        ),
    )
    .expect("write manifest");
}

fn install(plugins_dir: &Path, manifest_path: &Path) {
    run_ok_json(
        &[
            "cli",
            "plugins",
            "install",
            manifest_path.to_str().expect("utf-8"),
        ],
        plugins_dir,
    );
}

#[cfg(unix)]
fn set_read_only_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o555)).expect("chmod 555");
}

#[cfg(unix)]
fn set_writable_dir(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(path, fs::Permissions::from_mode(0o755)).expect("chmod 755");
}

#[test]
#[cfg(unix)]
fn install_reports_write_failures_and_preserves_existing_registry_entries() {
    let root = tmp_dir("install-write-failures");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let first_manifest = root.join("ok.json");
    write_python_manifest(&first_manifest, "stableplug", "plugin:main");
    install(&plugins_dir, &first_manifest);

    let second_manifest = root.join("candidate.json");
    write_python_manifest(&second_manifest, "candidateplug", "plugin:main");

    set_read_only_dir(&plugins_dir);
    let out = run(
        &[
            "cli",
            "plugins",
            "install",
            second_manifest.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    set_writable_dir(&plugins_dir);

    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(!out.stderr.is_empty());

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    let plugins = listed["plugins"].as_array().expect("plugins array");
    assert_eq!(plugins.len(), 1);
    assert_eq!(plugins[0]["manifest"]["namespace"], "stableplug");
}

#[test]
fn plugin_check_fails_when_entrypoint_disappears_after_install() {
    let root = tmp_dir("entrypoint-disappears");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("entrypoint.json");
    let entrypoint = root.join("runner.sh");
    fs::write(&entrypoint, "#!/bin/sh\necho ok\n").expect("write entrypoint");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&entrypoint, fs::Permissions::from_mode(0o755))
            .expect("set executable");
    }
    fs::write(
        &manifest,
        format!(
            r#"{{
  "name": "goneplug",
  "version": "0.1.0",
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {{"min_inclusive":"0.1.0", "max_exclusive": null}},
  "namespace": "goneplug",
  "kind": "external-exec",
  "aliases": [],
  "entrypoint": "{}",
  "capabilities": []
}}"#,
            entrypoint.to_string_lossy()
        ),
    )
    .expect("write external manifest");
    install(&plugins_dir, &manifest);

    fs::remove_file(&entrypoint).expect("remove entrypoint");
    let check = run(&["cli", "plugins", "check", "goneplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("entrypoint"));
}

#[test]
fn plugin_check_fails_when_manifest_mutates_after_install() {
    let root = tmp_dir("manifest-mutates");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("manifest.json");
    write_python_manifest(&manifest, "mutateplug", "plugin:main");
    install(&plugins_dir, &manifest);

    let registry_path = plugins_dir.join("registry.json");
    let mut registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).expect("read registry"))
            .expect("parse registry");
    registry["plugins"]["mutateplug"]["manifest"]["entrypoint"] = Value::String("".to_string());
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&registry).expect("serialize registry"),
    )
    .expect("write mutated registry");

    let check = run(&["cli", "plugins", "check", "mutateplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&check.stderr).contains("entrypoint"));
}

#[test]
fn plugin_check_fails_when_runtime_kind_becomes_unsupported() {
    let root = tmp_dir("runtime-kind-unsupported");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("kind.json");
    write_python_manifest(&manifest, "nativeplug", "plugin:main");
    install(&plugins_dir, &manifest);

    let registry_path = plugins_dir.join("registry.json");
    let mut registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).expect("read registry"))
            .expect("parse registry");
    registry["plugins"]["nativeplug"]["manifest"]["kind"] = Value::String("native".to_string());
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&registry).expect("serialize registry"),
    )
    .expect("write mutated registry");

    let check = run(&["cli", "plugins", "check", "nativeplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(1));
    assert!(!check.stderr.is_empty());
}

#[test]
fn check_fails_on_broken_registry_record_and_list_stays_usable_after_doctor() {
    let root = tmp_dir("inspect-broken-record");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let good_manifest = root.join("good.json");
    write_python_manifest(&good_manifest, "healthyplug", "plugin:main");
    install(&plugins_dir, &good_manifest);

    let bad_manifest = root.join("bad.json");
    write_python_manifest(&bad_manifest, "brokenplug", "plugin:main");
    install(&plugins_dir, &bad_manifest);

    let registry_path = plugins_dir.join("registry.json");
    let mut registry: Value =
        serde_json::from_str(&fs::read_to_string(&registry_path).expect("read registry"))
            .expect("parse registry");
    registry["plugins"]["brokenplug"]["manifest"] =
        Value::String("invalid-manifest-shape".to_string());
    fs::write(
        &registry_path,
        serde_json::to_string_pretty(&registry).expect("serialize registry"),
    )
    .expect("write broken registry record");

    let check = run(&["cli", "plugins", "check", "brokenplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(1));

    let doctor = run_ok_json(&["cli", "plugins", "doctor"], &plugins_dir);
    assert_eq!(doctor["status"], "ok");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"].is_array());
}

#[test]
#[cfg(unix)]
fn uninstall_disable_enable_failures_do_not_break_existing_plugin_state() {
    let root = tmp_dir("toggle-failure");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("toggle.json");
    write_python_manifest(&manifest, "toggleplug", "plugin:main");
    install(&plugins_dir, &manifest);

    set_read_only_dir(&plugins_dir);
    let disable_fail = run(&["cli", "plugins", "disable", "toggleplug"], &plugins_dir);
    let enable_fail = run(&["cli", "plugins", "enable", "toggleplug"], &plugins_dir);
    let uninstall_fail = run(&["cli", "plugins", "uninstall", "toggleplug"], &plugins_dir);
    set_writable_dir(&plugins_dir);

    assert_eq!(disable_fail.status.code(), Some(1));
    assert_eq!(enable_fail.status.code(), Some(1));
    assert_eq!(uninstall_fail.status.code(), Some(1));

    let check = run_ok_json(&["cli", "plugins", "check", "toggleplug"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
#[cfg(unix)]
fn install_and_uninstall_retries_are_idempotent_after_transient_write_failures() {
    let root = tmp_dir("retry-idempotent");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("retry.json");
    write_python_manifest(&manifest, "retryplug", "plugin:main");

    set_read_only_dir(&plugins_dir);
    let first_install = run(
        &[
            "cli",
            "plugins",
            "install",
            manifest.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    set_writable_dir(&plugins_dir);
    assert_eq!(first_install.status.code(), Some(1));

    let second_install = run(
        &[
            "cli",
            "plugins",
            "install",
            manifest.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    assert!(second_install.status.success());

    set_read_only_dir(&plugins_dir);
    let first_uninstall = run(&["cli", "plugins", "uninstall", "retryplug"], &plugins_dir);
    set_writable_dir(&plugins_dir);
    assert_eq!(first_uninstall.status.code(), Some(1));

    let second_uninstall = run(&["cli", "plugins", "uninstall", "retryplug"], &plugins_dir);
    assert!(second_uninstall.status.success());
}
