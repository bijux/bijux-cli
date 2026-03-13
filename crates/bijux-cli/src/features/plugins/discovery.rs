#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::errors::PluginError;
use super::manifest::parse_manifest_v1;
use super::models::PluginDiscoveryCache;

/// Scan plugin directory tree for manifests at `<plugin-dir>/*/plugin.manifest.json`.
#[allow(dead_code)]
pub fn discover_plugin_manifests(plugins_dir: &Path) -> Result<Vec<PathBuf>, PluginError> {
    if !plugins_dir.exists() {
        return Ok(Vec::new());
    }

    let mut manifests = Vec::new();
    for entry in fs::read_dir(plugins_dir)? {
        let entry = entry?;
        let candidate_dir = entry.path();
        if !candidate_dir.is_dir() {
            continue;
        }

        let manifest_path = candidate_dir.join("plugin.manifest.json");
        if manifest_path.exists() {
            manifests.push(manifest_path);
        }
    }
    manifests.sort();
    Ok(manifests)
}

/// Refresh discovery cache from plugin directory scan.
#[allow(dead_code)]
pub fn refresh_discovery_cache(
    cache: &mut PluginDiscoveryCache,
    plugins_dir: &Path,
) -> Result<(), PluginError> {
    let discovered = discover_plugin_manifests(plugins_dir)?;
    let mut manifests = BTreeMap::new();

    for manifest_path in discovered {
        let text = fs::read_to_string(&manifest_path)?;
        let manifest = parse_manifest_v1(&text)?;
        manifests.insert(manifest.namespace.0, manifest_path);
    }

    cache.root = plugins_dir.to_path_buf();
    cache.manifests = manifests;
    cache.last_updated_millis =
        SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |duration| duration.as_millis());
    Ok(())
}

/// Parse registry path from the plugin directory.
#[must_use]
pub fn registry_path_from_plugins_dir(plugins_dir: &Path) -> PathBuf {
    plugins_dir.join("registry.json")
}

#[cfg(test)]
mod tests {
    use super::discover_plugin_manifests;

    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> std::path::PathBuf {
        let nonce = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let dir = std::env::temp_dir().join(format!("bijux-plugin-discovery-{label}-{nonce}"));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn discovers_current_manifest_filename_only() {
        let root = temp_dir("manifest");
        let modern = root.join("modern");
        let legacy = root.join("legacy");
        fs::create_dir_all(&modern).expect("create modern plugin dir");
        fs::create_dir_all(&legacy).expect("create legacy plugin dir");
        fs::write(modern.join("plugin.manifest.json"), "{}").expect("write modern manifest");
        fs::write(legacy.join("plugin.json"), "{}").expect("write legacy manifest");

        let discovered = discover_plugin_manifests(&root).expect("discover manifests");

        assert_eq!(discovered, vec![modern.join("plugin.manifest.json")]);
    }
}
