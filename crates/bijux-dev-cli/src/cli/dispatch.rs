//! Maintainer command dispatch for `bijux dev cli ...`.

use std::collections::BTreeMap;

use anyhow::Result;
use serde_json::Value;

use crate::reports::{
    control_plane as dev_control_plane, env as dev_env, registry as dev_registry,
    runtime_identity as dev_runtime_identity, state_audit as dev_state_audit,
};

#[path = "routes/config.rs"]
mod config;
#[path = "routes/evidence.rs"]
mod evidence;
#[path = "routes/maintenance.rs"]
mod maintenance;
#[path = "routes/python.rs"]
mod python;
#[path = "routes/release.rs"]
mod release;
#[path = "routes/root.rs"]
mod root;
#[path = "routes/rustdoc.rs"]
mod rustdoc;

/// Runtime route inventory queried by maintainer route diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RouteInventoryQuery {
    /// Canonical route segment lists.
    pub routes: Vec<Vec<String>>,
    /// Alias rewrite pairs as `(alias_segments, canonical_segments)`.
    pub aliases: Vec<(Vec<String>, Vec<String>)>,
}

/// Runtime-derived input for `dev cli doctor` report assembly.
#[derive(Debug, Clone)]
pub struct DoctorReportInput {
    /// Configuration loading and shape issues.
    pub config_issues: Vec<Value>,
    /// PATH/install diagnostics issues.
    pub path_issues: Vec<Value>,
    /// Plugin diagnostics surfaced at load time.
    pub plugin_issues: Vec<Value>,
    /// History state diagnostics surfaced from runtime state checks.
    pub history_issues: Vec<Value>,
    /// Memory state diagnostics surfaced from runtime state checks.
    pub memory_issues: Vec<Value>,
}

/// Runtime-derived input for `dev cli state-audit` report assembly.
#[derive(Debug, Clone)]
pub struct StateAuditInput {
    /// Structured path status data.
    pub path_status: dev_state_audit::StatePathStatusInput,
    /// Corruption/repair diagnostics.
    pub corruption_health: Value,
}

/// Runtime-derived schema contracts input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractsSchemaInput {
    /// Stable schema ids.
    pub schema_ids: Vec<String>,
    /// Schema version marker.
    pub schema_version: String,
}

/// Runtime-owned query adapter used by dev-cli command dispatch.
pub trait RuntimeQueryProvider {
    /// Return route inventory rows from runtime routing services.
    fn route_inventory(&self) -> RouteInventoryQuery;

    /// Return namespace inventory rows from runtime registry services.
    fn registry_inventory(&self) -> Vec<dev_registry::NamespaceInventoryRow>;

    /// Return currently installed plugins for maintainer visibility.
    fn plugin_list(&self) -> Vec<Value>;

    /// Return canonical product mount contracts owned by Bijux projects.
    fn product_contracts(&self) -> Vec<dev_control_plane::ProductContractRow>;

    /// Return filtered runtime environment values used by CLI state resolution.
    fn env_map(&self) -> BTreeMap<String, String>;

    /// Return runtime-resolved active path set.
    fn active_paths(&self) -> dev_env::ActivePaths;

    /// Return runtime diagnostics for doctor report assembly.
    fn doctor_report_input(&self) -> DoctorReportInput;

    /// Return runtime diagnostics for state-audit report assembly.
    fn state_audit_input(&self) -> StateAuditInput;

    /// Return runtime diagnosis payload for state-doctor report assembly.
    fn state_doctor_report(&self) -> Value;

    /// Return structured contracts schema data from runtime routing services.
    fn contracts_schema_input(&self) -> ContractsSchemaInput;

    /// Return runtime identity diagnostics and channel metadata.
    fn runtime_identity_input(&self) -> dev_runtime_identity::RuntimeIdentityInput;
}

/// Return true when the normalized path belongs to `dev cli` dispatch ownership.
#[must_use]
pub fn owns_path(normalized_path: &[String]) -> bool {
    match normalized_path {
        [a, b, c]
            if a == "dev"
                && b == "cli"
                && matches!(
                    c.as_str(),
                    "routes"
                        | "atlas"
                        | "di"
                        | "list-products"
                        | "list-plugins"
                        | "route-audit"
                        | "inventory"
                        | "registry"
                        | "parity"
                        | "docs"
                        | "status"
                        | "maintenance-audit"
                        | "snapshots-audit"
                        | "fixture-audit"
                        | "crate-health"
                        | "package-health"
                        | "env"
                        | "doctor"
                        | "docs-prune-plan"
                        | "state-audit"
                        | "state-doctor"
                        | "dashboard"
                        | "quickcheck"
                        | "truth"
                        | "blockers"
                        | "next"
                        | "docs-audit"
                        | "plugin-health"
                        | "contracts"
                        | "runtime-identity"
                ) =>
        {
            true
        }
        [a, b, c, _]
            if a == "dev"
                && b == "cli"
                && matches!(
                    c.as_str(),
                    "maintenance"
                        | "rustdoc"
                        | "release"
                        | "evidence"
                        | "config"
                        | "python"
                        | "repo"
                ) =>
        {
            true
        }
        [a, b, c, d, _] if a == "dev" && b == "cli" && c == "maintenance" && d == "status" => true,
        _ => false,
    }
}

/// Dispatch `dev cli` command paths and return report payloads.
pub fn try_handle(
    normalized_path: &[String],
    argv: &[String],
    runtime: &dyn RuntimeQueryProvider,
) -> Result<Option<Value>> {
    if let Some(payload) = root::try_handle(normalized_path, argv, runtime) {
        return Ok(Some(payload));
    }
    if let Some(payload) = maintenance::try_handle(normalized_path, argv)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = rustdoc::try_handle(normalized_path) {
        return Ok(Some(payload));
    }
    if let Some(payload) = release::try_handle(normalized_path) {
        return Ok(Some(payload));
    }
    if let Some(payload) = evidence::try_handle(normalized_path, argv)? {
        return Ok(Some(payload));
    }
    if let Some(payload) = config::try_handle(normalized_path) {
        return Ok(Some(payload));
    }
    if let Some(payload) = python::try_handle(normalized_path) {
        return Ok(Some(payload));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::owns_path;

    #[test]
    fn owns_path_matches_dev_cli_dispatch_surface() {
        assert!(owns_path(&["dev".into(), "cli".into(), "status".into()]));
        assert!(owns_path(&["dev".into(), "cli".into(), "maintenance".into(), "audit".into()]));
        assert!(owns_path(&["dev".into(), "cli".into(), "release".into(), "status".into()]));

        assert!(!owns_path(&["dev".into(), "status".into()]));
        assert!(!owns_path(&["cli".into(), "status".into()]));
        assert!(!owns_path(&["dev".into(), "cli".into(), "not-a-command".into()]));
        assert!(!owns_path(&["dev".into(), "cli".into(), "unknown".into(), "leaf".into()]));
    }
}
