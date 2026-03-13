//! Shared types for maintainer report modules.

use serde::{Deserialize, Serialize};

/// Canonical maintainer command namespace.
pub const MAINTAINER_COMMAND_NAMESPACE: &str = "bijux-dev-cli";

/// Canonical maintainer command grouping model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevCliCommandGroup {
    /// Top-level maintainer dashboard and health summaries.
    Dashboard,
    /// Runtime routing and registry diagnostics.
    Routing,
    /// Runtime/state/environment/package diagnostics.
    Runtime,
    /// Documentation and maintenance ownership audits.
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

/// Canonical maintainer command identity used by maintainer report modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DevCliCommand {
    /// `bijux-dev-cli status`
    Status,
    /// `bijux-dev-cli parity`
    Parity,
    /// `bijux-dev-cli routes`
    Routes,
    /// `bijux-dev-cli registry`
    Registry,
    /// `bijux-dev-cli env`
    Env,
    /// `bijux-dev-cli contracts`
    Contracts,
    /// `bijux-dev-cli config`
    Config,
    /// `bijux-dev-cli runtime-identity`
    RuntimeIdentity,
    /// `bijux-dev-cli package-health`
    PackageHealth,
    /// `bijux-dev-cli state-audit`
    StateAudit,
    /// `bijux-dev-cli state-doctor`
    StateDoctor,
    /// `bijux-dev-cli docs-audit`
    DocsAudit,
    /// `bijux-dev-cli maintenance`
    Maintenance,
    /// `bijux-dev-cli maintenance-audit`
    MaintenanceAudit,
    /// `bijux-dev-cli rustdoc`
    Rustdoc,
    /// `bijux-dev-cli release`
    Release,
    /// `bijux-dev-cli evidence`
    Evidence,
    /// `bijux-dev-cli python`
    Python,
    /// `bijux-dev-cli repo`
    Repo,
    /// `bijux-dev-cli crate-health`
    CrateHealth,
    /// `bijux-dev-cli route-audit`
    RouteAudit,
    /// `bijux-dev-cli doctor`
    Doctor,
    /// `bijux-dev-cli plugin-health`
    PluginHealth,
    /// `bijux-dev-cli snapshots-audit`
    SnapshotsAudit,
    /// `bijux-dev-cli fixture-audit`
    FixtureAudit,
    /// `bijux-dev-cli inventory`
    Inventory,
    /// `bijux-dev-cli docs`
    Docs,
    /// `bijux-dev-cli docs-prune-plan`
    DocsPrunePlan,
    /// `bijux-dev-cli atlas`
    Atlas,
    /// `bijux-dev-cli di`
    Di,
    /// `bijux-dev-cli list-products`
    ListProducts,
    /// `bijux-dev-cli list-plugins`
    ListPlugins,
    /// `bijux-dev-cli dashboard`
    Dashboard,
    /// `bijux-dev-cli quickcheck`
    Quickcheck,
    /// `bijux-dev-cli truth`
    Truth,
    /// `bijux-dev-cli blockers`
    Blockers,
    /// `bijux-dev-cli next`
    Next,
}

impl DevCliCommand {
    /// Returns the canonical command string for this command identity.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Status => "bijux-dev-cli status",
            Self::Parity => "bijux-dev-cli parity",
            Self::Routes => "bijux-dev-cli routes",
            Self::Registry => "bijux-dev-cli registry",
            Self::Env => "bijux-dev-cli env",
            Self::Contracts => "bijux-dev-cli contracts",
            Self::Config => "bijux-dev-cli config",
            Self::RuntimeIdentity => "bijux-dev-cli runtime-identity",
            Self::PackageHealth => "bijux-dev-cli package-health",
            Self::StateAudit => "bijux-dev-cli state-audit",
            Self::StateDoctor => "bijux-dev-cli state-doctor",
            Self::DocsAudit => "bijux-dev-cli docs-audit",
            Self::Maintenance => "bijux-dev-cli maintenance",
            Self::MaintenanceAudit => "bijux-dev-cli maintenance-audit",
            Self::Rustdoc => "bijux-dev-cli rustdoc",
            Self::Release => "bijux-dev-cli release",
            Self::Evidence => "bijux-dev-cli evidence",
            Self::Python => "bijux-dev-cli python",
            Self::Repo => "bijux-dev-cli repo",
            Self::CrateHealth => "bijux-dev-cli crate-health",
            Self::RouteAudit => "bijux-dev-cli route-audit",
            Self::Doctor => "bijux-dev-cli doctor",
            Self::PluginHealth => "bijux-dev-cli plugin-health",
            Self::SnapshotsAudit => "bijux-dev-cli snapshots-audit",
            Self::FixtureAudit => "bijux-dev-cli fixture-audit",
            Self::Inventory => "bijux-dev-cli inventory",
            Self::Docs => "bijux-dev-cli docs",
            Self::DocsPrunePlan => "bijux-dev-cli docs-prune-plan",
            Self::Atlas => "bijux-dev-cli atlas",
            Self::Di => "bijux-dev-cli di",
            Self::ListProducts => "bijux-dev-cli list-products",
            Self::ListPlugins => "bijux-dev-cli list-plugins",
            Self::Dashboard => "bijux-dev-cli dashboard",
            Self::Quickcheck => "bijux-dev-cli quickcheck",
            Self::Truth => "bijux-dev-cli truth",
            Self::Blockers => "bijux-dev-cli blockers",
            Self::Next => "bijux-dev-cli next",
        }
    }

    /// Returns the command group for ownership inventory and reporting.
    #[must_use]
    pub const fn group(self) -> DevCliCommandGroup {
        match self {
            Self::Status
            | Self::Parity
            | Self::Doctor
            | Self::Dashboard
            | Self::Quickcheck
            | Self::Truth
            | Self::Blockers
            | Self::Next => DevCliCommandGroup::Dashboard,
            Self::Routes | Self::Registry | Self::RouteAudit => DevCliCommandGroup::Routing,
            Self::Env
            | Self::Contracts
            | Self::Config
            | Self::RuntimeIdentity
            | Self::PackageHealth
            | Self::StateAudit
            | Self::StateDoctor
            | Self::PluginHealth => DevCliCommandGroup::Runtime,
            Self::DocsAudit
            | Self::Maintenance
            | Self::MaintenanceAudit
            | Self::Rustdoc
            | Self::Release
            | Self::Evidence
            | Self::Python
            | Self::Repo
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
            command: DevCliCommand::Config,
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
            command: DevCliCommand::Maintenance,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::MaintenanceAudit,
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
            command: DevCliCommand::Python,
            group: DevCliCommandGroup::Audit,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Repo,
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
            command: DevCliCommand::Dashboard,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Quickcheck,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Truth,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Blockers,
            group: DevCliCommandGroup::Dashboard,
            visible: true,
            owner: "bijux-dev-cli",
        },
        DevCliCommandMetadata {
            command: DevCliCommand::Next,
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
