use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::state_quality_reports_executor::run(workspace_root, contract_id)
        .or_else(|| super::state_laws_reports_executor::run(workspace_root, contract_id))
        .or_else(|| super::history_behavior_reports_executor::run(workspace_root, contract_id))
}
