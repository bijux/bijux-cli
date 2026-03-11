use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::ownership_invariants::run(workspace_root, contract_id)
        .or_else(|| super::stale_artifacts::run(workspace_root, contract_id))
        .or_else(|| super::ownership_diagnostics_surface::run(workspace_root, contract_id))
}
