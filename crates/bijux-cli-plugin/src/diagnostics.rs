#![forbid(unsafe_code)]

use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use bijux_cli_routing::PluginKind;

use crate::errors::PluginError;
use crate::manifest::is_version_compatible;
use crate::models::PluginLoadDiagnostic;
use crate::registry::{load_registry, save_registry};

/// Generate load-time diagnostics for broken or incompatible plugins.
pub fn load_time_diagnostics(
    registry_path: &Path,
    host_version: &str,
) -> Result<Vec<PluginLoadDiagnostic>, PluginError> {
    let registry = load_registry(registry_path)?;
    let mut diagnostics = Vec::new();

    for (namespace, record) in &registry.plugins {
        if record.state == bijux_cli_routing::PluginLifecycleState::Broken {
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
pub fn self_repair_registry(path: &Path) -> Result<crate::models::PluginRegistry, PluginError> {
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
            let repaired = crate::models::PluginRegistry::default();
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
