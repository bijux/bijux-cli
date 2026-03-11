use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::cross_surface_foundation::run(workspace_root, contract_id)
        .or_else(|| super::cross_surface_consistency::run(workspace_root, contract_id))
}
