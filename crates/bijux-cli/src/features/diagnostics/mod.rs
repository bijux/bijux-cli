#![forbid(unsafe_code)]
//! Read-only state and parity/status query interfaces for maintainer tooling.

mod parity_status;
mod routing_inventory;
mod state_diagnostics;
/// Runtime state-path resolution and diagnostics helpers consumed by maintainer tools.
pub mod state_paths;

pub use parity_status::{parity_status_query, ParityStatusQuery};
pub use routing_inventory::{registry_inventory, route_inventory, RegistryInventory, RouteInventory};
pub use state_diagnostics::{state_diagnostics_query, StateDiagnosticsQuery, StatePathStatus};
