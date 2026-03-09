#![forbid(unsafe_code)]
//! Shared durable contracts for all Rust bijux-cli crates.

use serde::{Deserialize, Serialize};

/// Stable result marker used by integration boundaries.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContractMarker {
    /// Contract namespace identifier.
    pub namespace: String,
}
