use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::orchestration_reports_executor::run(workspace_root, contract_id)
        .or_else(|| super::orchestration_bundle_executor::run(workspace_root, contract_id))
}
