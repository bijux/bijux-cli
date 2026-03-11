use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::compatibility_reports_executor::run(workspace_root, contract_id)
        .or_else(|| super::install_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::runtime_resilience_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::plugin_runtime_reports_executor::run(workspace_root, contract_id))
}
