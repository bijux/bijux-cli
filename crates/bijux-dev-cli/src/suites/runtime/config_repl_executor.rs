use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::config_surface_reports_executor::run(workspace_root, contract_id)
        .or_else(|| super::bridge_and_repl_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::runtime_law_reports_executor::run(workspace_root, contract_id))
}
