#![forbid(unsafe_code)]
//! Install metadata and package-channel strategy contracts.

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

/// Decide canonical crate naming strategy.
#[must_use]
pub fn canonical_crate_name() -> &'static str {
    "bijux-cli"
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
