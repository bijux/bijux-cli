#![forbid(unsafe_code)]
//! Installation diagnostics and health reporting.

use crate::paths::{
    detect_stale_wrapper_scripts, discover_path_binaries, legacy_installer_conflicts,
    resolve_active_binary,
};

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

/// Build installation diagnostics for binary resolution and ecosystem overlap checks.
#[must_use]
pub fn install_health_report(
    path_value: &str,
    bin_override: Option<&str>,
    wheel_version: Option<&str>,
    runtime_version: &str,
) -> InstallHealthReport {
    let mut path_binaries = discover_path_binaries(path_value);
    path_binaries.dedup();
    let active_binary = resolve_active_binary(path_value, bin_override);
    let has_path_shadowing = path_binaries.len() > 1;
    let has_duplicate_installs = path_binaries.iter().any(|path| path.contains(".cargo"))
        && path_binaries.iter().any(|path| path.contains("site-packages"));
    let stale_wrapper_scripts = detect_stale_wrapper_scripts(path_value);
    let has_mismatched_wheel_binary_versions =
        wheel_version.is_some_and(|version| version != runtime_version);
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
