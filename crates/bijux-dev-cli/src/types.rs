//! Shared types for dev-cli maintainer report modules.

use serde::{Deserialize, Serialize};

/// Canonical maintainer command namespace.
pub const MAINTAINER_COMMAND_NAMESPACE: &str = "dev cli";

/// Canonical maintainer command grouping model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevCliCommandGroup {
    /// Top-level maintainer dashboard and health summaries.
    Dashboard,
    /// Runtime routing and registry diagnostics.
    Routing,
    /// Runtime/state/environment/package diagnostics.
    Runtime,
    /// Documentation and script ownership audits.
    Audit,
    /// Internal maintainer inventory and hidden probes.
    Internal,
}

impl DevCliCommandGroup {
    /// Returns the stable group key for machine-readable payloads.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Dashboard => "dashboard",
            Self::Routing => "routing",
            Self::Runtime => "runtime",
            Self::Audit => "audit",
            Self::Internal => "internal",
        }
    }
}

/// Canonical dev-cli command identity used by maintainer report modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevCliCommand {
    /// `bijux dev cli status`
    Status,
    /// `bijux dev cli parity`
    Parity,
    /// `bijux dev cli routes`
    Routes,
    /// `bijux dev cli registry`
    Registry,
    /// `bijux dev cli env`
    Env,
    /// `bijux dev cli contracts`
    Contracts,
    /// `bijux dev cli runtime-identity`
    RuntimeIdentity,
    /// `bijux dev cli package-health`
    PackageHealth,
    /// `bijux dev cli state-audit`
    StateAudit,
    /// `bijux dev cli state-doctor`
    StateDoctor,
    /// `bijux dev cli docs-audit`
    DocsAudit,
    /// `bijux dev cli scripts`
    Scripts,
    /// `bijux dev cli script-audit`
    ScriptAudit,
    /// `bijux dev cli rustdoc`
    Rustdoc,
    /// `bijux dev cli release`
    Release,
    /// `bijux dev cli evidence`
    Evidence,
    /// `bijux dev cli crate-health`
    CrateHealth,
    /// `bijux dev cli route-audit`
    RouteAudit,
    /// `bijux dev cli doctor`
    Doctor,
    /// `bijux dev cli plugin-health`
    PluginHealth,
    /// `bijux dev cli snapshots-audit`
    SnapshotsAudit,
    /// `bijux dev cli fixture-audit`
    FixtureAudit,
    /// `bijux dev cli inventory`
    Inventory,
    /// `bijux dev cli docs`
    Docs,
    /// `bijux dev cli docs-prune-plan`
    DocsPrunePlan,
    /// `bijux dev cli atlas`
    Atlas,
    /// `bijux dev cli di`
    Di,
    /// `bijux dev cli list-products`
    ListProducts,
    /// `bijux dev cli list-plugins`
    ListPlugins,
}

impl DevCliCommand {
    /// Returns the canonical command string for this command identity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "dev cli status",
            Self::Parity => "dev cli parity",
            Self::Routes => "dev cli routes",
            Self::Registry => "dev cli registry",
            Self::Env => "dev cli env",
            Self::Contracts => "dev cli contracts",
            Self::RuntimeIdentity => "dev cli runtime-identity",
            Self::PackageHealth => "dev cli package-health",
            Self::StateAudit => "dev cli state-audit",
            Self::StateDoctor => "dev cli state-doctor",
            Self::DocsAudit => "dev cli docs-audit",
            Self::Scripts => "dev cli scripts",
            Self::ScriptAudit => "dev cli script-audit",
            Self::Rustdoc => "dev cli rustdoc",
            Self::Release => "dev cli release",
            Self::Evidence => "dev cli evidence",
            Self::CrateHealth => "dev cli crate-health",
            Self::RouteAudit => "dev cli route-audit",
            Self::Doctor => "dev cli doctor",
            Self::PluginHealth => "dev cli plugin-health",
            Self::SnapshotsAudit => "dev cli snapshots-audit",
            Self::FixtureAudit => "dev cli fixture-audit",
            Self::Inventory => "dev cli inventory",
            Self::Docs => "dev cli docs",
            Self::DocsPrunePlan => "dev cli docs-prune-plan",
            Self::Atlas => "dev cli atlas",
            Self::Di => "dev cli di",
            Self::ListProducts => "dev cli list-products",
            Self::ListPlugins => "dev cli list-plugins",
        }
    }

    /// Returns the command group for ownership inventory and reporting.
    #[must_use]
    pub const fn group(self) -> DevCliCommandGroup {
        match self {
            Self::Status | Self::Parity | Self::Doctor => DevCliCommandGroup::Dashboard,
            Self::Routes | Self::Registry | Self::RouteAudit => DevCliCommandGroup::Routing,
            Self::Env
            | Self::Contracts
            | Self::RuntimeIdentity
            | Self::PackageHealth
            | Self::StateAudit
            | Self::StateDoctor
            | Self::PluginHealth => DevCliCommandGroup::Runtime,
            Self::DocsAudit
            | Self::Scripts
            | Self::ScriptAudit
            | Self::Rustdoc
            | Self::Release
            | Self::Evidence
            | Self::CrateHealth
            | Self::SnapshotsAudit
            | Self::FixtureAudit
            | Self::Docs
            | Self::DocsPrunePlan => DevCliCommandGroup::Audit,
            Self::Inventory | Self::Atlas | Self::Di | Self::ListProducts | Self::ListPlugins => {
                DevCliCommandGroup::Internal
            }
        }
    }
}

/// Canonical maintainer command registry entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DevCliCommandMetadata {
    /// Stable command identity.
    pub command: DevCliCommand,
    /// Stable command grouping.
    pub group: DevCliCommandGroup,
    /// Whether the command is maintainer-facing in normal usage.
    pub visible: bool,
    /// Stable owner declaration.
    pub owner: &'static str,
}

/// Canonical maintainer command metadata source.
#[must_use]
pub const fn command_registry() -> &'static [DevCliCommandMetadata] {
    &[
        DevCliCommandMetadata {
            command: DevCliCommand::Status,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Parity,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Routes,
            group: DevCliCommandGroup::Routing,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Registry,
            group: DevCliCommandGroup::Routing,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Env,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Contracts,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::RuntimeIdentity,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::PackageHealth,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::StateAudit,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::StateDoctor,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::DocsAudit,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Scripts,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::ScriptAudit,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Rustdoc,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Release,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Evidence,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::CrateHealth,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::RouteAudit,
            group: DevCliCommandGroup::Routing,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Doctor,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::PluginHealth,
            group: DevCliCommandGroup::Runtime,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::SnapshotsAudit,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::FixtureAudit,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Inventory,
            group: DevCliCommandGroup::Internal,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Docs,
            group: DevCliCommandGroup::Audit,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::DocsPrunePlan,
            group: DevCliCommandGroup::Audit,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Atlas,
            group: DevCliCommandGroup::Internal,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Di,
            group: DevCliCommandGroup::Internal,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::ListProducts,
            group: DevCliCommandGroup::Internal,
            visible: false,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::ListPlugins,
            group: DevCliCommandGroup::Internal,
            visible: false,
            owner: "bijux-dev-cli",
        },
    ]
}

/// Shared immutable context for report assembly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportContext {
    /// UTC timestamp encoded by caller in ISO-8601 form.
    pub generated_at: String,
    /// Source component providing low-level structured data.
    pub data_source: String,
}
