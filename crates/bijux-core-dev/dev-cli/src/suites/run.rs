use crate::contracts::maintenance::{Path, Value};

use super::{control_plane, quality, resilience, runtime};

pub(crate) fn run_native_status_contract(
    workspace_root: &Path,
    contract_id: &str,
) -> Option<Value> {
    control_plane::run(workspace_root, contract_id)
        .or_else(|| runtime::run(workspace_root, contract_id))
        .or_else(|| resilience::run(workspace_root, contract_id))
        .or_else(|| quality::run(workspace_root, contract_id))
}
