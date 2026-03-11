#[path = "control_plane_diagnostics_ownership_executor.rs"]
mod diagnostics_ownership;
#[path = "control_plane_orchestration_executor.rs"]
mod orchestration;
#[path = "control_plane_scope_bridge_executor.rs"]
mod scope_bridge;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    orchestration::run(workspace_root, contract_id)
        .or_else(|| diagnostics_ownership::run(workspace_root, contract_id))
        .or_else(|| scope_bridge::run(workspace_root, contract_id))
}
