#[path = "executor_control_plane_diagnostics_ownership.rs"]
mod diagnostics_ownership;
#[path = "executor_control_plane_orchestration.rs"]
mod orchestration;
#[path = "executor_control_plane_scope_bridge.rs"]
mod scope_bridge;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    orchestration::run(workspace_root, contract_id)
        .or_else(|| diagnostics_ownership::run(workspace_root, contract_id))
        .or_else(|| scope_bridge::run(workspace_root, contract_id))
}
