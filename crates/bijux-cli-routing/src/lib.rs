#![forbid(unsafe_code)]
//! Routing graph and namespace resolution surfaces.

use bijux_cli_contracts::ContractMarker;
use bijux_cli_core::core_marker;

/// Resolve the initial routing marker.
#[must_use]
pub fn route_marker() -> ContractMarker {
    let mut marker = core_marker();
    marker.namespace = format!("{}:routing", marker.namespace);
    marker
}
