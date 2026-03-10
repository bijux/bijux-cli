#![forbid(unsafe_code)]
//! Write-path maturity tests: transactional behavior, rollback, and lifecycle edge cases.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_plugin::{
    disable_plugin, enable_plugin, install_plugin, list_plugins, registry_path_from_plugins_dir,
    uninstall_plugin, InstallPluginRequest, PluginError, PluginTrustLevel,
};
use bijux_cli_routing as _;
use bijux_cli_routing::PluginLifecycleState;
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

fn temp_plugins_dir(label: &str) -> PathBuf {
    let ts =
        SystemTime::now().duration_since(UNIX_EPOCH).expect("clock should be monotonic").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-plugin-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("directory should be created");
    dir
}

fn manifest_text(namespace: &str, version: &str) -> String {
    format!(
        r#"{{
  "name": "{namespace}",
  "version": "{version}",
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

fn install_example(registry_path: &PathBuf, namespace: &str, version: &str) {
    install_plugin(
        registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text(namespace, version),
            source: format!("local:/tmp/{namespace}-{version}"),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install should succeed");
}

#[test]
fn install_and_uninstall_are_transaction_safe_and_cleanup_backup_files() {
    let plugins_dir = temp_plugins_dir("transaction-safety");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "transact", "1.0.0");
    assert!(registry_path.exists());
    assert!(!registry_path.with_extension("bak").exists());

    uninstall_plugin(&registry_path, "transact").expect("uninstall should succeed");
    assert!(!registry_path.with_extension("bak").exists());
}

#[test]
fn failed_install_rolls_back_and_preserves_existing_plugin_list() {
    let plugins_dir = temp_plugins_dir("install-rollback");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "stable", "1.0.0");

    let error = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: "{broken-json".to_string(),
            source: "local:/tmp/broken".to_string(),
            trust_level: PluginTrustLevel::Unknown,
        },
        "0.1.0",
    )
    .expect_err("invalid manifest must fail");
    assert!(matches!(error, PluginError::ManifestParse(_)));

    let listed = list_plugins(&registry_path).expect("list should still work");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest.namespace.0, "stable");
}

#[test]
fn failed_uninstall_rolls_back_and_keeps_registry_unchanged() {
    let plugins_dir = temp_plugins_dir("uninstall-rollback");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "stable", "1.0.0");

    let error = uninstall_plugin(&registry_path, "missing")
        .expect_err("missing plugin uninstall must fail");
    assert!(matches!(error, PluginError::PluginNotFound(_)));

    let listed = list_plugins(&registry_path).expect("list should still work");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest.namespace.0, "stable");
}

#[test]
fn reinstall_upgrade_and_downgrade_without_uninstall_are_rejected_consistently() {
    let plugins_dir = temp_plugins_dir("version-change");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "cycle", "1.0.0");

    let upgrade = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cycle", "1.1.0"),
            source: "local:/tmp/cycle-1.1.0".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("upgrade without explicit uninstall is unsupported");
    assert!(matches!(upgrade, PluginError::NamespaceConflict(_)));

    let downgrade = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cycle", "0.9.0"),
            source: "local:/tmp/cycle-0.9.0".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("downgrade without explicit uninstall is unsupported");
    assert!(matches!(downgrade, PluginError::NamespaceConflict(_)));
}

#[test]
fn enabling_broken_plugin_is_rejected_and_disabling_missing_plugin_fails_cleanly() {
    let plugins_dir = temp_plugins_dir("broken-enable");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "recoverable", "1.0.0");

    disable_plugin(&registry_path, "recoverable").expect("disable should succeed");

    // Corrupt lifecycle intentionally to emulate a broken plugin recorded in registry.
    let mut registry = bijux_cli_plugin::load_registry(&registry_path).expect("load registry");
    registry.plugins.get_mut("recoverable").expect("plugin exists").state =
        PluginLifecycleState::Broken;
    bijux_cli_plugin::save_registry(&registry_path, &registry).expect("save registry");

    let enable_error = enable_plugin(&registry_path, "recoverable")
        .expect_err("broken plugin must not be enabled");
    assert!(matches!(enable_error, PluginError::InvalidField(_)));

    let disable_error =
        disable_plugin(&registry_path, "missing").expect_err("missing plugin disable should fail");
    assert!(matches!(disable_error, PluginError::PluginNotFound(_)));
}

#[test]
fn failed_install_does_not_pollute_listing_results() {
    let plugins_dir = temp_plugins_dir("failed-install-listing");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    install_example(&registry_path, "healthy", "1.0.0");

    let _ = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("cli", "1.0.0"),
            source: "local:/tmp/cli-reserved".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("reserved namespace install should fail");

    let listed = list_plugins(&registry_path).expect("list should succeed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0].manifest.namespace.0, "healthy");
}
