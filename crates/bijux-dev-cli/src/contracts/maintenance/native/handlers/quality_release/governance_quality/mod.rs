mod command_surface_governance;
mod state_quality_laws;

use crate::contract_engine::maintenance::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    state_quality_laws::run(workspace_root, contract_id)
        .or_else(|| command_surface_governance::run(workspace_root, contract_id))
}
