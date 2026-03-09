#![forbid(unsafe_code)]
//! Registry and command internals coverage for plugin lifecycle operations.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_contracts::PluginLifecycleState;
use bijux_cli_core as _;
use bijux_cli_plugin::{
    compatibility_check, disable_plugin, enable_plugin, inspect_plugin, install_plugin,
    list_plugins, load_registry, plugin_doctor, registry_path_from_plugins_dir, uninstall_plugin,
    InstallPluginRequest,
};
use semver as _;
use serde as _;
use serde_json as _;
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

fn manifest_text(namespace: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "1.0.0",
  "schema_version": "1",
  "manifest_version": "1",
  "compatibility": {{ "min_inclusive": "0.1.0", "max_exclusive": "2.0.0" }},
  "namespace": "{namespace}",
  "kind": "delegated",
  "aliases": ["{namespace}-alias"],
  "entrypoint": "{namespace}.plugin:run",
  "capabilities": [{{"name": "exec", "version": "1"}}]
}}"#
    )
}

#[test]
fn install_enable_disable_inspect_list_and_uninstall_plugin() {
    let plugins_dir = temp_plugins_dir("lifecycle");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    let installed = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("community"),
            source: "local:/tmp/community".to_string(),
        },
        "0.1.0",
    )
    .expect("plugin should install");
    assert_eq!(installed.state, PluginLifecycleState::Installed);

    let enabled = enable_plugin(&registry_path, "community").expect("plugin should enable");
    assert_eq!(enabled.state, PluginLifecycleState::Enabled);

    let disabled = disable_plugin(&registry_path, "community").expect("plugin should disable");
    assert_eq!(disabled.state, PluginLifecycleState::Disabled);

    let inspected = inspect_plugin(&registry_path, "community").expect("inspect should work");
    assert_eq!(inspected.manifest.namespace.0, "community");

    let listed = list_plugins(&registry_path).expect("list should work");
    assert_eq!(listed.len(), 1);

    uninstall_plugin(&registry_path, "community").expect("uninstall should work");
    let listed_after = list_plugins(&registry_path).expect("list should work");
    assert!(listed_after.is_empty());
}

#[test]
fn compatibility_and_doctor_are_reported() {
    let plugins_dir = temp_plugins_dir("doctor");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    let installed = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("inspector"),
            source: "local:/tmp/inspector".to_string(),
        },
        "0.1.0",
    )
    .expect("plugin should install");

    let is_compatible = compatibility_check(&installed.manifest, "0.1.0").expect("check should pass");
    assert!(is_compatible);

    let report = plugin_doctor(&registry_path).expect("doctor should run");
    assert_eq!(report.installed, 1);
    assert!(report.broken.is_empty());
    assert!(report.incompatible.is_empty());

    let registry = load_registry(&registry_path).expect("registry should load");
    assert_eq!(registry.version, "1");
}
