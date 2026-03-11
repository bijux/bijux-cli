mod diagnostics_ownership;
mod orchestration;
mod scope_bridge;

use crate::domain::automation_contracts::{Path, Value};

pub(super) fn run(workspace_root: &Path, contract_id: &str) -> Option<Value> {
    orchestration::run(workspace_root, contract_id)
        .or_else(|| diagnostics_ownership::run(workspace_root, contract_id))
        .or_else(|| scope_bridge::run(workspace_root, contract_id))
}
