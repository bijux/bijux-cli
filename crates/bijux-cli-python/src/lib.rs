#![forbid(unsafe_code)]
//! Python compatibility bridge surfaces.

use bijux_cli_contracts::ContractMarker;

/// Build python-bridge marker.
#[must_use]
pub fn python_bridge_marker() -> ContractMarker {
    ContractMarker { namespace: "python-bridge".to_string() }
}
