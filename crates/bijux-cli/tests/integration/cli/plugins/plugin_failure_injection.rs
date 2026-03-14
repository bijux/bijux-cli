#![forbid(unsafe_code)]
//! Plugin lifecycle failure-injection coverage: install/uninstall/enable/disable/check resilience.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli as _;
use libc as _;
use serde_json::Value;
use shlex as _;
use thiserror as _;

const CURRENT_PLUGIN_HOST_FLOOR: &str = "0.2.1-dev";

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
    serde_json::from_slice(&out.stdout).expect("valid json")
}

fn tmp_dir(name: &str) -> PathBuf {
    let base =
        std::env::temp_dir().join(format!("bijux-plugin-failure-{}-{}", name, std::process::id()));
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
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"{CURRENT_PLUGIN_HOST_FLOOR}", "max_exclusive": null}},
  "namespace": "{namespace}",
  "kind": "python",
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

fn install(plugins_dir: &Path, manifest_path: &Path) {
    run_ok_json(
        &["cli", "plugins", "install", manifest_path.to_str().expect("utf-8")],
        plugins_dir,
    );
}

fn mutate_manifest<F>(manifest_path: &Path, mutator: F)
where
    F: FnOnce(&mut Value),
{
    let mut manifest: Value =
        serde_json::from_str(&fs::read_to_string(manifest_path).expect("read manifest"))
            .expect("parse manifest");
    mutator(&mut manifest);
    fs::write(manifest_path, serde_json::to_string_pretty(&manifest).expect("serialize manifest"))
        .expect("write manifest");
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
    let out =
        run(&["cli", "plugins", "install", second_manifest.to_str().expect("utf-8")], &plugins_dir);
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
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"{CURRENT_PLUGIN_HOST_FLOOR}", "max_exclusive": null}},
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
fn delegated_plugin_check_fails_when_module_file_disappears_after_install() {
    let root = tmp_dir("delegated-entrypoint-disappears");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let scaffold_dir = root.join("delegated_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "goneplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    install(&plugins_dir, &scaffold_dir.join("plugin.manifest.json"));

    fs::remove_file(scaffold_dir.join("plugin.py")).expect("remove delegated entrypoint");
    let check = run(&["cli", "plugins", "check", "goneplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("entrypoint"));

    let explain = run_ok_json(&["cli", "plugins", "explain", "goneplug"], &plugins_dir);
    assert!(explain["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|row| row["message"] == "delegated entrypoint was not found"));

    let inspect = run_ok_json(&["cli", "plugins", "inspect", "goneplug"], &plugins_dir);
    assert!(inspect["compatibility_warnings"]
        .as_array()
        .expect("compatibility warnings")
        .is_empty());
    assert!(inspect["load_diagnostics"]
        .as_array()
        .expect("load diagnostics")
        .iter()
        .any(|row| row["message"] == "delegated entrypoint was not found"));

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert_eq!(listed["status"], "degraded");
    assert_eq!(listed["load_diagnostic_count"], 1);
    assert!(listed["load_diagnostics"]
        .as_array()
        .expect("load diagnostics")
        .iter()
        .any(|row| row["message"] == "delegated entrypoint was not found"));
}

#[test]
fn plugin_check_fails_when_manifest_file_drifts_after_install() {
    let root = tmp_dir("manifest-drift");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let scaffold_dir = root.join("drift_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "driftplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    let manifest_path = scaffold_dir.join("plugin.manifest.json");
    install(&plugins_dir, &manifest_path);

    mutate_manifest(&manifest_path, |manifest| {
        manifest["version"] = Value::String("0.1.1".to_string());
    });

    let check = run(&["cli", "plugins", "check", "driftplug"], &plugins_dir);
    assert_eq!(check.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&check.stderr).contains("manifest file drifted since install"));

    let explain = run_ok_json(&["cli", "plugins", "explain", "driftplug"], &plugins_dir);
    assert!(explain["diagnostics"]
        .as_array()
        .expect("diagnostics")
        .iter()
        .any(|row| row["message"] == "manifest file drifted since install"));
}

#[test]
fn enable_rejects_plugins_with_current_runtime_issues() {
    let root = tmp_dir("enable-current-issues");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let scaffold_dir = root.join("enable_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "enableplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    install(&plugins_dir, &scaffold_dir.join("plugin.manifest.json"));
    run_ok_json(&["cli", "plugins", "disable", "enableplug"], &plugins_dir);

    fs::remove_file(scaffold_dir.join("plugin.py")).expect("remove entrypoint");
    let enable = run(&["cli", "plugins", "enable", "enableplug"], &plugins_dir);
    assert_eq!(enable.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&enable.stderr)
        .contains("cannot enable plugin with current runtime issue"));
}

#[test]
fn plugin_doctor_reports_live_runtime_failures() {
    let root = tmp_dir("doctor-live-failures");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let drift_dir = root.join("drift_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "driftdoctor",
            "--path",
            drift_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    let drift_manifest = drift_dir.join("plugin.manifest.json");
    install(&plugins_dir, &drift_manifest);
    mutate_manifest(&drift_manifest, |manifest| {
        manifest["version"] = Value::String("0.1.1".to_string());
    });

    let missing_dir = root.join("missing_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "missingdoctor",
            "--path",
            missing_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    install(&plugins_dir, &missing_dir.join("plugin.manifest.json"));
    fs::remove_file(missing_dir.join("plugin.py")).expect("remove entrypoint");

    let doctor = run_ok_json(&["cli", "plugins", "doctor"], &plugins_dir);
    assert_eq!(doctor["status"], "degraded");
    let broken = doctor["doctor"]["broken"].as_array().expect("broken array");
    assert!(broken.iter().any(|value| value == "driftdoctor"));
    assert!(broken.iter().any(|value| value == "missingdoctor"));
    assert!(doctor["load_diagnostics"]
        .as_array()
        .expect("load diagnostics")
        .iter()
        .any(|row| row["message"] == "manifest file drifted since install"));
    assert!(doctor["load_diagnostics"]
        .as_array()
        .expect("load diagnostics")
        .iter()
        .any(|row| row["message"] == "delegated entrypoint was not found"));
}

#[test]
fn delegated_plugin_install_fails_when_local_entrypoint_is_missing() {
    let root = tmp_dir("delegated-install-missing-entrypoint");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let scaffold_dir = root.join("delegated_plugin");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "missingplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    fs::remove_file(scaffold_dir.join("plugin.py")).expect("remove delegated entrypoint");

    let out = run(
        &[
            "cli",
            "plugins",
            "install",
            scaffold_dir.join("plugin.manifest.json").to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("entrypoint"));
}

#[test]
fn delegated_plugin_check_accepts_package_init_entrypoint() {
    let root = tmp_dir("delegated-package-entrypoint");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let manifest = root.join("package.manifest.json");
    fs::create_dir_all(root.join("plugin")).expect("mkdir package");
    fs::write(
        root.join("plugin").join("__init__.py"),
        "def main(argv):\n    return {'status': 'ok'}\n",
    )
    .expect("write package entrypoint");
    fs::write(
        &manifest,
        r#"{
  "name": "packageplug",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {"min_inclusive":"0.2.1-dev", "max_exclusive": "1.0.0"},
  "namespace": "packageplug",
  "kind": "python",
  "aliases": [],
  "entrypoint": "plugin:main",
  "capabilities": []
}"#,
    )
    .expect("write package manifest");
    install(&plugins_dir, &manifest);

    let check = run_ok_json(&["cli", "plugins", "check", "packageplug"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
#[cfg(unix)]
fn external_exec_plugin_install_resolves_relative_entrypoints_from_manifest_root() {
    use std::os::unix::fs::PermissionsExt;

    let root = tmp_dir("external-relative-entrypoint");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::create_dir_all(root.join("bin")).expect("mkdir bin");

    let entrypoint = root.join("bin").join("runner.sh");
    fs::write(&entrypoint, "#!/bin/sh\necho ok\n").expect("write runner");
    fs::set_permissions(&entrypoint, fs::Permissions::from_mode(0o755)).expect("chmod 755");

    let manifest = root.join("runner.manifest.json");
    fs::write(
        &manifest,
        r#"{
  "name": "runnerplug",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {"min_inclusive":"0.2.1-dev", "max_exclusive":"1.0.0"},
  "namespace": "runnerplug",
  "kind": "external-exec",
  "aliases": [],
  "entrypoint": "bin/runner.sh",
  "capabilities": []
}"#,
    )
    .expect("write manifest");

    let install =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(
        install.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&install.stderr)
    );

    let check = run_ok_json(&["cli", "plugins", "check", "runnerplug"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
#[cfg(unix)]
fn external_exec_install_keeps_manifest_anchor_when_source_label_is_overridden() {
    use std::os::unix::fs::PermissionsExt;

    let root = tmp_dir("external-source-label");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::create_dir_all(root.join("bin")).expect("mkdir bin");

    let entrypoint = root.join("bin").join("runner.sh");
    fs::write(&entrypoint, "#!/bin/sh\necho ok\n").expect("write runner");
    fs::set_permissions(&entrypoint, fs::Permissions::from_mode(0o755)).expect("chmod 755");

    let manifest = root.join("runner.manifest.json");
    fs::write(
        &manifest,
        r#"{
  "name": "runnerlabel",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {"min_inclusive":"0.2.1-dev", "max_exclusive":"1.0.0"},
  "namespace": "runnerlabel",
  "kind": "external-exec",
  "aliases": [],
  "entrypoint": "bin/runner.sh",
  "capabilities": []
}"#,
    )
    .expect("write manifest");

    let install = run(
        &[
            "cli",
            "plugins",
            "install",
            manifest.to_str().expect("utf-8"),
            "--source",
            "verified-catalog",
        ],
        &plugins_dir,
    );
    assert_eq!(
        install.status.code(),
        Some(0),
        "stderr={}",
        String::from_utf8_lossy(&install.stderr)
    );

    let check = run_ok_json(&["cli", "plugins", "check", "runnerlabel"], &plugins_dir);
    assert_eq!(check["status"], "healthy");
}

#[test]
#[cfg(unix)]
fn explain_reports_non_executable_external_entrypoints() {
    use std::os::unix::fs::PermissionsExt;

    let root = tmp_dir("external-non-executable");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let entrypoint = root.join("runner.sh");
    fs::write(&entrypoint, "#!/bin/sh\necho ok\n").expect("write runner");
    fs::set_permissions(&entrypoint, fs::Permissions::from_mode(0o644)).expect("chmod 644");

    let manifest = root.join("runner.manifest.json");
    fs::write(
        &manifest,
        format!(
            r#"{{
  "name": "noexecplug",
  "version": "0.1.0",
  "schema_version": "v2",
  "manifest_version": "v2",
  "compatibility": {{"min_inclusive":"0.2.1-dev", "max_exclusive":"1.0.0"}},
  "namespace": "noexecplug",
  "kind": "external-exec",
  "aliases": [],
  "entrypoint": "{}",
  "capabilities": []
}}"#,
            entrypoint.to_string_lossy()
        ),
    )
    .expect("write manifest");

    let out = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("not executable"));
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
    fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("serialize registry"))
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
    fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("serialize registry"))
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
    fs::write(&registry_path, serde_json::to_string_pretty(&registry).expect("serialize registry"))
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
    let first_install =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    set_writable_dir(&plugins_dir);
    assert_eq!(first_install.status.code(), Some(1));

    let second_install =
        run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert!(second_install.status.success());

    set_read_only_dir(&plugins_dir);
    let first_uninstall = run(&["cli", "plugins", "uninstall", "retryplug"], &plugins_dir);
    set_writable_dir(&plugins_dir);
    assert_eq!(first_uninstall.status.code(), Some(1));

    let second_uninstall = run(&["cli", "plugins", "uninstall", "retryplug"], &plugins_dir);
    assert!(second_uninstall.status.success());
}

#[test]
#[cfg(unix)]
fn unreadable_registry_surfaces_degraded_integrity_in_list_and_inspect() {
    use std::os::unix::fs::PermissionsExt;

    let root = tmp_dir("unreadable-registry");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let registry = plugins_dir.join("registry.json");
    fs::write(&registry, "{\"version\":\"v1\",\"plugins\":{}}").expect("seed registry");

    fs::set_permissions(&registry, fs::Permissions::from_mode(0o000)).expect("chmod 000");
    if fs::read_to_string(&registry).is_ok() {
        fs::set_permissions(&registry, fs::Permissions::from_mode(0o644)).expect("restore");
        return;
    }

    let listed = run(&["cli", "plugins", "list"], &plugins_dir);
    assert_eq!(listed.status.code(), Some(0));
    assert!(listed.stderr.is_empty());
    let listed_payload: Value = serde_json::from_slice(&listed.stdout).expect("list payload");
    assert_eq!(listed_payload["integrity_status"], "degraded");
    assert!(listed_payload["integrity_error"]
        .as_str()
        .is_some_and(|value| value.to_ascii_lowercase().contains("permission")));

    let inspected = run(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(inspected.status.code(), Some(0));
    assert!(inspected.stderr.is_empty());
    let inspect_payload: Value =
        serde_json::from_slice(&inspected.stdout).expect("inspect payload");
    assert_eq!(inspect_payload["status"], "degraded");
    assert_eq!(inspect_payload["integrity_status"], "degraded");
    assert!(inspect_payload["integrity_issues"].as_array().is_some_and(|rows| !rows.is_empty()));

    fs::set_permissions(&registry, fs::Permissions::from_mode(0o644)).expect("restore");
}
