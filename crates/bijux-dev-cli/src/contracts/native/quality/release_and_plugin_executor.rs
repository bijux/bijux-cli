use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::release_reports_executor::run(workspace_root, contract_id)
        .or_else(|| super::plugin_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::config_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::diagnostics_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::status_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::maintainer_control_plane_executor::run(workspace_root, contract_id))
        .or_else(|| super::crate_boundary_metrics_executor::run(workspace_root, contract_id))
}
