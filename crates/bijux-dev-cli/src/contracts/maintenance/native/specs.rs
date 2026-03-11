#[path = "control_plane_spec.rs"]
mod control_plane;
#[path = "quality_spec.rs"]
mod quality;
#[path = "resilience_spec.rs"]
mod resilience;
#[path = "runtime_spec.rs"]
mod runtime;

#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(crate) fn native_status_contract_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(control_plane::rows());
    rows.extend(runtime::rows());
    rows.extend(resilience::rows());
    rows.extend(quality::rows());
    rows
}
