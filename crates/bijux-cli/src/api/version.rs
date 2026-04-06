#![forbid(unsafe_code)]
//! Runtime version API.

/// Runtime version source classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeVersionInfo {
    /// Runtime binary/product name.
    pub name: &'static str,
    /// User-facing version string.
    pub version: &'static str,
    /// Semver-compatible runtime version.
    pub semver: &'static str,
    /// Version provenance source.
    pub source: &'static str,
    /// Optional commit identifier for the build.
    pub git_commit: Option<&'static str>,
    /// Optional repository dirty-state marker.
    pub git_dirty: Option<bool>,
    /// Runtime profile (`debug` or `release`).
    pub build_profile: &'static str,
}

/// Runtime version string for user-visible output.
#[must_use]
pub const fn runtime_version() -> &'static str {
    crate::shared::version::runtime_version()
}

/// Semver-compatible runtime version string used for compatibility checks.
#[must_use]
pub const fn runtime_semver() -> &'static str {
    crate::shared::version::runtime_semver()
}

/// Runtime version source classification.
#[must_use]
pub fn runtime_version_source() -> &'static str {
    crate::shared::version::runtime_version_source()
}

/// Runtime commit identifier when available.
#[must_use]
pub fn runtime_git_commit() -> Option<&'static str> {
    crate::shared::version::runtime_git_commit()
}

/// Runtime dirty-state marker when available.
#[must_use]
pub fn runtime_git_dirty() -> Option<bool> {
    crate::shared::version::runtime_git_dirty()
}

/// Runtime build profile (`debug` or `release`).
#[must_use]
pub fn runtime_build_profile() -> &'static str {
    crate::shared::version::runtime_build_profile()
}

/// Canonical runtime version metadata.
#[must_use]
pub fn runtime_version_info() -> RuntimeVersionInfo {
    let info = crate::shared::version::runtime_version_info();
    RuntimeVersionInfo {
        name: info.name,
        version: info.version,
        semver: info.semver,
        source: info.source,
        git_commit: info.git_commit,
        git_dirty: info.git_dirty,
        build_profile: info.build_profile,
    }
}

/// Docker-style text line for `version` rendering.
#[must_use]
pub fn runtime_version_line() -> String {
    crate::shared::version::runtime_version_line()
}
