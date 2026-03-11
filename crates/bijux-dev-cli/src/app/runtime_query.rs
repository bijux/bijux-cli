//! Runtime query contracts used by dev-cli command routing.

use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    commands::env as dev_env, commands::registry as dev_registry,
    commands::runtime_identity as dev_runtime_identity, commands::state_audit as dev_state_audit,
    control_plane as dev_control_plane,
};

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
