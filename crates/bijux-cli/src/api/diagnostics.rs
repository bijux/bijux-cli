#![forbid(unsafe_code)]
//! Read-only diagnostics and inventory query facade.

/// State-path and corruption diagnostics query helpers.
pub mod state_paths {
    pub use crate::features::diagnostics::state_paths::{
        env_map, resolve_state_paths, state_diagnostics, state_path_status_value,
        ResolvedStatePaths,
    };
}

pub use crate::features::diagnostics::{
    parity_status_query, registry_inventory, route_inventory, state_diagnostics_query,
    ParityStatusQuery, RegistryInventory, RouteInventory, StateDiagnosticsQuery, StatePathStatus,
};
