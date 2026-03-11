use crate::contracts::maintenance::Value;

use super::{control_plane, quality, resilience, runtime};

pub(crate) fn native_status_contract_rows() -> Vec<Value> {
    let mut rows = Vec::new();
    rows.extend(control_plane::rows());
    rows.extend(runtime::rows());
    rows.extend(resilience::rows());
    rows.extend(quality::rows());
    rows
}
