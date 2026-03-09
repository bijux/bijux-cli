#![forbid(unsafe_code)]
//! Core runtime primitives for Rust bijux-cli.

use bijux_cli_contracts::ContractMarker;

/// Build a canonical core marker for diagnostics.
#[must_use]
pub fn core_marker() -> ContractMarker {
    ContractMarker { namespace: "core".to_string() }
}
