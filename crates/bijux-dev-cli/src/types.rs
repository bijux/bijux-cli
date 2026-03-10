//! Shared types for dev-cli maintainer report modules.

use serde::{Deserialize, Serialize};

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
    /// `bijux dev cli docs-audit`
    DocsAudit,
    /// `bijux dev cli script-audit`
    ScriptAudit,
    /// `bijux dev cli crate-health`
    CrateHealth,
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
            Self::DocsAudit => "dev cli docs-audit",
            Self::ScriptAudit => "dev cli script-audit",
            Self::CrateHealth => "dev cli crate-health",
        }
    }
}

/// Shared immutable context for report assembly.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportContext {
    /// UTC timestamp encoded by caller in ISO-8601 form.
    pub generated_at: String,
    /// Source component providing low-level structured data.
    pub data_source: String,
}
