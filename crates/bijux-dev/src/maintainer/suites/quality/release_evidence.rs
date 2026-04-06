use crate::contracts::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::release_evidence_reports::run(workspace_root, contract_id)
        .or_else(|| super::plugin_quality::run(workspace_root, contract_id))
        .or_else(|| super::command_surface_config::run(workspace_root, contract_id))
        .or_else(|| super::command_surface_diagnostics::run(workspace_root, contract_id))
        .or_else(|| super::plugin_quality_status::run(workspace_root, contract_id))
        .or_else(|| super::command_surface_control_plane::run(workspace_root, contract_id))
        .or_else(|| super::command_surface_crate_boundaries::run(workspace_root, contract_id))
}
