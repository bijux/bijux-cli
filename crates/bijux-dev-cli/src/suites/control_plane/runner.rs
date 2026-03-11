use crate::contract_engine::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::orchestration_executor::run(workspace_root, contract_id)
        .or_else(|| super::diagnostics_ownership_executor::run(workspace_root, contract_id))
        .or_else(|| super::scope_bridge_executor::run(workspace_root, contract_id))
}
