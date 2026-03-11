#[path = "executor_control_plane.rs"]
mod control_plane;
#[path = "executor_quality.rs"]
mod quality;
#[path = "executor_resilience.rs"]
mod resilience;
#[path = "executor_runtime.rs"]
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
