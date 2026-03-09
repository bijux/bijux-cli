#![forbid(unsafe_code)]
//! Resilience checks: deterministic discovery and partial-write recovery.

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_contracts as _;
use bijux_cli_plugin::{
    discover_plugin_manifests, refresh_discovery_cache, registry_path_from_plugins_dir,
    self_repair_registry, PluginDiscoveryCache,
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

#[test]
fn deterministic_discovery_order_is_stable_independent_of_creation_order() {
    let plugins_dir = temp_dir("discovery-order");

    for name in ["zeta", "alpha", "gamma"] {
        let dir = plugins_dir.join(name);
        fs::create_dir_all(&dir).expect("plugin dir should be created");
        fs::write(
            dir.join("plugin.json"),
            format!(
                r#"{{
  "name":"{name}","version":"1.0.0","schema_version":"1","manifest_version":"1",
  "compatibility":{{"min_inclusive":"0.1.0","max_exclusive":"2.0.0"}},
  "namespace":"{name}","kind":"delegated","aliases":[],"entrypoint":"{name}.plugin:run","capabilities":[]
}}"#
            ),
        )
        .expect("manifest should be written");
    }

    let manifests = discover_plugin_manifests(&plugins_dir).expect("discovery should succeed");
    let names = manifests
        .iter()
        .map(|path| {
            path.parent()
                .expect("manifest parent")
                .file_name()
                .expect("name")
                .to_string_lossy()
                .to_string()
        })
        .collect::<Vec<_>>();

    assert_eq!(names, vec!["alpha", "gamma", "zeta"]);

    let mut cache = PluginDiscoveryCache::default();
    refresh_discovery_cache(&mut cache, &plugins_dir).expect("cache refresh should succeed");
    let keys = cache.manifests.keys().cloned().collect::<Vec<_>>();
    assert_eq!(keys, vec!["alpha", "gamma", "zeta"]);
}

#[test]
fn registry_self_repair_recovers_from_partial_write() {
    let plugins_dir = temp_dir("partial-write");
    let registry_path = registry_path_from_plugins_dir(&plugins_dir);

    fs::write(&registry_path, "{\"version\": \"1\", \"plugins\": {")
        .expect("partial write should succeed");

    let repaired = self_repair_registry(&registry_path).expect("repair should succeed");
    assert!(repaired.plugins.is_empty());

    let repaired_text = fs::read_to_string(&registry_path).expect("repaired registry should exist");
    let parsed: serde_json::Value =
        serde_json::from_str(&repaired_text).expect("repaired json should parse");
    assert_eq!(parsed["version"], "1");
}
