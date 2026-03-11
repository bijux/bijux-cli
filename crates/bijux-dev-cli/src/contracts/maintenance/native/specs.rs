#[path = "spec_control_plane.rs"]
mod control_plane;
#[path = "spec_quality.rs"]
mod quality;
#[path = "spec_resilience.rs"]
mod resilience;
#[path = "spec_runtime.rs"]
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
