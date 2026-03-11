use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::install_runtime_compatibility::run(workspace_root, contract_id)
        .or_else(|| super::install_runtime_reports::run(workspace_root, contract_id))
        .or_else(|| super::repl_bridge_resilience::run(workspace_root, contract_id))
        .or_else(|| super::repl_bridge_plugins::run(workspace_root, contract_id))
}
