#![forbid(unsafe_code)]
//! REPL orchestration boundaries.

use bijux_cli_contracts::ContractMarker;
use bijux_cli_routing::route_marker;

/// Build REPL marker chained from routing state.
#[must_use]
pub fn repl_marker() -> ContractMarker {
    let mut marker = route_marker();
    marker.namespace = format!("{}:repl", marker.namespace);
    marker
}
