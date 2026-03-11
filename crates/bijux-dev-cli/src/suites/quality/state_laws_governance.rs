use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::state_laws::run(workspace_root, contract_id)
        .or_else(|| super::command_surface_governance::run(workspace_root, contract_id))
}
