#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

use bijux_cli_contracts::ContractMarker;

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

#[cfg(test)]
mod tests {
    use super::*;

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
}
