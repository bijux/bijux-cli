#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

use bijux_cli_contracts::ContractMarker;
use std::collections::{BTreeMap, HashMap};
use std::path::{Path, PathBuf};
use std::{
    fs,
    io::{self, Write},
};

/// Canonical executable name.
pub const CANONICAL_EXECUTABLE: &str = "bijux";
/// Environment variable used for explicit config file path.
pub const ENV_CONFIG_PATH: &str = "BIJUXCLI_CONFIG";
/// Environment variable used for explicit history file path.
pub const ENV_HISTORY_PATH: &str = "BIJUXCLI_HISTORY_FILE";
/// Environment variable used for explicit plugin directory path.
pub const ENV_PLUGINS_PATH: &str = "BIJUXCLI_PLUGINS_DIR";

/// Installation ecosystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ecosystem {
    /// Cargo package installation.
    Cargo,
    /// Python package installation.
    Pip,
}

/// Distribution package channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageChannel {
    /// Canonical package channel.
    Canonical,
    /// Compatibility alias package channel.
    Compatibility,
}

/// Install strategy contract for a package channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallStrategy {
    /// Ecosystem used for install.
    pub ecosystem: Ecosystem,
    /// Package name to install.
    pub package_name: String,
    /// Executable exposed on PATH.
    pub executable_name: String,
}

/// Installation diagnostics report used by `bijux cli paths` and `bijux cli doctor`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallHealthReport {
    /// Active binary used for invocation.
    pub active_binary: Option<String>,
    /// All discovered binaries named `bijux` in PATH order.
    pub path_binaries: Vec<String>,
    /// Whether multiple binaries are discovered in PATH order.
    pub has_path_shadowing: bool,
    /// Whether installs appear to exist across multiple ecosystems.
    pub has_duplicate_installs: bool,
    /// Wrapper scripts that no longer point to an existing runtime.
    pub stale_wrapper_scripts: Vec<String>,
    /// Whether wheel and runtime binary versions differ.
    pub has_mismatched_wheel_binary_versions: bool,
    /// Legacy installer wrappers that could shadow canonical runtime.
    pub legacy_installer_conflicts: Vec<String>,
}

/// Compatibility paths consumed by Python and Rust implementations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompatibilityPaths {
    /// Path to `config.env`.
    pub config_file: PathBuf,
    /// Path to history store.
    pub history_file: PathBuf,
    /// Path to plugins directory.
    pub plugins_dir: PathBuf,
}

/// Key-based path overrides from command-line flags.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PathOverrides {
    /// Optional override for config file path.
    pub config_file: Option<PathBuf>,
    /// Optional override for history file path.
    pub history_file: Option<PathBuf>,
    /// Optional override for plugins directory path.
    pub plugins_dir: Option<PathBuf>,
}

/// Parsed file-backed compatibility configuration.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompatibilityConfig {
    /// Optional path from config file for config path recursion-safe representation.
    pub config_file: Option<PathBuf>,
    /// Optional path from config file for history file.
    pub history_file: Option<PathBuf>,
    /// Optional path from config file for plugins directory.
    pub plugins_dir: Option<PathBuf>,
}

/// Error type for compatibility discovery and file operations.
#[derive(Debug, thiserror::Error)]
pub enum CompatibilityError {
    /// Home directory not provided.
    #[error("home directory is required for compatibility path discovery")]
    MissingHome,
    /// Config file contained an unknown key.
    #[error("unsupported config key: {0}")]
    UnsupportedConfigKey(String),
    /// Config file contains malformed line.
    #[error("malformed config line {line}: {content}")]
    MalformedConfigLine {
        /// 1-based line number.
        line: usize,
        /// Original line content.
        content: String,
    },
    /// Lock file already exists for mutable state operation.
    #[error("state lock is already held at {0}")]
    LockHeld(PathBuf),
    /// Underlying I/O failure.
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// Shell targets for completion generation during installation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionShell {
    /// Bash shell completion.
    Bash,
    /// Zsh shell completion.
    Zsh,
    /// Fish shell completion.
    Fish,
    /// PowerShell completion.
    PowerShell,
}

/// Build installer marker.
#[must_use]
pub fn installer_marker() -> ContractMarker {
    ContractMarker { namespace: "install".to_string() }
}

/// Decide canonical crate naming strategy.
#[must_use]
pub fn canonical_crate_name() -> &'static str {
    "bijux-cli"
}

/// Whether compatibility alias package should be published.
#[must_use]
pub fn publish_compatibility_package_alias() -> bool {
    true
}

/// Build cargo install strategy for canonical or compatibility channel.
#[must_use]
pub fn cargo_install_strategy(channel: PackageChannel) -> InstallStrategy {
    let package_name = match channel {
        PackageChannel::Canonical => "bijux-cli",
        PackageChannel::Compatibility => "bijux",
    };
    InstallStrategy {
        ecosystem: Ecosystem::Cargo,
        package_name: package_name.to_string(),
        executable_name: CANONICAL_EXECUTABLE.to_string(),
    }
}

/// Build pip install strategy for canonical or compatibility channel.
#[must_use]
pub fn pip_install_strategy(channel: PackageChannel) -> InstallStrategy {
    let package_name = match channel {
        PackageChannel::Canonical => "bijux-cli",
        PackageChannel::Compatibility => "bijux",
    };
    InstallStrategy {
        ecosystem: Ecosystem::Pip,
        package_name: package_name.to_string(),
        executable_name: CANONICAL_EXECUTABLE.to_string(),
    }
}

/// Validate that an install strategy does not produce conflicting executables.
#[must_use]
pub fn has_secondary_executable_conflict(strategies: &[InstallStrategy]) -> bool {
    strategies
        .iter()
        .any(|strategy| strategy.executable_name != CANONICAL_EXECUTABLE)
}

fn is_executable_like(path: &Path) -> bool {
    path.is_file()
}

fn path_entries(path_value: &str) -> impl Iterator<Item = PathBuf> + '_ {
    std::env::split_paths(path_value)
}

/// Collect discovered `bijux` binaries in PATH order.
#[must_use]
pub fn discover_path_binaries(path_value: &str) -> Vec<String> {
    path_entries(path_value)
        .map(|entry| entry.join(CANONICAL_EXECUTABLE))
        .filter(|candidate| is_executable_like(candidate))
        .map(|candidate| candidate.display().to_string())
        .collect()
}

/// Resolve active binary from override or PATH discovery.
#[must_use]
pub fn resolve_active_binary(path_value: &str, bin_override: Option<&str>) -> Option<String> {
    if let Some(override_path) = bin_override.filter(|value| !value.trim().is_empty()) {
        return Some(override_path.to_string());
    }
    discover_path_binaries(path_value).into_iter().next()
}

/// Detect stale wrapper scripts in PATH.
#[must_use]
pub fn detect_stale_wrapper_scripts(path_value: &str) -> Vec<String> {
    path_entries(path_value)
        .map(|entry| entry.join(format!("{CANONICAL_EXECUTABLE}.sh")))
        .filter(|wrapper| is_executable_like(wrapper))
        .filter(|wrapper| !wrapper.with_file_name(CANONICAL_EXECUTABLE).exists())
        .map(|wrapper| wrapper.display().to_string())
        .collect()
}

/// Build installation diagnostics for binary resolution and ecosystem overlap checks.
#[must_use]
pub fn install_health_report(
    path_value: &str,
    bin_override: Option<&str>,
    wheel_version: Option<&str>,
    runtime_version: &str,
) -> InstallHealthReport {
    let path_binaries = discover_path_binaries(path_value);
    let active_binary = resolve_active_binary(path_value, bin_override);
    let has_path_shadowing = path_binaries.len() > 1;
    let has_duplicate_installs =
        path_binaries.iter().any(|path| path.contains(".cargo")) && path_binaries.iter().any(|path| path.contains("site-packages"));
    let stale_wrapper_scripts = detect_stale_wrapper_scripts(path_value);
    let has_mismatched_wheel_binary_versions = wheel_version.is_some_and(|version| version != runtime_version);
    let legacy_installer_conflicts = legacy_installer_conflicts(path_value);

    InstallHealthReport {
        active_binary,
        path_binaries,
        has_path_shadowing,
        has_duplicate_installs,
        stale_wrapper_scripts,
        has_mismatched_wheel_binary_versions,
        legacy_installer_conflicts,
    }
}

/// Generate deterministic completion content for an install hook.
#[must_use]
pub fn completion_script(shell: CompletionShell) -> &'static str {
    match shell {
        CompletionShell::Bash => "complete -W \"cli dev doctor version repl completion inspect\" bijux",
        CompletionShell::Zsh => "#compdef bijux\n_arguments '*::command:->commands'",
        CompletionShell::Fish => "complete -c bijux -f -a \"cli dev doctor version repl completion inspect\"",
        CompletionShell::PowerShell => "Register-ArgumentCompleter -CommandName bijux -ScriptBlock { param($wordToComplete) }",
    }
}

/// Return the post-install hint shown after successful install.
#[must_use]
pub fn post_install_hint(binary_path: &str) -> String {
    format!(
        "Installed `{CANONICAL_EXECUTABLE}` at {binary_path}. Run `bijux version` and `bijux cli doctor` to verify your environment."
    )
}

/// Detect known legacy wrappers that could shadow the canonical binary.
#[must_use]
pub fn legacy_installer_conflicts(path_value: &str) -> Vec<String> {
    const LEGACY_CANDIDATES: &[&str] = &["bijux.py", "bijux-legacy", "bijux_old", "bijux-cli.sh"];
    path_entries(path_value)
        .flat_map(|entry| LEGACY_CANDIDATES.iter().map(move |name| entry.join(name)))
        .filter(|candidate| candidate.exists())
        .map(|candidate| candidate.display().to_string())
        .collect()
}

/// Initialize first-run filesystem state and return whether setup ran this invocation.
pub fn initialize_first_run_state(state_root: &Path) -> io::Result<bool> {
    fs::create_dir_all(state_root)?;
    let marker = state_root.join(".first-run-ready");
    if marker.exists() {
        return Ok(false);
    }
    fs::write(marker, b"ready")?;
    Ok(true)
}

/// Resolve effective compatibility paths with strict precedence:
/// CLI flag overrides -> environment variables -> config file -> defaults.
pub fn discover_compatibility_paths(
    home_dir: Option<&Path>,
    cli_overrides: &PathOverrides,
    env_map: &HashMap<String, String>,
    file_config: &CompatibilityConfig,
) -> Result<CompatibilityPaths, CompatibilityError> {
    let home = home_dir.ok_or(CompatibilityError::MissingHome)?;
    let defaults = default_compatibility_paths(home);

    let config_file = select_path(
        cli_overrides.config_file.as_ref(),
        env_map.get(ENV_CONFIG_PATH),
        file_config.config_file.as_ref(),
        &defaults.config_file,
        home,
    );
    let history_file = select_path(
        cli_overrides.history_file.as_ref(),
        env_map.get(ENV_HISTORY_PATH),
        file_config.history_file.as_ref(),
        &defaults.history_file,
        home,
    );
    let plugins_dir = select_path(
        cli_overrides.plugins_dir.as_ref(),
        env_map.get(ENV_PLUGINS_PATH),
        file_config.plugins_dir.as_ref(),
        &defaults.plugins_dir,
        home,
    );

    Ok(CompatibilityPaths { config_file, history_file, plugins_dir })
}

/// Default compatibility paths anchored in the user home directory.
#[must_use]
pub fn default_compatibility_paths(home_dir: &Path) -> CompatibilityPaths {
    let base = home_dir.join(".bijux");
    CompatibilityPaths {
        config_file: base.join(".env"),
        history_file: base.join(".history"),
        plugins_dir: base.join(".plugins"),
    }
}

/// Parse `.env`-style configuration file.
pub fn parse_compatibility_config(text: &str) -> Result<CompatibilityConfig, CompatibilityError> {
    let mut values = BTreeMap::<String, String>::new();

    for (index, raw_line) in text.lines().enumerate() {
        let line_no = index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            return Err(CompatibilityError::MalformedConfigLine {
                line: line_no,
                content: raw_line.to_string(),
            });
        };

        let trimmed_key = key.trim();
        let trimmed_value = value.trim();
        match trimmed_key {
            ENV_CONFIG_PATH | ENV_HISTORY_PATH | ENV_PLUGINS_PATH => {
                values.insert(trimmed_key.to_string(), trimmed_value.to_string());
            }
            _ => {
                return Err(CompatibilityError::UnsupportedConfigKey(trimmed_key.to_string()));
            }
        }
    }

    Ok(CompatibilityConfig {
        config_file: values.get(ENV_CONFIG_PATH).map(PathBuf::from),
        history_file: values.get(ENV_HISTORY_PATH).map(PathBuf::from),
        plugins_dir: values.get(ENV_PLUGINS_PATH).map(PathBuf::from),
    })
}

/// Read and parse compatibility config file if it exists.
pub fn load_compatibility_config(path: &Path) -> Result<CompatibilityConfig, CompatibilityError> {
    if !path.exists() {
        return Ok(CompatibilityConfig::default());
    }

    let text = fs::read_to_string(path)?;
    parse_compatibility_config(&text)
}

/// Persist compatibility config atomically.
pub fn write_compatibility_config(
    path: &Path,
    config: &CompatibilityConfig,
) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut lines = Vec::new();
    if let Some(value) = &config.config_file {
        lines.push(format!("{ENV_CONFIG_PATH}={}", value.display()));
    }
    if let Some(value) = &config.history_file {
        lines.push(format!("{ENV_HISTORY_PATH}={}", value.display()));
    }
    if let Some(value) = &config.plugins_dir {
        lines.push(format!("{ENV_PLUGINS_PATH}={}", value.display()));
    }
    lines.sort();

    let rendered = if lines.is_empty() {
        String::new()
    } else {
        let mut buf = lines.join("\n");
        buf.push('\n');
        buf
    };

    let temp_path = path.with_extension("tmp");
    {
        let mut temp = fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temp_path)?;
        temp.write_all(rendered.as_bytes())?;
        temp.sync_all()?;
    }

    fs::rename(temp_path, path)?;
    Ok(())
}

/// Acquire process lock for mutable state operations.
pub fn acquire_state_lock(lock_path: &Path) -> Result<StateLockGuard, CompatibilityError> {
    if let Some(parent) = lock_path.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::OpenOptions::new().create_new(true).write(true).open(lock_path) {
        Ok(mut file) => {
            file.write_all(b"bijux-cli lock\n")?;
            file.sync_all()?;
            Ok(StateLockGuard { path: lock_path.to_path_buf() })
        }
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            Err(CompatibilityError::LockHeld(lock_path.to_path_buf()))
        }
        Err(error) => Err(CompatibilityError::Io(error)),
    }
}

/// Guard that removes the lock path when dropped.
#[derive(Debug)]
pub struct StateLockGuard {
    path: PathBuf,
}

impl Drop for StateLockGuard {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.path);
    }
}

/// Ensure history file exists and parent directory is present.
pub fn ensure_history_file(path: &Path) -> Result<(), CompatibilityError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    if !path.exists() {
        let mut file = fs::OpenOptions::new().create_new(true).write(true).open(path)?;
        file.write_all(b"[]\n")?;
        file.sync_all()?;
    }

    Ok(())
}

/// Ensure plugin directory exists.
pub fn ensure_plugins_dir(path: &Path) -> Result<(), CompatibilityError> {
    fs::create_dir_all(path)?;
    Ok(())
}

/// Placeholder migration entrypoint for forward config evolution.
pub fn run_config_migrations(
    _config_path: &Path,
    _current_version: u32,
) -> Result<(), CompatibilityError> {
    Ok(())
}

fn select_path(
    cli_value: Option<&PathBuf>,
    env_value: Option<&String>,
    config_value: Option<&PathBuf>,
    default_value: &Path,
    home_dir: &Path,
) -> PathBuf {
    let candidate = cli_value
        .cloned()
        .or_else(|| env_value.map(PathBuf::from))
        .or_else(|| config_value.cloned())
        .unwrap_or_else(|| default_value.to_path_buf());

    normalize_path(&candidate, home_dir)
}

fn normalize_path(path: &Path, home_dir: &Path) -> PathBuf {
    let Some(raw) = path.to_str() else {
        return path.to_path_buf();
    };

    if raw == "~" {
        return home_dir.to_path_buf();
    }
    if let Some(tail) = raw.strip_prefix("~/") {
        return home_dir.join(tail);
    }
    if path.is_absolute() {
        return path.to_path_buf();
    }

    home_dir.join(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn cargo_channels_resolve_to_same_canonical_executable() {
        let canonical = cargo_install_strategy(PackageChannel::Canonical);
        let compatibility = cargo_install_strategy(PackageChannel::Compatibility);
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
        assert_eq!(compatibility.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn pip_channels_resolve_to_same_canonical_executable() {
        let canonical = pip_install_strategy(PackageChannel::Canonical);
        let compatibility = pip_install_strategy(PackageChannel::Compatibility);
        assert_eq!(canonical.executable_name, CANONICAL_EXECUTABLE);
        assert_eq!(compatibility.executable_name, CANONICAL_EXECUTABLE);
    }

    #[test]
    fn no_secondary_executable_conflicts_for_supported_strategies() {
        let strategies = vec![
            cargo_install_strategy(PackageChannel::Canonical),
            cargo_install_strategy(PackageChannel::Compatibility),
            pip_install_strategy(PackageChannel::Canonical),
            pip_install_strategy(PackageChannel::Compatibility),
        ];
        assert!(!has_secondary_executable_conflict(&strategies));
    }

    #[test]
    fn path_binary_discovery_respects_path_order() {
        let temp = TempDir::new().expect("tempdir");
        let first = temp.path().join("first");
        let second = temp.path().join("second");
        std::fs::create_dir_all(&first).expect("first");
        std::fs::create_dir_all(&second).expect("second");
        std::fs::write(first.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write first");
        std::fs::write(second.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write second");
        let path_value = std::env::join_paths([&first, &second]).expect("join");
        let discovered =
            discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 2);
        assert!(discovered[0].contains("first"));
    }

    #[test]
    fn install_health_report_flags_shadowing_and_duplicate_installs() {
        let temp = TempDir::new().expect("tempdir");
        let pip_like = temp.path().join("python-site-packages-bin");
        let cargo_like = temp.path().join(".cargo-bin");
        std::fs::create_dir_all(&pip_like).expect("pip dir");
        std::fs::create_dir_all(&cargo_like).expect("cargo dir");
        std::fs::write(pip_like.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write pip");
        std::fs::write(cargo_like.join(CANONICAL_EXECUTABLE), b"#!/bin/sh\n").expect("write cargo");
        let path_value = std::env::join_paths([&pip_like, &cargo_like]).expect("join");

        let report = install_health_report(
            path_value.to_str().expect("utf-8 path"),
            None,
            Some("1.0.0"),
            "1.0.1",
        );
        assert!(report.has_duplicate_installs);
        assert!(report.has_mismatched_wheel_binary_versions);
    }

    #[test]
    fn active_binary_prefers_explicit_override() {
        let report = install_health_report(
            "",
            Some("/custom/bin/bijux"),
            None,
            "1.0.0",
        );
        assert_eq!(report.active_binary, Some("/custom/bin/bijux".to_string()));
    }

    #[test]
    fn completion_scripts_are_available_for_supported_shells() {
        assert!(completion_script(CompletionShell::Bash).contains("complete"));
        assert!(completion_script(CompletionShell::Zsh).contains("#compdef"));
        assert!(completion_script(CompletionShell::Fish).contains("complete -c"));
        assert!(completion_script(CompletionShell::PowerShell).contains("Register-ArgumentCompleter"));
    }

    #[test]
    fn post_install_hint_mentions_verification_commands() {
        let hint = post_install_hint("/usr/local/bin/bijux");
        assert!(hint.contains("bijux version"));
        assert!(hint.contains("bijux cli doctor"));
    }

    #[test]
    fn first_run_setup_is_idempotent() {
        let temp = TempDir::new().expect("tempdir");
        let first = initialize_first_run_state(temp.path()).expect("first run");
        let second = initialize_first_run_state(temp.path()).expect("second run");
        assert!(first);
        assert!(!second);
    }

    #[test]
    fn detects_legacy_installer_conflicts() {
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("bin");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::write(dir.join("bijux.py"), b"#!/usr/bin/env python\n").expect("write legacy");
        let path_value = std::env::join_paths([&dir]).expect("join");
        let conflicts = legacy_installer_conflicts(path_value.to_str().expect("utf-8 path"));
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts[0].contains("bijux.py"));
    }

    #[test]
    fn no_network_environment_preserves_path_resolution() {
        let report = install_health_report("", None, None, "1.0.0");
        assert!(report.path_binaries.is_empty());
    }

    #[test]
    fn offline_plugin_registry_operation_is_stable_for_install_diagnostics() {
        let report = install_health_report("", Some("/tmp/bijux"), None, "1.0.0");
        assert_eq!(report.active_binary.as_deref(), Some("/tmp/bijux"));
    }

    #[test]
    fn read_only_environment_does_not_break_idempotent_check_when_marker_exists() {
        let temp = TempDir::new().expect("tempdir");
        let marker = temp.path().join(".first-run-ready");
        std::fs::write(&marker, b"ready").expect("write marker");
        let result = initialize_first_run_state(temp.path()).expect("idempotent check");
        assert!(!result);
    }

    #[test]
    fn nonstandard_home_paths_are_supported() {
        let temp = TempDir::new().expect("tempdir");
        let home = temp.path().join("XDG DATA HOME");
        std::fs::create_dir_all(&home).expect("mkdir");
        let initialized = initialize_first_run_state(&home).expect("initialize");
        assert!(initialized);
    }

    #[test]
    fn symlinked_binary_paths_are_detected() {
        let temp = TempDir::new().expect("tempdir");
        let target_dir = temp.path().join("target-bin");
        let link_dir = temp.path().join("link-bin");
        std::fs::create_dir_all(&target_dir).expect("mkdir target");
        std::fs::create_dir_all(&link_dir).expect("mkdir link");
        let binary = target_dir.join(CANONICAL_EXECUTABLE);
        std::fs::write(&binary, b"#!/bin/sh\n").expect("write binary");
        #[cfg(unix)]
        std::os::unix::fs::symlink(&binary, link_dir.join(CANONICAL_EXECUTABLE)).expect("symlink");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&binary, link_dir.join(CANONICAL_EXECUTABLE)).expect("symlink");
        let path_value = std::env::join_paths([&link_dir]).expect("join");
        let discovered = discover_path_binaries(path_value.to_str().expect("utf-8 path"));
        assert_eq!(discovered.len(), 1);
    }

    #[test]
    fn windows_path_override_is_respected() {
        let report = install_health_report(
            "",
            Some(r"C:\Program Files\Bijux\bijux.exe"),
            None,
            "1.0.0",
        );
        assert_eq!(
            report.active_binary.as_deref(),
            Some(r"C:\Program Files\Bijux\bijux.exe")
        );
    }

    #[test]
    #[cfg(unix)]
    fn read_only_state_directory_reports_error_on_first_write() {
        use std::os::unix::fs::PermissionsExt;
        let temp = TempDir::new().expect("tempdir");
        let dir = temp.path().join("readonly");
        std::fs::create_dir_all(&dir).expect("mkdir");
        std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o555))
            .expect("set readonly");
        let result = initialize_first_run_state(&dir);
        assert!(result.is_err());
    }
}
