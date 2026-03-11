use std::path::Path;

use serde_json::Value;

use crate::status_contracts;

/// Builds `dev cli maintenance status inventory` report payload.
#[must_use]
pub fn build_status_contracts_report(workspace_root: &Path) -> Value {
    status_contracts::build_inventory_report(workspace_root)
}

/// Backward-compatible alias for legacy callsites.
#[must_use]
pub fn build_status_maintenance_report(workspace_root: &Path) -> Value {
    build_status_contracts_report(workspace_root)
}

/// Runs one status contract by stable id.
#[must_use]
pub fn run_status_contract(
    workspace_root: &Path,
    contract_id: Option<&str>,
    source_ref: Option<&str>,
    args: &[String],
) -> Value {
    status_contracts::run_contract(workspace_root, contract_id, source_ref, args)
}

/// Runs all status contracts, optionally filtered by kind.
#[must_use]
pub fn run_all_status_contracts(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    status_contracts::run_all_contracts(workspace_root, kind_filter, args)
}

/// Backward-compatible alias for legacy callsites.
#[must_use]
pub fn run_all_status_maintenance(
    workspace_root: &Path,
    kind_filter: Option<&str>,
    args: &[String],
) -> Value {
    run_all_status_contracts(workspace_root, kind_filter, args)
}
