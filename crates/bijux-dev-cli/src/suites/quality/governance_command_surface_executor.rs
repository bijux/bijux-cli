use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::command_surface_behavior_executor::run(workspace_root, contract_id)
        .or_else(|| super::compatibility_metadata_executor::run(workspace_root, contract_id))
}
