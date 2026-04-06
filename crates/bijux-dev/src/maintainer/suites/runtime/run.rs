use crate::contracts::maintenance::{Path, Value};

pub(crate) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::install_runtime::run(workspace_root, contract_id)
        .or_else(|| super::config_surface::run(workspace_root, contract_id))
        .or_else(|| super::cross_surface::run(workspace_root, contract_id))
}
