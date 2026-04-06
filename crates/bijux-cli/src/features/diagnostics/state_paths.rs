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
    CompatibilityConfig, CompatibilityError, PathOverrides, ENV_CONFIG_PATH, ENV_HISTORY_PATH,
    ENV_PLUGINS_PATH,
};
use crate::features::plugins::{
    plugin_doctor, prune_registry_backup, registry_path_from_plugins_dir, self_repair_registry,
    PluginError,
};
use crate::infrastructure::state_store::{read_history_report, read_memory_map};
use crate::routing::parser::ParsedGlobalFlags;

fn non_empty_env_value(name: &str) -> Option<String> {
    env::var(name).ok().map(|value| value.trim().to_string()).filter(|value| !value.is_empty())
}

fn home_dir_from_env(
    home: Option<&str>,
    user_profile: Option<&str>,
    home_drive: Option<&str>,
    home_path: Option<&str>,
    fallback_current_dir: PathBuf,
) -> (PathBuf, Option<String>) {
    if let Some(value) = home {
        return (PathBuf::from(value), None);
    }
    if let Some(value) = user_profile {
        return (
            PathBuf::from(value),
            Some(format!("HOME is unset; resolved state paths from USERPROFILE ({value})")),
        );
    }
    if let (Some(drive), Some(path)) = (home_drive, home_path) {
        let resolved = PathBuf::from(format!("{drive}{path}"));
        return (
            resolved.clone(),
            Some(format!(
                "HOME and USERPROFILE are unset; resolved state paths from HOMEDRIVE/HOMEPATH ({})",
                resolved.display()
            )),
        );
    }
    (
        fallback_current_dir.clone(),
        Some(format!(
            "HOME, USERPROFILE, and HOMEDRIVE/HOMEPATH are unset; resolved state paths from current directory ({})",
            fallback_current_dir.display()
        )),
    )
}

fn resolved_home_dir() -> (PathBuf, Option<String>) {
    let fallback_current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    home_dir_from_env(
        non_empty_env_value("HOME").as_deref(),
        non_empty_env_value("USERPROFILE").as_deref(),
        non_empty_env_value("HOMEDRIVE").as_deref(),
        non_empty_env_value("HOMEPATH").as_deref(),
        fallback_current_dir,
    )
}

fn merge_warnings(primary: Option<String>, secondary: Option<String>) -> Option<String> {
    match (primary, secondary) {
        (Some(first), Some(second)) => Some(format!("{first}; {second}")),
        (Some(first), None) => Some(first),
        (None, Some(second)) => Some(second),
        (None, None) => None,
    }
}

/// Collect runtime path override environment variables.
#[must_use]
pub fn env_map() -> HashMap<String, String> {
    [ENV_CONFIG_PATH, ENV_HISTORY_PATH, ENV_PLUGINS_PATH]
        .iter()
        .filter_map(|key| non_empty_env_value(key).map(|value| ((*key).to_string(), value)))
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
    /// Compatibility config file path used during discovery.
    pub compatibility_config_file: PathBuf,
    /// Compatibility override parse warning (if fallback defaults were used).
    pub compatibility_config_warning: Option<String>,
}

/// Resolve runtime state file paths from defaults, compatibility config, env, and flags.
pub fn resolve_state_paths(flags: &ParsedGlobalFlags) -> Result<ResolvedStatePaths> {
    let (effective_home, home_resolution_warning) = resolved_home_dir();
    let defaults = default_compatibility_paths(&effective_home);

    let compatibility_config_file = defaults.config_file.clone();
    let (config, compatibility_parse_warning) =
        match load_compatibility_config(&compatibility_config_file) {
            Ok(config) => (config, None),
            Err(error @ CompatibilityError::UnsupportedConfigKey(_))
            | Err(error @ CompatibilityError::MalformedConfigLine { .. })
            | Err(error @ CompatibilityError::DuplicateConfigKey { .. })
            | Err(error @ CompatibilityError::EmptyConfigValue { .. }) => (
                CompatibilityConfig::default(),
                Some(format!(
                    "compatibility override parsing failed for {}: {error}",
                    compatibility_config_file.display()
                )),
            ),
            Err(error) => return Err(error.into()),
        };
    let compatibility_config_warning =
        merge_warnings(home_resolution_warning, compatibility_parse_warning);
    let mut overrides = PathOverrides::default();
    if let Some(path) = &flags.config_path {
        overrides.config_file = Some(path.into());
    }

    let resolved = discover_compatibility_paths(
        Some(effective_home.as_path()),
        &overrides,
        &env_map(),
        &config,
    )?;
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
        compatibility_config_file,
        compatibility_config_warning,
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

    if let Some(message) = &paths.compatibility_config_warning {
        issues.push(json!({
            "area": "paths",
            "severity": "warning",
            "message": message,
            "path": paths.compatibility_config_file,
        }));
    }

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

    match read_history_report(&paths.history_file, 20) {
        Ok(history_report) => {
            if history_report.dropped_invalid_entries > 0 {
                issues.push(json!({
                    "area": "history",
                    "severity": "warning",
                    "message": "history file contains invalid entries that were ignored",
                    "dropped_invalid_entries": history_report.dropped_invalid_entries,
                    "accepted_entries": history_report.total_entries,
                    "observed_entries": history_report.observed_entries,
                    "path": paths.history_file,
                }));
            }
            if history_report.truncated_command_entries > 0 {
                issues.push(json!({
                    "area": "history",
                    "severity": "warning",
                    "message": "history file contains commands that exceeded the command size budget",
                    "truncated_command_entries": history_report.truncated_command_entries,
                    "accepted_entries": history_report.total_entries,
                    "observed_entries": history_report.observed_entries,
                    "path": paths.history_file,
                }));
            }
            if matches!(history_report.source_format, "legacy-lines" | "legacy-json-lines") {
                issues.push(json!({
                    "area": "history",
                    "severity": "warning",
                    "message": "history file uses legacy layout; rewrite as a JSON array for deterministic behavior",
                    "source_format": history_report.source_format,
                    "accepted_entries": history_report.total_entries,
                    "observed_entries": history_report.observed_entries,
                    "path": paths.history_file,
                }));
            }
        }
        Err(err) => {
            issues.push(json!({
                "area": "history",
                "severity": "error",
                "message": err.to_string(),
                "path": paths.history_file,
            }));
        }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::home_dir_from_env;

    #[test]
    fn home_resolution_prefers_home_without_warning() {
        let (path, warning) = home_dir_from_env(
            Some("/tmp/home"),
            Some("/tmp/profile"),
            Some("C:"),
            Some("\\Users\\profile"),
            PathBuf::from("/tmp/fallback"),
        );
        assert_eq!(path, PathBuf::from("/tmp/home"));
        assert!(warning.is_none());
    }

    #[test]
    fn home_resolution_uses_userprofile_when_home_is_missing() {
        let (path, warning) = home_dir_from_env(
            None,
            Some(r"C:\Users\profile"),
            Some("C:"),
            Some("\\Users\\profile"),
            PathBuf::from("."),
        );
        assert_eq!(path, PathBuf::from(r"C:\Users\profile"));
        assert!(warning.as_deref().is_some_and(|value| value.contains("USERPROFILE")));
    }

    #[test]
    fn home_resolution_uses_homedrive_and_homepath_when_others_are_missing() {
        let (path, warning) =
            home_dir_from_env(None, None, Some("D:"), Some("\\Work\\User"), PathBuf::from("."));
        assert_eq!(path, PathBuf::from(r"D:\Work\User"));
        assert!(warning.as_deref().is_some_and(|value| value.contains("HOMEDRIVE/HOMEPATH")));
    }

    #[test]
    fn home_resolution_falls_back_to_current_directory_with_warning() {
        let fallback = PathBuf::from("/tmp/fallback");
        let (path, warning) = home_dir_from_env(None, None, None, None, fallback.clone());
        assert_eq!(path, fallback);
        assert!(warning.as_deref().is_some_and(|value| value.contains("current directory")));
    }
}
