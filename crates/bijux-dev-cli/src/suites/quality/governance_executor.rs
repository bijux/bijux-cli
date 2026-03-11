use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::governance_state_laws_executor::run(workspace_root, contract_id)
        .or_else(|| super::governance_command_surface_executor::run(workspace_root, contract_id))
}
