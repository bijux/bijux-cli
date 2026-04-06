#![forbid(unsafe_code)]
//! Read-only install/runtime identity query interfaces for maintainer tooling.

use super::diagnostics::install_health_report;

/// Structured runtime identity diagnostics queried from install services.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
pub struct RuntimeIdentityQuery {
    /// Active binary used for invocation.
    pub active_binary: Option<String>,
    /// All discovered canonical binaries in PATH order.
    pub path_binaries: Vec<String>,
    /// Whether PATH shadowing is present.
    pub has_path_shadowing: bool,
    /// Whether duplicate installs are detected across ecosystems.
    pub has_duplicate_installs: bool,
    /// Wrapper scripts that no longer point to a valid runtime.
    pub stale_wrapper_scripts: Vec<String>,
    /// Whether wheel/runtime version mismatch is detected.
    pub has_mismatched_wheel_binary_versions: bool,
    /// Legacy installer conflicts detected on PATH.
    pub legacy_installer_conflicts: Vec<String>,
    /// Whether configured active binary path is missing.
    pub active_binary_missing: bool,
    /// Whether configured active binary path is a broken symlink.
    pub broken_symlink_active_binary: bool,
}

impl RuntimeIdentityQuery {
    /// Return stale-wrapper maintenance paths using neutral naming.
    #[must_use]
    pub fn stale_wrapper_maintenance(&self) -> Vec<String> {
        self.stale_wrapper_scripts.clone()
    }
}

/// Query runtime identity diagnostics without presentation formatting.
#[must_use]
pub fn runtime_identity_query(
    path_value: &str,
    bin_override: Option<&str>,
    wheel_version: Option<&str>,
    runtime_version: &str,
) -> RuntimeIdentityQuery {
    let report = install_health_report(path_value, bin_override, wheel_version, runtime_version);
    RuntimeIdentityQuery {
        active_binary: report.active_binary,
        path_binaries: report.path_binaries,
        has_path_shadowing: report.has_path_shadowing,
        has_duplicate_installs: report.has_duplicate_installs,
        stale_wrapper_scripts: report.stale_wrapper_scripts,
        has_mismatched_wheel_binary_versions: report.has_mismatched_wheel_binary_versions,
        legacy_installer_conflicts: report.legacy_installer_conflicts,
        active_binary_missing: report.active_binary_missing,
        broken_symlink_active_binary: report.broken_symlink_active_binary,
    }
}
