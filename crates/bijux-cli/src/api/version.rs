#![forbid(unsafe_code)]
//! Runtime version API.

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
