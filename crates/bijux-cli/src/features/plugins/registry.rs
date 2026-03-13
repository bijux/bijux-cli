#![forbid(unsafe_code)]

use std::collections::BTreeSet;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::constants::REGISTRY_VERSION;
use super::diagnostics::load_time_diagnostics;
use super::entrypoint::{
    installed_manifest_root, is_executable, resolve_delegated_entrypoint,
    resolve_external_exec_entrypoint,
};
use super::errors::PluginError;
use super::manifest::{is_version_compatible, parse_manifest_v2, validate_manifest};
use super::models::{
    InstallPluginRequest, PluginDoctorReport, PluginLoadEntry, PluginOriginMetadata, PluginRecord,
    PluginRegistry,
};
use crate::api::version::runtime_semver;
use crate::contracts::PluginKind;
use crate::infrastructure::fs_store::atomic_write_text;

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

    let mut rendered = serde_json::to_string_pretty(registry)?;
    rendered.push('\n');
    atomic_write_text(path, &rendered)?;
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

#[derive(Debug)]
struct RegistryLockGuard {
    path: PathBuf,
}

impl Drop for RegistryLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

fn lock_path(path: &Path) -> PathBuf {
    path.with_extension("lock")
}

fn acquire_registry_lock(path: &Path) -> Result<RegistryLockGuard, PluginError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let lock = lock_path(path);
    match fs::OpenOptions::new().create_new(true).write(true).open(&lock) {
        Ok(mut file) => {
            let _ = writeln!(file, "pid={}", std::process::id());
            let _ = file.sync_all();
            Ok(RegistryLockGuard { path: lock })
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            Err(PluginError::RegistryLocked(lock))
        }
        Err(error) => Err(error.into()),
    }
}

fn restore_registry(path: &Path, backup: Option<PathBuf>) -> Result<(), PluginError> {
    if let Some(backup_path) = backup {
        replace_file(&backup_path, path)?;
    }
    Ok(())
}

fn cleanup_backup(backup: Option<PathBuf>) {
    if let Some(path) = backup {
        let _ = fs::remove_file(path);
    }
}

fn replace_file(source: &Path, destination: &Path) -> Result<(), std::io::Error> {
    #[cfg(windows)]
    {
        if destination.exists() {
            match fs::remove_file(destination) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error),
            }
        }
    }

    fs::rename(source, destination)
}

/// Update plugin registry atomically with rollback support.
pub fn update_registry<F>(path: &Path, mutator: F) -> Result<PluginRegistry, PluginError>
where
    F: FnOnce(&mut PluginRegistry) -> Result<(), PluginError>,
{
    let _lock = acquire_registry_lock(path)?;
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
    let mut existing_namespaces = BTreeSet::new();
    for plugin in registry.plugins.values() {
        existing_namespaces.insert(plugin.manifest.namespace.0.to_ascii_lowercase());
        for alias in &plugin.manifest.aliases {
            existing_aliases.insert(alias.to_ascii_lowercase());
        }
    }

    if existing_aliases.contains(&candidate.manifest.namespace.0.to_ascii_lowercase()) {
        return Err(PluginError::AliasConflict(candidate.manifest.namespace.0.clone()));
    }

    for alias in &candidate.manifest.aliases {
        let normalized = alias.to_ascii_lowercase();
        if existing_aliases.contains(&normalized) || existing_namespaces.contains(&normalized) {
            return Err(PluginError::AliasConflict(alias.clone()));
        }
    }

    Ok(())
}

fn resolve_namespace_reference(
    registry: &PluginRegistry,
    reference: &str,
) -> Result<String, PluginError> {
    let normalized = reference.to_ascii_lowercase();
    if let Some((namespace, _)) =
        registry.plugins.iter().find(|(namespace, _)| namespace.to_ascii_lowercase() == normalized)
    {
        return Ok(namespace.clone());
    }

    registry
        .plugins
        .iter()
        .find(|(_, record)| {
            record.manifest.aliases.iter().any(|alias| alias.to_ascii_lowercase() == normalized)
        })
        .map(|(namespace, _)| namespace.clone())
        .ok_or_else(|| PluginError::PluginNotFound(reference.to_string()))
}

fn validate_local_entrypoint(record: &PluginRecord) -> Result<(), PluginError> {
    match record.manifest.kind {
        PluginKind::Delegated | PluginKind::Python => {
            if let Some(manifest_root) =
                installed_manifest_root(record.manifest_path.as_deref(), &record.source)
            {
                let candidates = super::entrypoint::delegated_entrypoint_candidates(
                    &manifest_root,
                    &record.manifest.entrypoint,
                );
                let resolved = resolve_delegated_entrypoint(
                    record.manifest_path.as_deref(),
                    &record.source,
                    &record.manifest.entrypoint,
                );
                if resolved.is_none() {
                    if let Some(path) = candidates.into_iter().next() {
                        return Err(PluginError::MissingEntrypointPath {
                            kind: record.manifest.kind,
                            path,
                        });
                    }
                }
            }
        }
        PluginKind::ExternalExec => {
            let entrypoint_path = Path::new(&record.manifest.entrypoint);
            if installed_manifest_root(record.manifest_path.as_deref(), &record.source).is_some()
                || entrypoint_path.is_absolute()
            {
                let path = resolve_external_exec_entrypoint(
                    record.manifest_path.as_deref(),
                    &record.source,
                    &record.manifest.entrypoint,
                );
                if !path.exists() {
                    return Err(PluginError::MissingEntrypointPath {
                        kind: record.manifest.kind,
                        path,
                    });
                }
                if !is_executable(&path)? {
                    return Err(PluginError::NonExecutableEntrypoint { path });
                }
            }
        }
        PluginKind::Native => {}
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
    let manifest = parse_manifest_v2(&request.manifest_text)?;
    let validated = validate_manifest(manifest, host_version, reserved_namespaces)?;

    let namespace = validated.manifest.namespace.0.clone();
    let source = request.source;
    let trust_level = request.trust_level;
    let record = PluginRecord {
        manifest: validated.manifest,
        state: crate::contracts::PluginLifecycleState::Installed,
        source,
        manifest_path: request.manifest_path,
        trust_level,
        manifest_checksum_sha256,
    };
    validate_local_entrypoint(&record)?;

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
        let resolved = resolve_namespace_reference(registry, namespace)?;
        if registry.plugins.remove(&resolved).is_none() {
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
        let resolved = resolve_namespace_reference(registry, namespace)?;
        let plugin = registry
            .plugins
            .get_mut(&resolved)
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
    let resolved = resolve_namespace_reference(&registry, namespace)?;
    registry
        .plugins
        .get(&resolved)
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
    let diagnostics = load_time_diagnostics(registry_path, runtime_semver())?;
    let mut broken = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "error")
        .map(|diagnostic| diagnostic.namespace.clone())
        .collect::<Vec<_>>();
    let mut incompatible = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.severity == "warning")
        .map(|diagnostic| diagnostic.namespace.clone())
        .collect::<Vec<_>>();
    broken.sort();
    broken.dedup();
    incompatible.sort();
    incompatible.dedup();

    Ok(PluginDoctorReport { installed: registry.plugins.len(), broken, incompatible })
}

/// Check plugin compatibility against host version without mutating registry.
#[allow(dead_code)]
pub fn compatibility_check(
    manifest: &crate::contracts::PluginManifestV2,
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

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::Arc;
    use std::thread;

    use tempfile::TempDir;

    use super::{
        load_registry, lock_path, save_registry, update_registry, PluginError, PluginRegistry,
    };

    #[test]
    fn concurrent_registry_writes_keep_registry_parseable() {
        let temp = TempDir::new().expect("tempdir");
        let path = Arc::new(temp.path().join("registry.json"));

        let mut writers = Vec::new();
        for _ in 0..8 {
            let path = Arc::clone(&path);
            writers.push(thread::spawn(move || {
                for _ in 0..40 {
                    save_registry(path.as_path(), &PluginRegistry::default())
                        .expect("save registry");
                }
            }));
        }

        for writer in writers {
            writer.join().expect("join writer");
        }

        let loaded = load_registry(path.as_path()).expect("load registry");
        assert_eq!(loaded, PluginRegistry::default());
    }

    #[test]
    fn update_registry_rejects_when_lock_is_held() {
        let temp = TempDir::new().expect("tempdir");
        let path = temp.path().join("registry.json");
        save_registry(path.as_path(), &PluginRegistry::default()).expect("seed");

        let lock = lock_path(path.as_path());
        fs::write(&lock, "held\n").expect("seed lock");

        let err = update_registry(path.as_path(), |_| Ok(())).expect_err("lock should block write");
        assert!(matches!(err, PluginError::RegistryLocked(_)));
    }
}
