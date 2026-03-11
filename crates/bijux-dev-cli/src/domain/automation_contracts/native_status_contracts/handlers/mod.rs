mod control_plane;
mod quality_release;
mod resilience;
mod runtime_surfaces;

#[allow(clippy::wildcard_imports)]
use crate::domain::automation_contracts::*;

pub(crate) fn run_native_status_contract(
    workspace_root: &Path,
    contract_id: &str,
) -> Option<Value> {
    control_plane::run(workspace_root, contract_id)
        .or_else(|| runtime_surfaces::run(workspace_root, contract_id))
        .or_else(|| resilience::run(workspace_root, contract_id))
        .or_else(|| quality_release::run(workspace_root, contract_id))
}
