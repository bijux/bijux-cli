use crate::contracts::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::plugin_quality_state::run(workspace_root, contract_id)
        .or_else(|| super::state_laws_reports::run(workspace_root, contract_id))
        .or_else(|| super::command_surface_history::run(workspace_root, contract_id))
}
