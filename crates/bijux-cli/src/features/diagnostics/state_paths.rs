//! Runtime state path resolution and diagnostics helpers.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::Result;
use serde_json::{json, Value};

use super::StatePathStatus;
use crate::features::config::storage::{ConfigRepository, FileConfigRepository};
use crate::features::install::{
    default_compatibility_paths, discover_compatibility_paths, load_compatibility_config,
    CompatibilityConfig, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH,
};
use crate::features::plugins::{
    plugin_doctor, prune_registry_backup, registry_path_from_plugins_dir, self_repair_registry,
    PluginError,
};
use crate::infrastructure::state_store::{read_history_entries, read_memory_map};
use crate::routing::parser::ParsedGlobalFlags;

fn home_dir() -> Option<PathBuf> {
    env::var_os("HOME").map(PathBuf::from)
}

/// Collect runtime path override environment variables.
#[must_use]
pub fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| env::var(key).ok().map(|value| ((*key).to_string(), value)))
        .collect()
}

/// Runtime state path set resolved from defaults, compatibility, env, and flags.
#[derive(Debug, Clone)]
pub struct ResolvedStatePaths {
    /// Resolved config file path.
    pub config_file: PathBuf,
    /// Resolved history file path.
    pub history_file: PathBuf,
    /// Resolved plugins directory path.
    pub plugins_dir: PathBuf,
    /// Resolved plugin registry file path.
    pub plugin_registry_file: PathBuf,
    /// Resolved memory file path.
    pub memory_file: PathBuf,
}

/// Resolve runtime state file paths from defaults, compatibility config, env, and flags.
pub fn resolve_state_paths(flags: &ParsedGlobalFlags) -> Result<ResolvedStatePaths> {
    let home = home_dir();
    let defaults = home
        .as_deref()
        .map(default_compatibility_paths)
        .unwrap_or_else(|| default_compatibility_paths(Path::new(".")));

    let config = load_compatibility_config(&defaults.config_file)
        .unwrap_or_else(|_| CompatibilityConfig::default());
    let mut overrides = PathOverrides::default();
    if let Some(path) = &flags.config_path {
        overrides.config_file = Some(path.into());
    }

    let resolved = discover_compatibility_paths(home.as_deref(), &overrides, &env_map(), &config)?;
    let plugin_registry_file = registry_path_from_plugins_dir(&resolved.plugins_dir);
    let memory_file = resolved
        .config_file
        .parent()
        .map(|dir| dir.join(".memory.json"))
        .unwrap_or_else(|| Path::new(".").join(".bijux").join(".memory.json"));

    Ok(ResolvedStatePaths {
        config_file: resolved.config_file,
        history_file: resolved.history_file,
        plugins_dir: resolved.plugins_dir,
        plugin_registry_file,
        memory_file,
    })
}

/// Convert path status into JSON payload shape used by maintainer reports.
#[must_use]
pub fn state_path_status_value(status: &StatePathStatus) -> Value {
    json!({
        "path": status.path,
        "exists": status.exists,
        "is_file": status.is_file,
        "is_dir": status.is_dir,
        "size_bytes": status.size_bytes,
        "readable": status.readable,
        "writable": status.writable,
    })
}

/// Build state diagnostics and repair actions from resolved runtime state paths.
#[must_use]
pub fn state_diagnostics(paths: &ResolvedStatePaths) -> Value {
    let mut issues = Vec::<Value>::new();
    let mut repairs = Vec::<Value>::new();

    let repository = FileConfigRepository;
    if let Err(err) = repository.load(&paths.config_file) {
        issues.push(json!({
            "area": "config",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.config_file,
        }));
    }
    if let Ok(text) = fs::read_to_string(&paths.config_file) {
        let mut seen = std::collections::BTreeMap::<String, usize>::new();
        for line in
            text.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with('#'))
        {
            if let Some((left, _)) = line.split_once('=') {
                *seen.entry(left.trim().to_string()).or_insert(0) += 1;
            }
        }
        let duplicates: Vec<String> =
            seen.into_iter().filter_map(|(key, count)| (count > 1).then_some(key)).collect();
        if !duplicates.is_empty() {
            issues.push(json!({
                "area": "config",
                "severity": "error",
                "message": "duplicate config keys found",
                "keys": duplicates,
                "path": paths.config_file,
            }));
        }
    }

    let config_tmp = paths.config_file.with_extension("tmp");
    if config_tmp.exists() {
        issues.push(json!({
            "area": "config",
            "severity": "warning",
            "message": "partial-write rollback artifact detected",
            "path": config_tmp,
        }));
    }

    if let Err(err) = read_history_entries(&paths.history_file, 20) {
        issues.push(json!({
            "area": "history",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.history_file,
        }));
    }

    match read_memory_map(&paths.memory_file) {
        Ok(memory) => {
            let wrong_type_keys: Vec<String> = memory
                .iter()
                .filter_map(|(key, value)| {
                    (!(value.is_string() || value.is_object())).then_some(key.clone())
                })
                .collect();
            if !wrong_type_keys.is_empty() {
                issues.push(json!({
                    "area": "memory",
                    "severity": "warning",
                    "message": "memory entries with wrong-type values detected",
                    "keys": wrong_type_keys,
                    "path": paths.memory_file,
                }));
            }
        }
        Err(err) => {
            issues.push(json!({
                "area": "memory",
                "severity": "error",
                "message": err.to_string(),
                "path": paths.memory_file,
            }));
        }
    }

    let mut repaired_corrupted_registry = false;
    if let Err(err) = plugin_doctor(&paths.plugin_registry_file) {
        repaired_corrupted_registry = matches!(err, PluginError::RegistryCorrupted);
        issues.push(json!({
            "area": "plugins",
            "severity": "error",
            "message": err.to_string(),
            "path": paths.plugin_registry_file,
        }));
    }

    if self_repair_registry(&paths.plugin_registry_file).is_ok() {
        if repaired_corrupted_registry {
            repairs.push(json!({
                "area": "plugins",
                "action": "repaired-corrupted-registry",
                "path": paths.plugin_registry_file,
            }));
        }
        if let Ok(true) = prune_registry_backup(&paths.plugin_registry_file) {
            repairs.push(json!({
                "area": "plugins",
                "action": "removed-stale-backup",
                "path": paths.plugin_registry_file.with_extension("bak"),
            }));
        }
    }

    json!({
        "status": if issues.is_empty() { "healthy" } else { "degraded" },
        "issues": issues,
        "repairs": repairs,
    })
}
