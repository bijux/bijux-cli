//! Root command handlers.

use std::path::Path;

use serde_json::Value;

use crate::features::diagnostics::state_paths::ResolvedStatePaths;

use super::cli::{docs_inventory_report, runtime_audit_report, runtime_status_report};

pub(crate) fn try_handle(
    normalized_path: &[String],
    _argv: &[String],
    paths: &ResolvedStatePaths,
    plugin_registry_path: &Path,
) -> Option<Value> {
    match normalized_path {
        [a] if a == "status" => Some(runtime_status_report(paths, plugin_registry_path)),
        [a] if a == "audit" => Some(runtime_audit_report(paths, plugin_registry_path)),
        [a] if a == "docs" => Some(docs_inventory_report()),
        _ => None,
    }
}
