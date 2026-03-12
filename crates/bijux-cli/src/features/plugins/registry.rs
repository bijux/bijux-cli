#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::constants::REGISTRY_VERSION;
use super::errors::PluginError;
use super::manifest::{is_version_compatible, parse_manifest_v1, validate_manifest};
use super::models::{
    InstallPluginRequest, PluginDoctorReport, PluginLoadEntry, PluginOriginMetadata, PluginRecord,
    PluginRegistry,
};

fn checksum_sha256(input: &str) -> String {
    let digest = Sha256::digest(input.as_bytes());
    format!("{digest:x}")
}

/// Load plugin registry from disk.
pub fn load_registry(path: &Path) -> Result<PluginRegistry, PluginError> {
    if !path.exists() {
        return Ok(PluginRegistry::default());
    }

    let text = fs::read_to_string(path)?;
    let parsed: PluginRegistry =
        serde_json::from_str(&text).map_err(|_| PluginError::RegistryCorrupted)?;
    if parsed.version != REGISTRY_VERSION {
        return Err(PluginError::RegistryCorrupted);
    }
    Ok(parsed)
}

/// Save plugin registry atomically.
pub fn save_registry(path: &Path, registry: &PluginRegistry) -> Result<(), PluginError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let data = serde_json::to_vec_pretty(registry)?;
    let temporary = path.with_extension("tmp");

    {
        let mut file =
            OpenOptions::new().create(true).truncate(true).write(true).open(&temporary)?;
        file.write_all(&data)?;
        file.write_all(b"\n")?;
        file.sync_all()?;
    }

    fs::rename(temporary, path)?;
    Ok(())
}

fn backup_registry(path: &Path) -> Result<Option<PathBuf>, PluginError> {
    if !path.exists() {
        return Ok(None);
    }
    let backup = path.with_extension("bak");
    fs::copy(path, &backup)?;
    Ok(Some(backup))
}

fn restore_registry(path: &Path, backup: Option<PathBuf>) -> Result<(), PluginError> {
    if let Some(backup_path) = backup {
        fs::rename(backup_path, path)?;
    }
    Ok(())
}

fn cleanup_backup(backup: Option<PathBuf>) {
    if let Some(path) = backup {
        let _ = fs::remove_file(path);
    }
}

/// Update plugin registry atomically with rollback support.
pub fn update_registry<F>(path: &Path, mutator: F) -> Result<PluginRegistry, PluginError>
where
    F: FnOnce(&mut PluginRegistry) -> Result<(), PluginError>,
{
    let backup = backup_registry(path)?;
    let mut registry = load_registry(path)?;

    if let Err(error) = mutator(&mut registry) {
        restore_registry(path, backup)?;
        return Err(error);
    }

    if let Err(error) = save_registry(path, &registry) {
        restore_registry(path, backup)?;
        return Err(error);
    }

    cleanup_backup(backup);
    Ok(registry)
}

fn ensure_aliases_do_not_conflict(
    registry: &PluginRegistry,
    candidate: &PluginRecord,
) -> Result<(), PluginError> {
    let mut existing_aliases = BTreeSet::new();
    for plugin in registry.plugins.values() {
        for alias in &plugin.manifest.aliases {
            existing_aliases.insert(alias.to_ascii_lowercase());
        }
    }

    for alias in &candidate.manifest.aliases {
        if existing_aliases.contains(&alias.to_ascii_lowercase()) {
            return Err(PluginError::AliasConflict(alias.clone()));
        }
    }

    Ok(())
}

/// Install plugin into registry from manifest text.
pub fn install_plugin(
    registry_path: &Path,
    request: InstallPluginRequest,
    host_version: &str,
    reserved_namespaces: &[&str],
) -> Result<PluginRecord, PluginError> {
    let manifest_checksum_sha256 = checksum_sha256(&request.manifest_text);
    let manifest = parse_manifest_v1(&request.manifest_text)?;
    let validated = validate_manifest(manifest, host_version, reserved_namespaces)?;

    let namespace = validated.manifest.namespace.0.clone();
    let source = request.source;
    let trust_level = request.trust_level;
    let record = PluginRecord {
        manifest: validated.manifest,
        state: crate::contracts::PluginLifecycleState::Installed,
        source,
        trust_level,
        manifest_checksum_sha256,
    };

    update_registry(registry_path, |registry| {
        if registry.plugins.contains_key(&namespace) {
            return Err(PluginError::NamespaceConflict(namespace.clone()));
        }
        ensure_aliases_do_not_conflict(registry, &record)?;
        registry.plugins.insert(namespace.clone(), record.clone());
        Ok(())
    })?;

    Ok(record)
}

/// Remove plugin from registry.
pub fn uninstall_plugin(registry_path: &Path, namespace: &str) -> Result<(), PluginError> {
    update_registry(registry_path, |registry| {
        if registry.plugins.remove(namespace).is_none() {
            return Err(PluginError::PluginNotFound(namespace.to_string()));
        }
        Ok(())
    })?;
    Ok(())
}

fn set_plugin_state(
    registry_path: &Path,
    namespace: &str,
    state: crate::contracts::PluginLifecycleState,
) -> Result<PluginRecord, PluginError> {
    let mut updated: Option<PluginRecord> = None;

    update_registry(registry_path, |registry| {
        let plugin = registry
            .plugins
            .get_mut(namespace)
            .ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))?;
        if state == crate::contracts::PluginLifecycleState::Enabled
            && plugin.state == crate::contracts::PluginLifecycleState::Broken
        {
            return Err(PluginError::InvalidField("cannot enable broken plugin".to_string()));
        }
        plugin.state = state;
        updated = Some(plugin.clone());
        Ok(())
    })?;

    updated.ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))
}

/// Enable installed plugin.
pub fn enable_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    set_plugin_state(registry_path, namespace, crate::contracts::PluginLifecycleState::Enabled)
}

/// Disable installed plugin.
pub fn disable_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    set_plugin_state(registry_path, namespace, crate::contracts::PluginLifecycleState::Disabled)
}

/// Inspect plugin by namespace.
pub fn inspect_plugin(registry_path: &Path, namespace: &str) -> Result<PluginRecord, PluginError> {
    let registry = load_registry(registry_path)?;
    registry
        .plugins
        .get(namespace)
        .cloned()
        .ok_or_else(|| PluginError::PluginNotFound(namespace.to_string()))
}

/// Build plugin-origin metadata from registry contents.
pub fn plugin_origin_metadata(
    registry_path: &Path,
) -> Result<Vec<PluginOriginMetadata>, PluginError> {
    let registry = load_registry(registry_path)?;
    Ok(registry
        .plugins
        .into_iter()
        .map(|(namespace, record)| PluginOriginMetadata {
            namespace,
            source: record.source,
            trust_level: record.trust_level,
        })
        .collect())
}

/// List all plugins deterministically by namespace.
pub fn list_plugins(registry_path: &Path) -> Result<Vec<PluginRecord>, PluginError> {
    let registry = load_registry(registry_path)?;
    Ok(registry.plugins.into_values().collect())
}

/// Produce plugin health report.
pub fn plugin_doctor(registry_path: &Path) -> Result<PluginDoctorReport, PluginError> {
    let registry = load_registry(registry_path)?;

    let mut broken = Vec::new();
    let mut incompatible = Vec::new();

    for (namespace, plugin) in &registry.plugins {
        if plugin.state == crate::contracts::PluginLifecycleState::Broken {
            broken.push(namespace.clone());
        }
        if plugin.state == crate::contracts::PluginLifecycleState::Incompatible {
            incompatible.push(namespace.clone());
        }
    }

    Ok(PluginDoctorReport { installed: registry.plugins.len(), broken, incompatible })
}

/// Check plugin compatibility against host version without mutating registry.
#[allow(dead_code)]
pub fn compatibility_check(
    manifest: &crate::contracts::PluginManifestV1,
    host_version: &str,
) -> Result<bool, PluginError> {
    let _ = semver::VersionReq::parse(&format!("={host_version}"))
        .map_err(|_| PluginError::InvalidField("host_version".to_string()))?;
    is_version_compatible(&manifest.compatibility, host_version)
}

/// Return deterministic plugin load order contract.
#[allow(dead_code)]
pub fn plugin_load_order(registry_path: &Path) -> Result<Vec<PluginLoadEntry>, PluginError> {
    let registry = load_registry(registry_path)?;
    let mut items: Vec<PluginLoadEntry> = registry
        .plugins
        .iter()
        .map(|(namespace, record)| PluginLoadEntry {
            namespace: namespace.clone(),
            state: record.state,
        })
        .collect();

    items.sort_by(|left, right| {
        let left_rank = state_rank(left.state);
        let right_rank = state_rank(right.state);
        left_rank.cmp(&right_rank).then_with(|| left.namespace.cmp(&right.namespace))
    });

    Ok(items)
}

#[allow(dead_code)]
fn state_rank(state: crate::contracts::PluginLifecycleState) -> u8 {
    match state {
        crate::contracts::PluginLifecycleState::Enabled => 0,
        crate::contracts::PluginLifecycleState::Installed
        | crate::contracts::PluginLifecycleState::Validated => 1,
        crate::contracts::PluginLifecycleState::Disabled => 2,
        crate::contracts::PluginLifecycleState::Discovered => 3,
        crate::contracts::PluginLifecycleState::Incompatible => 4,
        crate::contracts::PluginLifecycleState::Broken => 5,
    }
}
