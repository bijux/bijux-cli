#[path = "executor_quality_governance_command_surface.rs"]
mod command_surface;
#[path = "executor_quality_governance_state_laws.rs"]
mod state_laws;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    state_laws::run(workspace_root, contract_id)
        .or_else(|| command_surface::run(workspace_root, contract_id))
}
