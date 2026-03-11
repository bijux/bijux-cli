#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use super::errors::PluginError;
use super::manifest::parse_manifest_v1;
use super::models::PluginDiscoveryCache;

/// Scan plugin directory tree for manifests at `<plugin-dir>/*/plugin.json`.
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

        let manifest_path = candidate_dir.join("plugin.json");
        if manifest_path.exists() {
            manifests.push(manifest_path);
        }
    }
    manifests.sort();
    Ok(manifests)
}

/// Refresh discovery cache from plugin directory scan.
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
