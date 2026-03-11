use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    super::ownership_invariants_executor::run(workspace_root, contract_id)
        .or_else(|| super::stale_artifact_gate_executor::run(workspace_root, contract_id))
        .or_else(|| super::diagnostics_surface_reports_executor::run(workspace_root, contract_id))
}
