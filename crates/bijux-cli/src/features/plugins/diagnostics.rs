#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::contracts::PluginKind;

use super::errors::PluginError;
use super::manifest::is_version_compatible;
use super::models::PluginLoadDiagnostic;
use super::registry::{load_registry, save_registry};

fn installed_manifest_root(source: &str) -> Option<&Path> {
    let path = Path::new(source);
    if !path.is_file() {
        return None;
    }
    if path.file_name().and_then(|name| name.to_str()) != Some("plugin.manifest.json") {
        return None;
    }
    path.parent()
}

fn delegated_entrypoint_file(manifest_root: &Path, entrypoint: &str) -> Option<std::path::PathBuf> {
    let module = entrypoint.split_once(':').map_or(entrypoint, |(module, _)| module);
    if module.trim().is_empty() {
        return None;
    }

    let mut path = manifest_root.to_path_buf();
    for segment in module.split('.') {
        if segment.trim().is_empty() {
            return None;
        }
        path.push(segment);
    }
    if path.extension().is_none() {
        path.set_extension("py");
    }
    Some(path)
}

/// Generate load-time diagnostics for broken or incompatible plugins.
pub fn load_time_diagnostics(
    registry_path: &Path,
    host_version: &str,
) -> Result<Vec<PluginLoadDiagnostic>, PluginError> {
    let registry = load_registry(registry_path)?;
    let mut diagnostics = Vec::new();

    for (namespace, record) in &registry.plugins {
        if record.state == crate::contracts::PluginLifecycleState::Broken {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "error".to_string(),
                message: "plugin is marked broken".to_string(),
            });
            continue;
        }

        if !is_version_compatible(&record.manifest.compatibility, host_version)? {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "warning".to_string(),
                message: format!("plugin compatibility does not include host {host_version}"),
            });
        }

        if record.manifest.kind == PluginKind::ExternalExec
            && !Path::new(&record.manifest.entrypoint).exists()
        {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "error".to_string(),
                message: "external-exec entrypoint was not found".to_string(),
            });
        }

        if matches!(record.manifest.kind, PluginKind::Delegated | PluginKind::Python)
            && installed_manifest_root(&record.source)
                .and_then(|root| delegated_entrypoint_file(root, &record.manifest.entrypoint))
                .is_some_and(|entrypoint| !entrypoint.exists())
        {
            diagnostics.push(PluginLoadDiagnostic {
                namespace: namespace.clone(),
                severity: "error".to_string(),
                message: "delegated entrypoint was not found".to_string(),
            });
        }
    }

    Ok(diagnostics)
}

/// Return compatibility warnings for plugin surfaces.
pub fn compatibility_warnings(
    registry_path: &Path,
    host_version: &str,
) -> Result<Vec<String>, PluginError> {
    let diagnostics = load_time_diagnostics(registry_path, host_version)?;
    Ok(diagnostics
        .into_iter()
        .map(|diagnostic| format!("{}: {}", diagnostic.namespace, diagnostic.message))
        .collect())
}

/// Try to repair a corrupted registry by quarantining the old file and writing a fresh empty registry.
pub fn self_repair_registry(path: &Path) -> Result<super::models::PluginRegistry, PluginError> {
    match load_registry(path) {
        Ok(registry) => Ok(registry),
        Err(PluginError::RegistryCorrupted) => {
            if path.exists() {
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .map_or(0, |duration| duration.as_secs());
                let quarantine = path.with_extension(format!("corrupt-{timestamp}.json"));
                fs::rename(path, quarantine)?;
            }
            let repaired = super::models::PluginRegistry::default();
            save_registry(path, &repaired)?;
            Ok(repaired)
        }
        Err(error) => Err(error),
    }
}

/// Remove stale transactional backup file left next to registry path.
pub fn prune_registry_backup(path: &Path) -> Result<bool, PluginError> {
    let backup = path.with_extension("bak");
    if !backup.exists() {
        return Ok(false);
    }
    fs::remove_file(backup)?;
    Ok(true)
}
