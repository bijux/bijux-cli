#![forbid(unsafe_code)]
//! Regression tests for invalid names, namespace shadowing, and persistence behavior.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_plugin::{
    install_plugin, load_registry, InstallPluginRequest, PluginError, PluginTrustLevel,
};
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

fn temp_dir(label: &str) -> PathBuf {
    let ts =
        SystemTime::now().duration_since(UNIX_EPOCH).expect("clock should be monotonic").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-plugin-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("directory should be created");
    dir
}

fn manifest_text(namespace: &str, alias: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "{namespace}",
  "kind": "delegated",
  "aliases": ["{alias}"],
  "entrypoint": "{namespace}.plugin:run",
  "capabilities": [{{"name": "exec", "version": "1"}}]
}}"#
    )
}

#[test]
fn rejects_invalid_plugin_namespace_names() {
    let root = temp_dir("invalid-name");
    let registry_path = root.join("registry.json");

    let error = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("BadName", "bad-name"),
            source: "local:/tmp/bad".to_string(),
            trust_level: PluginTrustLevel::Unknown,
        },
        "0.1.0",
    )
    .expect_err("invalid namespace should fail");

    assert!(matches!(error, PluginError::InvalidNamespace(_)));
}

#[test]
fn rejects_plugins_shadowing_reserved_or_installed_namespaces() {
    let root = temp_dir("shadowing");
    let registry_path = root.join("registry.json");

    let reserved = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cli", "cli-alias"),
            source: "local:/tmp/cli".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("reserved namespace should fail");
    assert!(matches!(reserved, PluginError::ReservedNamespace(_)));

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("echo", "echo-alias"),
            source: "local:/tmp/echo".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("first plugin should install");

    let duplicate = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("echo", "echo-second"),
            source: "local:/tmp/echo2".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("duplicate namespace should fail");
    assert!(matches!(duplicate, PluginError::NamespaceConflict(_)));
}

#[test]
fn persists_plugin_registry_across_restarts() {
    let root = temp_dir("persistence");
    let registry_path = root.join("registry.json");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("persisted", "persisted-alias"),
            source: "local:/tmp/persisted".to_string(),
            trust_level: PluginTrustLevel::Verified,
        },
        "0.1.0",
    )
    .expect("install should succeed");

    let first_load = load_registry(&registry_path).expect("load after install should work");
    assert!(first_load.plugins.contains_key("persisted"));

    let second_load = load_registry(&registry_path).expect("load on restart should work");
    assert!(second_load.plugins.contains_key("persisted"));
}
