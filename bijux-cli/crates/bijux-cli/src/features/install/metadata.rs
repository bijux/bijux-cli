#![forbid(unsafe_code)]
//! Install metadata and package-channel strategy contracts.

use crate::contracts::{known_bijux_tool, known_bijux_tools};

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

/// Canonical install target after alias resolution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallTarget {
    /// Canonical short target name accepted by `bijux install`.
    pub target_name: String,
    /// Strategy to install the requested target.
    pub strategy: InstallStrategy,
}

/// Decide canonical crate naming strategy.
#[must_use]
pub fn canonical_crate_name() -> &'static str {
    "bijux-cli"
}

fn cargo_install_strategy_for(package_name: &str, executable_name: &str) -> InstallStrategy {
    InstallStrategy {
        ecosystem: Ecosystem::Cargo,
        package_name: package_name.to_string(),
        executable_name: executable_name.to_string(),
    }
}

fn dev_cli_install_strategy() -> InstallStrategy {
    cargo_install_strategy_for("bijux-dev-cli", "bijux-dev-cli")
}

/// Build cargo install strategy for the documented public package.
#[must_use]
pub fn cargo_install_strategy() -> InstallStrategy {
    cargo_install_strategy_for("bijux-cli", CANONICAL_EXECUTABLE)
}

/// Build pip install strategy for the documented public package.
#[must_use]
pub fn pip_install_strategy() -> InstallStrategy {
    InstallStrategy {
        ecosystem: Ecosystem::Pip,
        package_name: "bijux-cli".to_string(),
        executable_name: CANONICAL_EXECUTABLE.to_string(),
    }
}

/// Resolve an install target alias into its canonical package strategy.
#[must_use]
pub fn resolve_install_target(raw_target: &str) -> Option<InstallTarget> {
    let normalized = raw_target.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    match normalized.as_str() {
        "cli" | "bijux-cli" => {
            return Some(InstallTarget {
                target_name: "cli".to_string(),
                strategy: cargo_install_strategy(),
            });
        }
        "dev-cli" | "bijux-dev-cli" => {
            return Some(InstallTarget {
                target_name: "dev-cli".to_string(),
                strategy: dev_cli_install_strategy(),
            });
        }
        _ => {}
    }

    if let Some(namespace) = normalized.strip_prefix("dev-") {
        let tool = known_bijux_tool(namespace)?;
        return Some(InstallTarget {
            target_name: format!("dev-{namespace}"),
            strategy: cargo_install_strategy_for(
                tool.control_package_name,
                tool.control_binary_name,
            ),
        });
    }

    if let Some(tool) = known_bijux_tool(&normalized) {
        return Some(InstallTarget {
            target_name: tool.namespace.to_string(),
            strategy: cargo_install_strategy_for(
                tool.runtime_package_name,
                tool.runtime_binary_name,
            ),
        });
    }

    known_bijux_tools().iter().find_map(|tool| {
        if normalized == tool.runtime_binary_name || normalized == tool.runtime_package_name {
            return Some(InstallTarget {
                target_name: tool.namespace.to_string(),
                strategy: cargo_install_strategy_for(
                    tool.runtime_package_name,
                    tool.runtime_binary_name,
                ),
            });
        }
        if normalized == tool.control_binary_name || normalized == tool.control_package_name {
            return Some(InstallTarget {
                target_name: format!("dev-{}", tool.namespace),
                strategy: cargo_install_strategy_for(
                    tool.control_package_name,
                    tool.control_binary_name,
                ),
            });
        }
        None
    })
}

/// List canonical short aliases supported by `bijux install`.
#[must_use]
pub fn install_target_aliases() -> Vec<String> {
    let mut aliases = vec!["cli".to_string(), "dev-cli".to_string()];
    for tool in known_bijux_tools() {
        aliases.push(tool.namespace.to_string());
        aliases.push(format!("dev-{}", tool.namespace));
    }
    aliases
}
