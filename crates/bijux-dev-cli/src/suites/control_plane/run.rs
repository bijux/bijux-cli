use crate::contract_engine::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::orchestration::run(workspace_root, contract_id)
        .or_else(|| super::ownership::run(workspace_root, contract_id))
        .or_else(|| super::ownership_scope_bridge::run(workspace_root, contract_id))
}
