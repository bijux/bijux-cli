#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

use bijux_cli_contracts::ContractMarker;
use std::path::{Path, PathBuf};

/// Canonical executable name.
pub const CANONICAL_EXECUTABLE: &str = "bijux";

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

    InstallHealthReport {
        active_binary,
        path_binaries,
        has_path_shadowing,
        has_duplicate_installs,
        stale_wrapper_scripts,
        has_mismatched_wheel_binary_versions,
    }
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
}
