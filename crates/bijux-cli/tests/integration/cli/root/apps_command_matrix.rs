#![forbid(unsafe_code)]
//! Root app inventory and mount-resolution integration coverage.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use bijux_cli::contracts::known_bijux_tools;
use serde_json::Value;

fn temp_dir(name: &str) -> PathBuf {
    let root =
        env::temp_dir().join(format!("bijux-apps-command-matrix-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("mkdir temp");
    root
}

fn run_with(root: &Path, args: &[&str], envs: &[(&str, String)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bijux"));
    command.current_dir(root).args(args);
    for (key, value) in envs {
        command.env(key, value);
    }
    command.output().expect("binary should execute")
}

#[cfg(unix)]
fn write_stub_binary(bin_dir: &Path, binary_name: &str, version_line: &str) {
    use std::os::unix::fs::PermissionsExt;

    let script = format!(
        "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  printf '%s\\n' '{version_line}'\n  exit 0\nfi\nprintf 'stub:{binary_name}\\n'\nprintf 'args:%s\\n' \"$*\"\n"
    );
    let path = bin_dir.join(binary_name);
    fs::write(&path, script).expect("write stub");
    let mut permissions = fs::metadata(&path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&path, permissions).expect("chmod");
}

#[cfg(windows)]
fn write_stub_binary(bin_dir: &Path, binary_name: &str, version_line: &str) {
    let script = format!(
        "@echo off\r\nif \"%1\" == \"--version\" (\r\n  echo {version_line}\r\n  exit /b 0\r\n)\r\necho stub:{binary_name}\r\necho args:%*\r\n"
    );
    fs::write(bin_dir.join(format!("{binary_name}.bat")), script).expect("write stub");
}

fn write_all_stubs(bin_dir: &Path) {
    for tool in known_bijux_tools() {
        write_stub_binary(
            bin_dir,
            tool.runtime_binary_name,
            &format!("{} 9.9.9", tool.runtime_binary_name),
        );
        write_stub_binary(
            bin_dir,
            tool.control_binary_name,
            &format!("{} 9.9.9", tool.control_binary_name),
        );
    }
}

fn parse_json(out: Output) -> Value {
    assert_eq!(out.status.code(), Some(0), "command should succeed");
    assert!(out.stderr.is_empty(), "stderr should stay empty on success");
    serde_json::from_slice(&out.stdout).expect("stdout should be valid json")
}

#[test]
fn apps_list_reports_known_products_and_health_fields() {
    let root = temp_dir("apps-list");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_all_stubs(&bin_dir);

    let payload = parse_json(run_with(
        &root,
        &["apps", "list", "--format", "json", "--no-pretty"],
        &[("PATH", bin_dir.display().to_string())],
    ));

    assert!(payload["apps"].is_array());
    let apps = payload["apps"].as_array().expect("apps array");
    assert_eq!(apps.len(), known_bijux_tools().len());
    let dag = apps.iter().find(|row| row["namespace"] == "dag").expect("dag row");
    assert_eq!(dag["source"], "compiled_official_registry");
    assert_eq!(dag["entrypoint"], "bijux-dag");
    assert_eq!(dag["status"], "declared");
    assert_eq!(dag["health"], "ok");
    assert!(dag["resolved_entrypoint"].as_str().is_some_and(|value| value.contains("bijux-dag")));
}

#[test]
fn apps_which_resolves_exact_runtime_entrypoint_from_path() {
    let root = temp_dir("apps-which");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&bin_dir).expect("mkdir bin");
    write_stub_binary(&bin_dir, "bijux-dag", "bijux-dag 1.2.3");

    let payload = parse_json(run_with(
        &root,
        &["apps", "which", "dag", "--format", "json", "--no-pretty"],
        &[("PATH", bin_dir.display().to_string())],
    ));

    assert_eq!(payload["namespace"], "dag");
    assert_eq!(payload["matched_via"], "namespace");
    assert_eq!(payload["health"], "ok");
    assert!(payload["resolved_entrypoint"].as_str().is_some_and(
        |value| value.ends_with(if cfg!(windows) { "bijux-dag.bat" } else { "bijux-dag" })
    ));
}

#[test]
fn apps_version_uses_descriptor_manifest_without_runtime_probe() {
    let root = temp_dir("apps-version");
    let app_dir = root.join(".bijux/apps");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::write(
        app_dir.join("dag.mount.json"),
        r#"{
  "namespace": "dag",
  "version": "0.4.0-dev",
  "entrypoint": {
    "kind": "binary",
    "command": "missing-bijux-dag"
  }
}"#,
    )
    .expect("write descriptor");

    let payload = parse_json(run_with(
        &root,
        &["apps", "version", "dag", "--format", "json", "--no-pretty"],
        &[("PATH", root.join("empty-bin").display().to_string())],
    ));

    assert_eq!(payload["namespace"], "dag");
    assert_eq!(payload["matched_via"], "namespace");
    assert_eq!(payload["version"], "0.4.0-dev");
    assert_eq!(payload["source"], "manifest");
    assert_eq!(payload["health"], "missing");
}

#[test]
fn apps_capabilities_accept_declared_alias_queries() {
    let root = temp_dir("apps-capabilities");
    let payload = parse_json(run_with(
        &root,
        &["apps", "capabilities", "workflow", "--format", "json", "--no-pretty"],
        &[],
    ));

    assert_eq!(payload["namespace"], "dag");
    assert_eq!(payload["matched_via"], "alias");
    assert_eq!(payload["entrypoint_kind"], "binary");
    assert!(payload["capabilities"]
        .as_array()
        .expect("capabilities")
        .iter()
        .any(|value| value == "run"));
}

#[test]
fn official_runtime_delegation_prefers_project_descriptor_entrypoint() {
    let root = temp_dir("apps-delegation");
    let app_dir = root.join(".bijux/apps");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::create_dir_all(&bin_dir).expect("mkdir bin dir");
    write_stub_binary(&bin_dir, "custom-dag", "custom-dag 7.7.7");
    fs::write(
        app_dir.join("dag.mount.json"),
        r#"{
  "namespace": "dag",
  "entrypoint": {
    "kind": "binary",
    "command": "../../bin/custom-dag"
  },
  "version": "7.7.7"
}"#,
    )
    .expect("write descriptor");

    let out = run_with(
        &root,
        &["dag", "status"],
        &[("PATH", root.join("empty-bin").display().to_string())],
    );

    assert_eq!(out.status.code(), Some(0));
    assert!(out.stderr.is_empty());
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(stdout.contains("stub:custom-dag"));
    assert!(stdout.contains("args:status"));
}

#[test]
fn apps_disabled_registry_marks_mount_as_disabled_without_descriptor_override() {
    let root = temp_dir("apps-disabled-registry");
    let app_dir = root.join(".bijux/apps");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::write(app_dir.join("disabled.json"), r#"{"disabled":["workflow"]}"#)
        .expect("write disabled registry");

    let payload = parse_json(run_with(
        &root,
        &["apps", "which", "dag", "--format", "json", "--no-pretty"],
        &[],
    ));

    assert_eq!(payload["namespace"], "dag");
    assert_eq!(payload["health"], "disabled");
    assert_eq!(payload["resolution_policy"], "standard_precedence");
}

#[test]
fn apps_doctor_reports_shadowed_plugin_conflicts_for_official_namespace() {
    let root = temp_dir("apps-conflict");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("mkdir plugins dir");
    fs::write(
        plugins_dir.join("registry.json"),
        r#"{
  "version": "1",
  "plugins": {
    "dag-shadow": {
      "manifest": {
        "name": "dag-shadow",
        "version": "0.1.0",
        "schema_version": "v2",
        "manifest_version": "v2",
        "compatibility": { "min_inclusive": "0.1.0", "max_exclusive": "1.0.0" },
        "namespace": "workflow",
        "kind": "external-exec",
        "aliases": [],
        "entrypoint": "workflow-plugin",
        "capabilities": []
      },
      "state": "enabled",
      "source": "test-fixture",
      "trust_level": "community",
      "manifest_checksum_sha256": "fixture"
    }
  }
}"#,
    )
    .expect("write plugin registry");

    let payload = parse_json(run_with(
        &root,
        &["apps", "doctor", "--format", "json", "--no-pretty"],
        &[("BIJUXCLI_PLUGINS_DIR", plugins_dir.display().to_string())],
    ));

    let apps = payload["apps"].as_array().expect("apps array");
    let dag = apps.iter().find(|row| row["namespace"] == "dag").expect("dag row");
    assert_eq!(dag["health"], "conflict");
    assert_eq!(dag["resolution_policy"], "official_wins");
    assert_eq!(dag["shadowed_plugins"], serde_json::json!(["workflow"]));
    assert!(dag["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|value| value.as_str().is_some_and(|text| text.contains("conflicting plugin namespaces"))));
}

#[test]
fn project_local_mount_json_takes_precedence_over_legacy_json_override() {
    let root = temp_dir("apps-mount-precedence");
    let app_dir = root.join(".bijux/apps");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::create_dir_all(&bin_dir).expect("mkdir bin dir");
    write_stub_binary(&bin_dir, "preferred-dag", "preferred-dag 3.0.0");
    write_stub_binary(&bin_dir, "legacy-dag", "legacy-dag 2.0.0");
    fs::write(
        app_dir.join("dag.mount.json"),
        r#"{
  "namespace": "dag",
  "entrypoint": {
    "kind": "binary",
    "command": "../../bin/preferred-dag"
  },
  "version": "3.0.0"
}"#,
    )
    .expect("write preferred descriptor");
    fs::write(
        app_dir.join("dag.json"),
        r#"{
  "namespace": "dag",
  "entrypoint": {
    "kind": "binary",
    "command": "../../bin/legacy-dag"
  },
  "version": "2.0.0"
}"#,
    )
    .expect("write legacy descriptor");

    let payload = parse_json(run_with(
        &root,
        &["apps", "which", "dag", "--format", "json", "--no-pretty"],
        &[("PATH", root.join("empty-bin").display().to_string())],
    ));

    assert_eq!(payload["source"], "project_local");
    assert!(payload["resolved_entrypoint"]
        .as_str()
        .is_some_and(|value| value.contains("preferred-dag")));
}

#[test]
fn project_local_python_module_mount_delegates_through_configured_interpreter() {
    let root = temp_dir("apps-python-module");
    let app_dir = root.join(".bijux/apps");
    let bin_dir = root.join("bin");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::create_dir_all(&bin_dir).expect("mkdir bin dir");
    write_stub_binary(&bin_dir, "fake-python", "fake-python 3.11.0");
    fs::write(
        app_dir.join("dag.mount.json"),
        r#"{
  "namespace": "dag",
  "entrypoint": {
    "kind": "python_module",
    "command": "bijux_dag_adapter"
  }
}"#,
    )
    .expect("write descriptor");

    let out = run_with(
        &root,
        &["dag", "validate", "graph.json"],
        &[
            ("PATH", root.join("empty-bin").display().to_string()),
            ("BIJUX_PYTHON_BIN", bin_dir.join(if cfg!(windows) { "fake-python.bat" } else { "fake-python" }).display().to_string()),
        ],
    );

    assert_eq!(out.status.code(), Some(0));
    let stdout = String::from_utf8(out.stdout).expect("utf-8");
    assert!(stdout.contains("stub:fake-python"));
    assert!(stdout.contains("args:-m bijux_dag_adapter validate graph.json"));
}

#[test]
fn project_local_embedded_mount_handles_status_and_help_without_external_binary() {
    let root = temp_dir("apps-embedded");
    let app_dir = root.join(".bijux/apps");
    fs::create_dir_all(&app_dir).expect("mkdir app dir");
    fs::write(
        app_dir.join("dag.mount.json"),
        r#"{
  "namespace": "dag",
  "help": { "summary": "Embedded DAG shell" },
  "version": "0.9.0-embedded",
  "entrypoint": {
    "kind": "embedded_rust",
    "command": "descriptor-shell"
  }
}"#,
    )
    .expect("write descriptor");

    let status = run_with(&root, &["dag", "status"], &[]);
    assert_eq!(status.status.code(), Some(0));
    let status_payload: Value = serde_json::from_slice(&status.stdout).expect("json");
    assert_eq!(status_payload["namespace"], "dag");
    assert_eq!(status_payload["mode"], "embedded_rust");

    let help = run_with(&root, &["dag", "--help"], &[]);
    assert_eq!(help.status.code(), Some(0));
    let help_stdout = String::from_utf8(help.stdout).expect("utf-8");
    assert!(help_stdout.contains("Usage: bijux dag"));
    assert!(help_stdout.contains("Embedded DAG shell"));
}
