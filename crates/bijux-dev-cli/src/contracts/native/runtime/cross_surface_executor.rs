use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::cross_surface_foundation_executor::run(workspace_root, contract_id)
        .or_else(|| super::cross_surface_consistency_executor::run(workspace_root, contract_id))
}
