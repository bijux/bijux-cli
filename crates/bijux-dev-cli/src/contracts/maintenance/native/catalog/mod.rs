mod control_plane;
mod quality_release;
mod resilience;
mod runtime_surfaces;

#[allow(clippy::wildcard_imports)]
use crate::contract_engine::maintenance::*;

pub(crate) fn native_status_contract_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(control_plane::rows());
    rows.extend(runtime_surfaces::rows());
    rows.extend(resilience::rows());
    rows.extend(quality_release::rows());
    rows
}
