#![forbid(unsafe_code)]
//! Plugin registration and lifecycle boundaries.

use bijux_cli_contracts::ContractMarker;
use bijux_cli_core::core_marker;

/// Build plugin marker chained from core state.
#[must_use]
pub fn plugin_marker() -> ContractMarker {
    let mut marker = core_marker();
    marker.namespace = format!("{}:plugin", marker.namespace);
    marker
}
