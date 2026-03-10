#![forbid(unsafe_code)]
//! Discovery, ordering, collision, rollback, and diagnostics tests.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_plugin::{
    disable_plugin, discover_plugin_manifests, enable_plugin, install_plugin,
    load_time_diagnostics, plugin_load_order, refresh_discovery_cache, self_repair_registry,
    uninstall_plugin, InstallPluginRequest, PluginDiscoveryCache, PluginError, PluginTrustLevel,
};
use bijux_cli_routing as _;
use bijux_cli_routing::PluginLifecycleState;
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
fn detects_alias_conflicts_and_keeps_registry_consistent() {
    let root = temp_dir("alias-collision");
    let registry_path = root.join("registry.json");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("alpha", "shared-alias"),
            source: "local:/tmp/alpha".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("first install should succeed");

    let error = install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("beta", "shared-alias"),
            source: "local:/tmp/beta".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect_err("second install must fail due alias collision");
    assert!(matches!(error, PluginError::AliasConflict(_)));
}

#[test]
fn computes_deterministic_load_order() {
    let root = temp_dir("load-order");
    let registry_path = root.join("registry.json");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("gamma", "gamma-alias"),
            source: "local:/tmp/gamma".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install gamma");
    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("alpha", "alpha-alias"),
            source: "local:/tmp/alpha".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install alpha");

    enable_plugin(&registry_path, "gamma").expect("enable gamma");
    disable_plugin(&registry_path, "alpha").expect("disable alpha");

    let order = plugin_load_order(&registry_path).expect("load order should work");
    assert_eq!(order[0].namespace, "gamma");
    assert_eq!(order[0].state, PluginLifecycleState::Enabled);
}

#[test]
fn discovers_manifests_and_refreshes_cache() {
    let plugins_dir = temp_dir("discover");
    let alpha_dir = plugins_dir.join("alpha");
    fs::create_dir_all(&alpha_dir).expect("alpha dir");
    fs::write(alpha_dir.join("plugin.json"), manifest_text("alpha", "alpha-alias"))
        .expect("manifest should be written");

    let manifests = discover_plugin_manifests(&plugins_dir).expect("discovery should succeed");
    assert_eq!(manifests.len(), 1);

    let mut cache = PluginDiscoveryCache::default();
    refresh_discovery_cache(&mut cache, &plugins_dir).expect("cache refresh should succeed");
    assert!(cache.manifests.contains_key("alpha"));
    assert!(cache.last_updated_millis > 0);
}

#[test]
fn reports_load_time_diagnostics_and_repairs_corrupt_registry() {
    let root = temp_dir("diagnostics");
    let registry_path = root.join("registry.json");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("delta", "delta-alias"),
            source: "local:/tmp/delta".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install should succeed");

    let diagnostics =
        load_time_diagnostics(&registry_path, "9.9.9").expect("diagnostics should run");
    assert!(!diagnostics.is_empty());

    fs::write(&registry_path, "{broken-json").expect("corrupt write should succeed");
    let repaired = self_repair_registry(&registry_path).expect("repair should succeed");
    assert!(repaired.plugins.is_empty());

    uninstall_plugin(&registry_path, "missing").expect_err("missing uninstall should fail cleanly");
}
