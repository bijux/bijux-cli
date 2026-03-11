use crate::contract_engine::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::governance_executor::run(workspace_root, contract_id)
        .or_else(|| super::release_and_plugin_executor::run(workspace_root, contract_id))
}
