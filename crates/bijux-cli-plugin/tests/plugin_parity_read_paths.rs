#![forbid(unsafe_code)]
//! Read-only plugin parity and namespace policy regression tests.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_plugin::{
    compatibility_warnings, inspect_plugin, install_plugin, list_plugins, load_time_diagnostics,
    registry_path_from_plugins_dir, uninstall_plugin, InstallPluginRequest, PluginError,
    PluginTrustLevel,
};
use semver as _;
use serde as _;
use serde_json::{self, Value};
use sha2 as _;
use thiserror as _;

fn temp_plugins_dir(label: &str) -> PathBuf {
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be monotonic")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-plugin-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("directory should be created");
    dir
}

fn manifest_text(namespace: &str, kind: &str, entrypoint: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "{namespace}",
  "kind": "{kind}",
  "aliases": ["{namespace}-alias"],
  "entrypoint": "{entrypoint}",
  "capabilities": [{{"name": "exec", "version": "1"}}]
}}"#
    )
}

#[test]
fn plugins_list_parity_shape_matches_python_capture_baseline() {
    let lock = fs::read_to_string("../../artifacts/current-python-behavior-lock.json")
        .expect("python lock should exist");
    let captures: Value = serde_json::from_str(&lock).expect("valid lock json");
    let python_plugins = serde_json::from_str::<Value>(
        captures["captures"]["bijux_plugins_list"]["stdout"]
            .as_str()
            .expect("python plugins capture"),
    )
    .expect("python plugins output should parse");

    let plugins_dir = temp_plugins_dir("list-parity");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);
    let rust_plugins = list_plugins(&registry_path).expect("list should succeed for empty registry");

    assert!(python_plugins.get("plugins").is_some());
    assert!(rust_plugins.is_empty());
}

#[test]
fn plugins_inspect_parity_shape_contains_manifest_namespace_and_source() {
    let plugins_dir = temp_plugins_dir("inspect-parity");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("inspector", "delegated", "inspector.plugin:run"),
            source: "local:/tmp/inspector".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install should succeed");

    let inspected = inspect_plugin(&registry_path, "inspector").expect("inspect should succeed");
    assert_eq!(inspected.manifest.namespace.0, "inspector");
    assert_eq!(inspected.source, "local:/tmp/inspector");
}

#[test]
fn namespace_rejection_message_is_stable_and_machine_usable() {
    let plugins_dir = temp_plugins_dir("reserved");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    let error = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cli", "delegated", "cli.plugin:run"),
            source: "local:/tmp/cli".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("reserved namespace must fail");

    match error {
        PluginError::ReservedNamespace(value) => assert_eq!(value, "cli"),
        other => panic!("unexpected error kind: {other:?}"),
    }
}

#[test]
fn duplicate_namespace_collision_is_rejected_consistently() {
    let plugins_dir = temp_plugins_dir("collision");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("echo", "delegated", "echo.plugin:run"),
            source: "local:/tmp/echo".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("first install should succeed");

    let error = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("echo", "delegated", "echo.plugin:run"),
            source: "local:/tmp/echo-two".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("duplicate install should fail");

    assert!(matches!(error, PluginError::NamespaceConflict(_)));
}

#[test]
fn registry_layout_compatibility_matches_expected_python_style_path() {
    let plugins_dir = PathBuf::from("/tmp/example/.bijux/.plugins");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);
    assert_eq!(registry_path, PathBuf::from("/tmp/example/.bijux/.plugins/registry.json"));
}

#[test]
fn broken_metadata_and_missing_entrypoint_are_reported() {
    let plugins_dir = temp_plugins_dir("broken-metadata");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    // malformed json metadata should fail parse
    let malformed = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: "{not-json".to_string(),
            source: "local:/tmp/malformed".to_string(),
            trust_level: PluginTrustLevel::Unknown,
        },
        "0.1.0",
    )
    .expect_err("malformed json should fail");
    assert!(matches!(malformed, PluginError::ManifestParse(_)));

    // external executable missing on filesystem should be diagnosed
    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("external", "external-exec", "/tmp/not-existing-binary"),
            source: "local:/tmp/external".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("external plugin install should succeed");

    let diagnostics = load_time_diagnostics(&registry_path, "0.1.0").expect("diagnostics should run");
    assert!(diagnostics
        .iter()
        .any(|item| item.namespace == "external" && item.message.contains("entrypoint was not found")));
}

#[test]
fn incompatible_version_and_reserved_product_namespace_are_rejected() {
    let plugins_dir = temp_plugins_dir("incompatible");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    let incompatible = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("oldplugin", "delegated", "oldplugin.plugin:run"),
            source: "local:/tmp/oldplugin".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "9.9.9",
    )
    .expect_err("incompatible version should fail");
    assert!(matches!(incompatible, PluginError::IncompatibleVersion { .. }));

    let reserved_product = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("atlas", "delegated", "atlas.plugin:run"),
            source: "local:/tmp/atlas".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("official reserved product namespace should fail");
    assert!(matches!(reserved_product, PluginError::FutureNamespaceConflict(_)));
}

#[test]
fn install_uninstall_cycle_reinstalls_cleanly() {
    let plugins_dir = temp_plugins_dir("cycle");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cycleplugin", "delegated", "cycleplugin.plugin:run"),
            source: "local:/tmp/cycleplugin".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("initial install should succeed");

    uninstall_plugin(&registry_path, "cycleplugin").expect("uninstall should succeed");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cycleplugin", "delegated", "cycleplugin.plugin:run"),
            source: "local:/tmp/cycleplugin-v2".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("reinstall should succeed");

    let warnings = compatibility_warnings(&registry_path, "0.1.0").expect("warnings should load");
    assert!(warnings.is_empty());
}
