#[path = "control_plane_executor.rs"]
mod control_plane;
#[path = "quality_executor.rs"]
mod quality;
#[path = "resilience_executor.rs"]
mod resilience;
#[path = "runtime_executor.rs"]
mod runtime;

#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(crate) fn run_native_status_contract(
    workspace_root: &Path,
    contract_id: &str,
) -> Option<Value> {
    control_plane::run(workspace_root, contract_id)
        .or_else(|| runtime::run(workspace_root, contract_id))
        .or_else(|| resilience::run(workspace_root, contract_id))
        .or_else(|| quality::run(workspace_root, contract_id))
}
