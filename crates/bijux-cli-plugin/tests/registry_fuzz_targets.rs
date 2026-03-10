#![forbid(unsafe_code)]
//! Plugin registry fuzz targets for hydration and discovery disagreement handling.
//! test_type: registry-fuzz

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_plugin::{
    discover_plugin_manifests, install_plugin, load_registry, refresh_discovery_cache,
    InstallPluginRequest, PluginDiscoveryCache, PluginError, PluginTrustLevel,
};
use semver as _;
use serde as _;
use serde_json as _;
use sha2 as _;
use thiserror as _;

fn temp_dir(label: &str) -> PathBuf {
    let ts = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
    let dir = std::env::temp_dir().join(format!("bijux-registry-fuzz-{label}-{ts}"));
    fs::create_dir_all(&dir).expect("create temp dir");
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
fn fuzz_plugin_registry_hydration_is_stable_under_malformed_inputs() {
    let root = temp_dir("hydration");
    let path = root.join("registry.json");

    let corpus = [
        "",
        "{}",
        "{broken-json",
        "[]",
        "{\"version\":\"1\",\"plugins\":{}}",
        "{\"version\":\"999\",\"plugins\":{}}",
        "{\"version\":\"1\",\"plugins\":{\"x\":{}}}",
    ];

    for sample in corpus {
        fs::write(&path, sample).expect("write sample");
        let a = load_registry(&path);
        let b = load_registry(&path);
        assert_eq!(a.is_ok(), b.is_ok());
        if let Err(err) = a {
            assert!(matches!(err, PluginError::RegistryCorrupted | PluginError::Io(_)));
        }
    }
}

#[test]
fn fuzz_registry_discovery_disagreement_resolution_is_deterministic() {
    let root = temp_dir("disagreement");
    let registry_path = root.join("registry.json");
    let plugins_dir = root.join("plugins");
    fs::create_dir_all(&plugins_dir).expect("plugins dir");

    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("alpha", "alpha-a"),
            source: "local:/tmp/alpha".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install alpha");
    install_plugin(
        &registry_path,
        InstallPluginRequest {
            manifest_text: manifest_text("gamma", "gamma-a"),
            source: "local:/tmp/gamma".to_string(),
            trust_level: PluginTrustLevel::Community,
        },
        "0.1.0",
    )
    .expect("install gamma");

    for ns in ["alpha", "beta"] {
        let dir = plugins_dir.join(ns);
        fs::create_dir_all(&dir).expect("plugin dir");
        fs::write(dir.join("plugin.json"), manifest_text(ns, &format!("{ns}-alias")))
            .expect("write manifest");
    }

    let mut cache = PluginDiscoveryCache::default();
    refresh_discovery_cache(&mut cache, &plugins_dir).expect("refresh cache");
    let discovered: BTreeSet<String> = cache.manifests.keys().cloned().collect();

    let registry = load_registry(&registry_path).expect("load registry");
    let installed: BTreeSet<String> = registry.plugins.keys().cloned().collect();

    let missing_on_disk: Vec<String> = installed.difference(&discovered).cloned().collect();
    let missing_in_registry: Vec<String> = discovered.difference(&installed).cloned().collect();

    assert_eq!(missing_on_disk, vec!["gamma".to_string()]);
    assert_eq!(missing_in_registry, vec!["beta".to_string()]);

    let manifests = discover_plugin_manifests(&plugins_dir).expect("discover");
    let manifests_again = discover_plugin_manifests(&plugins_dir).expect("discover again");
    assert_eq!(manifests, manifests_again);
}

#[test]
fn fuzz_reserved_namespace_registry_loading_rejects_reserved_namespaces() {
    let root = temp_dir("reserved-reject");
    let registry_path = root.join("registry.json");

    for reserved in ["cli", "dev", "help", "doctor", "inspect"] {
        let err = install_plugin(
            &registry_path,
            InstallPluginRequest {
                manifest_text: manifest_text(reserved, &format!("{reserved}-alias")),
                source: format!("local:/tmp/{reserved}"),
                trust_level: PluginTrustLevel::Community,
            },
            "0.1.0",
        )
        .expect_err("reserved namespace must fail");
        assert!(matches!(err, PluginError::ReservedNamespace(_)));
    }
}
