use crate::contract_engine::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::namespace_install_executor::run(workspace_root, contract_id)
        .or_else(|| super::config_repl_executor::run(workspace_root, contract_id))
        .or_else(|| super::cross_surface_executor::run(workspace_root, contract_id))
}
