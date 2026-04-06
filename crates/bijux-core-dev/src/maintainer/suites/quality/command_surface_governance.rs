use crate::contracts::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::command_surface::run(workspace_root, contract_id)
        .or_else(|| super::plugin_quality_compatibility::run(workspace_root, contract_id))
}
