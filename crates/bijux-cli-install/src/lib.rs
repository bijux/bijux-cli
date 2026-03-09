#![forbid(unsafe_code)]
//! Installation and distribution surfaces.

use bijux_cli_contracts::ContractMarker;

/// Build installer marker.
#[must_use]
pub fn installer_marker() -> ContractMarker {
    ContractMarker { namespace: "install".to_string() }
}
