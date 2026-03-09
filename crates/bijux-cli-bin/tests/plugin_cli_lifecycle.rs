#![forbid(unsafe_code)]
//! Plugin scaffold/install lifecycle integration and failure-path coverage.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use bijux_cli_core as _;
use libc as _;
use serde_json::Value;

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
    let base = std::env::temp_dir().join(format!("bijux-plugin-cli-{}-{}", name, std::process::id()));
    let _ = fs::remove_dir_all(&base);
    fs::create_dir_all(&base).expect("mkdir temp");
    base
}

fn manifest_file(scaffold_dir: &Path) -> PathBuf {
    scaffold_dir.join("plugin.manifest.json")
}

#[test]
fn python_scaffold_install_list_inspect_uninstall_flow() {
    let root = tmp_dir("python-flow");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let scaffold_dir = root.join("python_plugin");

    let scaffold = run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "pyflow",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    assert_eq!(scaffold["status"], "scaffolded");

    let install = run_ok_json(
        &[
            "cli",
            "plugins",
            "install",
            manifest_file(&scaffold_dir).to_str().expect("utf-8"),
            "--source",
            "local",
            "--trust",
            "community",
        ],
        &plugins_dir,
    );
    assert_eq!(install["status"], "installed");

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"].as_array().expect("plugins array").iter().any(
        |item| item["manifest"]["namespace"] == "pyflow"
    ));

    let inspected = run_ok_json(&["cli", "plugins", "inspect"], &plugins_dir);
    assert_eq!(inspected["status"], "loaded");

    let uninstall = run_ok_json(&["cli", "plugins", "uninstall", "pyflow"], &plugins_dir);
    assert_eq!(uninstall["status"], "uninstalled");
}

#[test]
fn rust_scaffold_install_list_inspect_uninstall_flow() {
    let root = tmp_dir("rust-flow");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let scaffold_dir = root.join("rust_plugin");

    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "rust",
            "rustflow",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );

    run_ok_json(
        &[
            "cli",
            "plugins",
            "install",
            manifest_file(&scaffold_dir).to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"].as_array().expect("plugins array").iter().any(
        |item| item["manifest"]["namespace"] == "rustflow"
    ));

    let uninstall = run_ok_json(&["cli", "plugins", "uninstall", "rustflow"], &plugins_dir);
    assert_eq!(uninstall["status"], "uninstalled");
}

#[test]
fn scaffold_rejects_unsafe_path_reserved_namespace_and_existing_path_without_force() {
    let root = tmp_dir("scaffold-failures");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let unsafe_out = run(
        &["cli", "plugins", "scaffold", "python", "unsafeplug", "--path", "../unsafe"],
        &plugins_dir,
    );
    assert_eq!(unsafe_out.status.code(), Some(1));

    let reserved_out = run(&["cli", "plugins", "scaffold", "python", "cli"], &plugins_dir);
    assert_eq!(reserved_out.status.code(), Some(1));

    let existing = root.join("existing");
    fs::create_dir_all(&existing).expect("mkdir existing");
    let existing_out = run(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "existingplug",
            "--path",
            existing.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    assert_eq!(existing_out.status.code(), Some(1));
}

#[test]
fn install_rejects_invalid_missing_reserved_and_duplicate_manifest_cases() {
    let root = tmp_dir("install-failures");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");

    let invalid_manifest = root.join("invalid.json");
    fs::write(&invalid_manifest, "{not-json").expect("write invalid manifest");
    let invalid_out = run(
        &["cli", "plugins", "install", invalid_manifest.to_str().expect("utf-8")],
        &plugins_dir,
    );
    assert_eq!(invalid_out.status.code(), Some(1));

    let missing_entrypoint = root.join("missing-entrypoint.json");
    fs::write(
        &missing_entrypoint,
        r#"{
  "name": "broken",
  "version": "0.1.0",
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {"min_inclusive":"0.1.0", "max_exclusive": null},
  "namespace": "broken",
  "kind": "python",
  "aliases": [],
  "entrypoint": "",
  "capabilities": []
}"#,
    )
    .expect("write missing entrypoint manifest");
    let missing_out = run(
        &["cli", "plugins", "install", missing_entrypoint.to_str().expect("utf-8")],
        &plugins_dir,
    );
    assert_eq!(missing_out.status.code(), Some(1));

    let reserved_manifest = root.join("reserved.json");
    fs::write(
        &reserved_manifest,
        r#"{
  "name": "reserved",
  "version": "0.1.0",
  "schema_version": "v1",
  "manifest_version": "v1",
  "compatibility": {"min_inclusive":"0.1.0", "max_exclusive": null},
  "namespace": "cli",
  "kind": "python",
  "aliases": [],
  "entrypoint": "plugin:main",
  "capabilities": []
}"#,
    )
    .expect("write reserved manifest");
    let reserved_out = run(
        &["cli", "plugins", "install", reserved_manifest.to_str().expect("utf-8")],
        &plugins_dir,
    );
    assert_eq!(reserved_out.status.code(), Some(1));

    let scaffold_dir = root.join("dup-scaffold");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "dupplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    let manifest = manifest_file(&scaffold_dir);
    let first = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert!(first.status.success());
    let duplicate = run(&["cli", "plugins", "install", manifest.to_str().expect("utf-8")], &plugins_dir);
    assert_eq!(duplicate.status.code(), Some(1));
}

#[test]
fn uninstall_failure_preserves_existing_registry_entries() {
    let root = tmp_dir("uninstall-rollback");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    let scaffold_dir = root.join("installed");
    run_ok_json(
        &[
            "cli",
            "plugins",
            "scaffold",
            "python",
            "keepplug",
            "--path",
            scaffold_dir.to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );
    run_ok_json(
        &[
            "cli",
            "plugins",
            "install",
            manifest_file(&scaffold_dir).to_str().expect("utf-8"),
        ],
        &plugins_dir,
    );

    let fail_uninstall = run(&["cli", "plugins", "uninstall", "missing"], &plugins_dir);
    assert_eq!(fail_uninstall.status.code(), Some(1));

    let listed = run_ok_json(&["cli", "plugins", "list"], &plugins_dir);
    assert!(listed["plugins"].as_array().expect("plugins array").iter().any(
        |item| item["manifest"]["namespace"] == "keepplug"
    ));
}

#[test]
fn plugin_doctor_self_repairs_corrupt_registry_file() {
    let root = tmp_dir("doctor-repair");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins");
    fs::write(plugins_dir.join("registry.json"), "{broken-json").expect("write corrupt registry");

    let doctor = run_ok_json(&["cli", "plugins", "doctor"], &plugins_dir);
    assert_eq!(doctor["status"], "ok");
    assert_eq!(doctor["self_repair_attempted"], true);
    assert_eq!(doctor["self_repair_success"], true);
}
