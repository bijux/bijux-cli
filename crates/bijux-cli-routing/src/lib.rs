#![forbid(unsafe_code)]
//! Routing graph and namespace resolution surfaces.

pub mod parser;
pub mod registry;

use bijux_cli_contracts::ContractMarker;

/// Resolve the initial routing marker.
#[must_use]
pub fn route_marker() -> ContractMarker {
    ContractMarker { namespace: "core:routing".to_string() }
}

#[cfg(test)]
use proptest as _;

#[cfg(test)]
use serde as _;
#[cfg(test)]
use serde_json as _;
